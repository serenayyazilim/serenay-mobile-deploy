#!/usr/bin/env ruby
# encoding: utf-8
# Fetches store versions for all projects and updates their version.json files.
# Usage: ruby sync_versions.rb /path/to/sermobilepro [project_id]

Encoding.default_external = Encoding::UTF_8
Encoding.default_internal = Encoding::UTF_8

require 'json'
require 'fileutils'
require 'open3'

STDOUT.sync = true
STDERR.sync = true

workspace = ARGV[0] || Dir.pwd
ONLY_PROJECT = ARGV[1].to_s.empty? ? nil : ARGV[1]

CONFIG_FILE = File.join(workspace, 'sermobileboss_config.json')
BUNDLE_ID_PREFIX = if File.exist?(CONFIG_FILE)
  (JSON.parse(File.read(CONFIG_FILE))['bundleIdPrefix'] rescue nil)
else
  nil
end

PROJECTS_FILE  = File.join(workspace, 'sermobileboss_projects.json')
PROJECTS_PATH  = File.join(workspace, 'lib/conf/sermobplus-projects')
ANDROID_DIR    = File.join(workspace, 'android')
IOS_APPFILE    = File.join(workspace, 'ios/fastlane/Appfile')

# ─── Helpers ────────────────────────────────────────────────────────────────

def log(emoji, msg)  = puts("#{emoji} #{msg}")
def err(msg)         = STDERR.puts("❌ #{msg}")

# "version" stays the deploy source of truth (Android first, iOS fallback);
# "android" / "ios" hold each store's own latest version.
def save_version(project_id, fields)
  dir  = File.join(PROJECTS_PATH, project_id)
  return false unless File.directory?(dir)

  file = File.join(dir, 'version.json')
  data = (JSON.parse(File.read(file)) rescue {})
  data = {} unless data.is_a?(Hash)
  fields.each { |k, v| data[k] = v if v }
  File.write(file, JSON.pretty_generate(data))
  true
end

def read_stored(project_id)
  file = File.join(PROJECTS_PATH, project_id, 'version.json')
  JSON.parse(File.read(file)) rescue {}
end

# ─── Project list ───────────────────────────────────────────────────────────

unless File.exist?(PROJECTS_FILE)
  err("sermobileboss_projects.json not found: #{PROJECTS_FILE}")
  exit 1
end

projects = JSON.parse(File.read(PROJECTS_FILE))
if ONLY_PROJECT
  projects = projects.select { |id, _| id == ONLY_PROJECT }
  if projects.empty?
    err("#{ONLY_PROJECT} not found in #{PROJECTS_FILE}")
    exit 1
  end
end
log("📋", "#{projects.size} projects found")

results = { updated: [], skipped: [], failed: [] }

# ─── Android — Google Play API ───────────────────────────────────────────────

log("🤖", "Android — fetching Google Play versions...")

android_versions = {}
begin
  raise "Bundle ID prefix not set: #{CONFIG_FILE} (fill it in via Workspace Settings)" unless BUNDLE_ID_PREFIX

  require 'google/apis/androidpublisher_v3'
  require 'googleauth'

  json_key = Dir[File.join(ANDROID_DIR, 'api-*.json')].first
  raise "Google Play API key not found: #{ANDROID_DIR}/api-*.json" unless json_key

  log("🔑", "API key: #{File.basename(json_key)}")

  service = Google::Apis::AndroidpublisherV3::AndroidPublisherService.new
  service.authorization = Google::Auth::ServiceAccountCredentials.make_creds(
    json_key_io: File.open(json_key),
    scope: 'https://www.googleapis.com/auth/androidpublisher'
  )

  projects.each_key do |project_id|
    package = "#{BUNDLE_ID_PREFIX}.#{project_id}"
    begin
      edit  = service.insert_edit(package)
      tracks = service.list_edit_tracks(package, edit.id)

      # Find the most recent completed release on the production track
      production = tracks.tracks&.find { |t| t.track == 'production' }
      release    = production&.releases&.find { |r| r.status == 'completed' } ||
                   production&.releases&.first

      if release && release.name && release.version_codes&.any?
        raw_name     = release.name.to_s.strip
        version_code = release.version_codes.first.to_i

        # Extract X.Y.Z from formats like "54700000 (5.4.7)" or "1 (1.0.0)"
        version_name = if raw_name =~ /\((\d+\.\d+[\.\d]*)\)/
          $1
        elsif raw_name =~ /^(\d+\.\d+[\.\d]*)/
          $1
        else
          raw_name
        end

        version_str = "#{version_name}+#{version_code}"
        android_versions[project_id] = version_str
        log("  ✅", "#{project_id}: #{version_str}")
      else
        log("  ⚠️", "#{project_id}: no production release found")
      end

      begin; service.delete_edit(package, edit.id); rescue; end
    rescue => e
      log("  ⚠️", "#{project_id}: #{e.message.lines.first.strip}")
    end
  end
