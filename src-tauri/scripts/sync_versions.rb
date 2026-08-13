#!/usr/bin/env ruby
# encoding: utf-8
# Fetches store versions for all projects and updates their version.json files.
# Usage: ruby sync_versions.rb /path/to/sermobilepro

require 'json'
require 'fileutils'
require 'open3'

STDOUT.sync = true
STDERR.sync = true

workspace = ARGV[0] || Dir.pwd

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

def build_number_from_version(version_str)
  parts = version_str.split('.').map(&:to_i)
  parts[0].to_i * 10_000_000 + parts[1].to_i * 100_000 + parts[2].to_i * 1_000
end

def save_version(project_id, version_str)
  dir  = File.join(PROJECTS_PATH, project_id)
  return false unless File.directory?(dir)

  file = File.join(dir, 'version.json')
  File.write(file, JSON.pretty_generate({ 'version' => version_str }))
  true
end

def read_current_version(project_id)
  file = File.join(PROJECTS_PATH, project_id, 'version.json')
  return nil unless File.exist?(file)
  JSON.parse(File.read(file))['version'] rescue nil
end

# ─── Project list ───────────────────────────────────────────────────────────

unless File.exist?(PROJECTS_FILE)
  err("sermobileboss_projects.json not found: #{PROJECTS_FILE}")
  exit 1
end

projects = JSON.parse(File.read(PROJECTS_FILE))
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

# ─── iOS — App Store Connect (bundle exec fastlane) ──────────────────────────

log("🍎", "iOS — fetching App Store Connect versions...")

ios_versions = {}
ios_dir = File.join(workspace, 'ios')

if Dir.exist?(ios_dir)
  begin
    # Runs fastlane as a real subprocess (not backticks) so its output streams to our
    # STDOUT line-by-line as it happens, and our own STDIN (piped from the Rust side) is
    # forwarded live into fastlane's STDIN. This lets the 2-step verification code the
    # user submits from the UI reach fastlane's Apple sign-in prompt while it's waiting.
    fastlane_lines = []

    Open3.popen2e({ 'LANG' => 'en_US.UTF-8' }, 'bundle', 'exec', 'fastlane', 'fetch_all_versions', chdir: ios_dir) do |stdin, stdout_err, wait_thr|
      forwarder = Thread.new do
        loop { stdin.write(STDIN.readpartial(4096)) }
      rescue IOError, Errno::EPIPE, EOFError
        # child stdin closed or nothing left to read — stop forwarding
      end

      stdout_err.each_line do |line|
        line = line.chomp
        next if line.empty?
        fastlane_lines << line
        puts line
      end

      forwarder.kill
      begin; stdin.close; rescue; end

      unless wait_thr.value.success?
        raise "fastlane fetch_all_versions exited with status #{wait_thr.value.exitstatus}"
      end
    end

    match = fastlane_lines.join("\n").match(/IOS_VERSIONS=(\{.+\})/)
    if match
      ios_versions = JSON.parse(match[1])
      log("  ✅", "#{ios_versions.size} apps found")
    else
      # Summarize the error output
      error_line = fastlane_lines.select { |l| l.include?('Error') || l.include?('error') || l.include?('[!]') }.first
      log("  ⚠️", "Failed to fetch iOS versions: #{error_line&.strip || 'could not parse output'}")
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
  # Android takes priority (version_code is accurate), fallback to iOS
  version_str = android_versions[project_id] || ios_versions[project_id]

  unless version_str
    results[:skipped] << project_id
    next
  end

  current = read_current_version(project_id)

  if current == version_str
    results[:skipped] << project_id
    log("  ⏭️",  "#{project_id}: unchanged (#{version_str})")
    next
  end

  if save_version(project_id, version_str)
    results[:updated] << { id: project_id, old: current, new: version_str }
    log("  💾", "#{project_id}: #{current || '?'} → #{version_str}")
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
