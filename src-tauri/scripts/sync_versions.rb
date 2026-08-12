#!/usr/bin/env ruby
# encoding: utf-8
# Tüm projelerin mağaza versiyonlarını çekip version.json dosyalarını günceller.
# Kullanım: ruby sync_versions.rb /path/to/sermobilepro

require 'json'
require 'fileutils'

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

# ─── Yardımcılar ───────────────────────────────────────────────────────────────

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

# ─── Proje listesi ─────────────────────────────────────────────────────────────

unless File.exist?(PROJECTS_FILE)
  err("sermobileboss_projects.json bulunamadı: #{PROJECTS_FILE}")
  exit 1
end

projects = JSON.parse(File.read(PROJECTS_FILE))
log("📋", "#{projects.size} proje bulundu")

results = { updated: [], skipped: [], failed: [] }

# ─── Android — Google Play API ─────────────────────────────────────────────────

log("🤖", "Android — Google Play versiyonları çekiliyor...")

android_versions = {}
begin
  raise "Bundle ID prefix ayarlanmamış: #{CONFIG_FILE} (Workspace Ayarları'ndan doldurun)" unless BUNDLE_ID_PREFIX

  require 'google/apis/androidpublisher_v3'
  require 'googleauth'

  json_key = Dir[File.join(ANDROID_DIR, 'api-*.json')].first
  raise "Google Play API key bulunamadı: #{ANDROID_DIR}/api-*.json" unless json_key

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

      # Production track'ten en güncel completed release'i bul
      production = tracks.tracks&.find { |t| t.track == 'production' }
      release    = production&.releases&.find { |r| r.status == 'completed' } ||
                   production&.releases&.first

      if release && release.name && release.version_codes&.any?
        raw_name     = release.name.to_s.strip
        version_code = release.version_codes.first.to_i

        # "54700000 (5.4.7)" veya "1 (1.0.0)" gibi formatlardan X.Y.Z'yi çıkar
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
        log("  ⚠️", "#{project_id}: production release bulunamadı")
      end

      begin; service.delete_edit(package, edit.id); rescue; end
    rescue => e
      log("  ⚠️", "#{project_id}: #{e.message.lines.first.strip}")
    end
  end
rescue => e
  err("Google Play API başlatılamadı: #{e.message}")
end

# ─── iOS — App Store Connect (bundle exec fastlane) ───────────────────────────

log("🍎", "iOS — App Store Connect versiyonları çekiliyor...")

ios_versions = {}
ios_dir = File.join(workspace, 'ios')

if Dir.exist?(ios_dir)
  begin
    output = `cd #{ios_dir} && bundle exec fastlane fetch_all_versions 2>&1`
    match  = output.match(/IOS_VERSIONS=(\{.+\})/)
    if match
      ios_versions = JSON.parse(match[1])
      log("  ✅", "#{ios_versions.size} uygulama bulundu")
    else
      # Hata çıktısını özetle
      error_line = output.lines.select { |l| l.include?('Error') || l.include?('error') || l.include?('[!]') }.first
      log("  ⚠️", "iOS versiyon çekme başarısız: #{error_line&.strip || 'çıktı parse edilemedi'}")
    end
  rescue => e
    err("iOS fastlane çalıştırılamadı: #{e.message}")
  end
else
  log("  ⚠️", "ios klasörü bulunamadı: #{ios_dir}")
end

# ─── version.json güncelle ─────────────────────────────────────────────────────

log("💾", "version.json dosyaları güncelleniyor...")

projects.each_key do |project_id|
  # Android öncelikli (version_code doğru gelir), fallback iOS
  version_str = android_versions[project_id] || ios_versions[project_id]

  unless version_str
    results[:skipped] << project_id
    next
  end

  current = read_current_version(project_id)

  if current == version_str
    results[:skipped] << project_id
    log("  ⏭️",  "#{project_id}: değişmedi (#{version_str})")
    next
  end

  if save_version(project_id, version_str)
    results[:updated] << { id: project_id, old: current, new: version_str }
    log("  💾", "#{project_id}: #{current || '?'} → #{version_str}")
  else
    results[:failed] << project_id
    err("#{project_id}: klasör bulunamadı")
  end
end

# ─── Özet ──────────────────────────────────────────────────────────────────────

log("📊", "Tamamlandı:")
log("  ✅", "Güncellendi: #{results[:updated].size}")
log("  ⏭️",  "Değişmedi:   #{results[:skipped].size}")
log("  ❌", "Başarısız:   #{results[:failed].size}")

puts "SYNC_RESULT=#{JSON.generate(results)}"