rescue => e
  err("Failed to initialize Google Play API: #{e.message}")
end

# ─── iOS — App Store Connect (fastlane, the same way deploy.rb runs it) ──────────────────────────

log("🍎", "iOS — fetching App Store Connect versions...")

ios_versions = {}
ios_dates = {}
ios_dir = File.join(workspace, 'ios')

if Dir.exist?(ios_dir)
  begin
    # Runs fastlane as a real subprocess so its output streams to our STDOUT line-by-line.
    # It signs in with the App Store Connect API key (ASC_* env vars) only; no Apple ID, no 2-step code.
    fastlane_lines = []

    Open3.popen2e({ 'LANG' => 'en_US.UTF-8' }, 'fastlane', 'fetch_all_versions', chdir: ios_dir) do |stdin, stdout_err, wait_thr|
      stdin.close
      stdout_err.each_line do |line|
        line = line.chomp
        next if line.empty?
        fastlane_lines << line
        puts line
      end

      unless wait_thr.value.success?
        raise "fastlane fetch_all_versions exited with status #{wait_thr.value.exitstatus}"
      end
    end

    match = fastlane_lines.join("\n").match(/IOS_VERSIONS=(\{.*\})/)
    ios_versions = JSON.parse(match[1]) if match
    dates = fastlane_lines.join("\n").match(/IOS_DATES=(\{.*\})/)
    ios_dates = JSON.parse(dates[1]) if dates
    log("  ⚠️", "The fetch_all_versions lane printed no IOS_DATES; iOS release dates are left empty (add it to the workspace ios/fastlane/Fastfile)") if ios_versions.any? && ios_dates.empty?
    if ios_versions.any?
      log("  ✅", "#{ios_versions.size} apps found")
    else
      # The lane prints IOS_VERSIONS={} when App Store Connect refuses it (e.g. an expired agreement)
      error_line = fastlane_lines.find { |l| l =~ /error|kurulamadı|\[!\]/i }
      err("Failed to fetch iOS versions: #{error_line&.gsub(/\e\[\d+m/, '')&.strip || 'could not parse output'}")
    end
  rescue => e
    err("Failed to run iOS fastlane: #{e.message}")
  end
else
  log("  ⚠️", "ios folder not found: #{ios_dir}")
end

# ─── Update version.json ─────────────────────────────────────────────────────

log("💾", "Updating version.json files...")

projects.each_key do |project_id|
  android = android_versions[project_id]
  ios     = ios_versions[project_id]
  version_str = android || ios

  unless version_str
    results[:skipped] << project_id
    next
  end

  stored = read_stored(project_id)
  fields = { 'version' => version_str, 'android' => android, 'ios' => ios }
  synced_at = Time.now.strftime('%Y-%m-%dT%H:%M:%S%:z')
  # Google Play gives no release date, so it is the day the sync first saw this version.
  # App Store Connect gives the version's release (or creation) date.
  fields['androidReleasedAt'] = (stored['android'] == android && stored['androidReleasedAt']) || synced_at if android
  fields['iosReleasedAt'] = ios_dates[project_id] || (stored['ios'] == ios ? stored['iosReleasedAt'] : nil) if ios

  if fields.slice('version', 'android', 'ios').all? { |k, v| v.nil? || stored[k] == v }
    save_version(project_id, fields.slice('androidReleasedAt', 'iosReleasedAt').merge('syncedAt' => synced_at))
    results[:skipped] << project_id
    log("  ⏭️",  "#{project_id}: unchanged (Android #{android || '-'}, iOS #{ios || '-'})")
    next
  end

  if save_version(project_id, fields.merge('syncedAt' => synced_at))
    results[:updated] << { id: project_id, old: stored['version'], new: version_str, android: android, ios: ios }
    log("  💾", "#{project_id}: Android #{android || '-'}, iOS #{ios || '-'}")
  else
    results[:failed] << project_id
    err("#{project_id}: folder not found")
  end
end

# ─── Summary ──────────────────────────────────────────────────────────────────

log("📊", "Completed:")
log("  ✅", "Updated:   #{results[:updated].size}")
log("  ⏭️",  "Unchanged: #{results[:skipped].size}")
log("  ❌", "Failed:    #{results[:failed].size}")

puts "SYNC_RESULT=#{JSON.generate(results)}"
