#!/usr/bin/env ruby
# encoding: utf-8

require 'fileutils'
require 'json'

# Disable STDOUT buffering - required for realtime logging
STDOUT.sync = true
STDERR.sync = true

class Deployer
  PUBSPEC_PATH = 'pubspec.yaml'
  IOS_CONFIG_PATH = 'ios'
  PROJECTS_PATH = 'lib/conf/sermobplus-projects'
  CURRENT_PROJECT_FILE = 'sermobileboss.txt'

  class << self
    attr_accessor :project_root
  end

  def self.flutter_root
    @project_root || Dir.pwd
  end

  def self.android_path
    File.join(flutter_root, 'android')
  end

  def self.ios_path
    File.join(flutter_root, 'ios')
  end

  def self.log(emoji, message)
    puts "#{emoji} #{message}"
  end

  def self.log_error(message)
    STDERR.puts "❌ ERROR: #{message}"
  end

  def self.run_command(command, description)
    log("⚙️", description)
    success = system(command)
    if success
      log("✅", "#{description} - Completed")
    else
      log_error("#{description} - Failed")
    end
    success
  end

  # ============ STORE LOCALES ============

  # The Fastfile writes release notes only for the app's store locales (STORE_LOCALES_*,
  # fetched from the store by the app), but deliver/supply upload every locale folder under
  # fastlane/metadata and create that language in the store if the app doesn't have it.
  # In a sermobileboss workspace those folders are shared by every app, so folders left over
  # from another app's deploy would add its languages to this one — remove their release notes.
  def self.prune_stale_release_notes
    log("🌍", "Store locales: iOS=#{ENV['STORE_LOCALES_IOS']}  Android=#{ENV['STORE_LOCALES_ANDROID']}")
    prune_release_notes(File.join(ios_path, 'fastlane', 'metadata'), ENV['STORE_LOCALES_IOS'], 'release_notes.txt')
    prune_release_notes(File.join(android_path, 'fastlane', 'metadata', 'android'), ENV['STORE_LOCALES_ANDROID'], 'changelogs')
  end

  def self.prune_release_notes(metadata_path, store_locales, notes_entry)
    locales = store_locales.to_s.split(',').map(&:strip).reject(&:empty?)
    return if locales.empty? || !Dir.exist?(metadata_path)

    Dir.children(metadata_path).each do |locale|
      next if locales.include?(locale) || ['default', 'review_information'].include?(locale)

      notes = File.join(metadata_path, locale, notes_entry)
      next unless File.exist?(notes)

      FileUtils.rm_rf(notes)
      dir = File.dirname(notes)
      Dir.rmdir(dir) if Dir.empty?(dir)
      log("🧹", "Removed #{locale} release notes (not a store language of this app)")
    end
  end

  # ============ VERSION ============
  DEFAULT_VERSION = "1.0.0+1"  # Used only when no version can be found anywhere

  def self.get_current_project
    project_file = File.join(flutter_root, CURRENT_PROJECT_FILE)
    return nil unless File.exist?(project_file)
    File.read(project_file).strip
  end

  def self.get_current_version
    pubspec = File.join(flutter_root, PUBSPEC_PATH)
    return nil unless File.exist?(pubspec)
    content = File.read(pubspec)
    match = content.match(/version:\s*([^\s]+)/)
    match ? match[1] : nil
  end

  # Read the version from the project's own version.json file
  def self.get_project_version(project)
    return nil unless project
    version_file = File.join(flutter_root, PROJECTS_PATH, project, 'version.json')
    return nil unless File.exist?(version_file)

    begin
      content = File.read(version_file)
      data = JSON.parse(content)
      data["version"]
    rescue
      nil
    end
  end

  def self.save_project_version(project, version)
    version_file = File.join(flutter_root, PROJECTS_PATH, project, 'version.json')
    FileUtils.mkdir_p(File.dirname(version_file))
    File.write(version_file, JSON.pretty_generate({ "version" => version }))
    log("💾", "Project version saved: #{project} -> #{version}")
  end

  def self.update_version(new_version)
    log("🔢", "Updating version: #{new_version}")

    pubspec = File.join(flutter_root, PUBSPEC_PATH)
    content = File.read(pubspec)
    old_version = get_current_version
    log("📋", "Current version: #{old_version}")

    content.gsub!(/version:\s*[^\s]+/, "version: #{new_version}")
    File.write(pubspec, content)
    log("✅", "pubspec.yaml updated")

    marketing_version = new_version.split('+')[0]
    build_number = new_version.split('+')[1] || '1'

    info_plist = File.join(flutter_root, IOS_CONFIG_PATH, 'Runner', 'Info.plist')
    if File.exist?(info_plist)
      system("/usr/libexec/PlistBuddy -c \"Set :CFBundleShortVersionString #{marketing_version}\" #{info_plist}")
      system("/usr/libexec/PlistBuddy -c \"Set :CFBundleVersion #{build_number}\" #{info_plist}")
      log("✅", "Info.plist updated")
    end

    begin
      require 'xcodeproj'
      xcodeproj_path = File.join(flutter_root, IOS_CONFIG_PATH, 'Runner.xcodeproj')
      if File.exist?(xcodeproj_path)
        project = Xcodeproj::Project.open(xcodeproj_path)
        project.targets.each do |target|
          target.build_configurations.each do |config|
            config.build_settings['MARKETING_VERSION'] = marketing_version
            config.build_settings['CURRENT_PROJECT_VERSION'] = build_number
          end
        end
        project.save
        log("✅", "Xcode project updated")
      end
    rescue LoadError
      log("⚠️", "xcodeproj gem not found")
    end

    current_project = get_current_project
    if current_project
      save_project_version(current_project, new_version)
    end

    new_version
  end

  # ============ NATIVE SPLASH ============

  # Read the splash configuration for the project (from splash.json)
  def self.get_splash_config(project)
    return nil unless project

    project_path = File.join(flutter_root, PROJECTS_PATH, project)
    splash_config_file = File.join(project_path, 'splash.json')

    # Default values
    default_config = {
      "color" => "#FFFFFF",
      "image" => nil
    }

    # Read splash.json if present
    if File.exist?(splash_config_file)
      begin
        content = File.read(splash_config_file)
        config = JSON.parse(content)
        default_config.merge(config)
      rescue
        log("⚠️", "Failed to read splash.json, using default values")
        default_config
      end
    else
      # No splash.json, look for an image in the Launch folder
      launch_path = File.join(project_path, 'Launch')
      if File.exist?(File.join(launch_path, 'splash.png'))
        default_config["image"] = "lib/conf/sermobplus-projects/#{project}/Launch/splash.png"
      elsif File.exist?(File.join(launch_path, '2x.png'))
        default_config["image"] = "lib/conf/sermobplus-projects/#{project}/Launch/2x.png"
      end
      default_config
    end
  end

  # Generate the Flutter native splash yaml file
  def self.generate_splash_yaml(project)
    config = get_splash_config(project)
    return false unless config && config["image"]

    color = config["color"] || "#FFFFFF"
    image = config["image"]

    yaml_content = <<~YAML
      flutter_native_splash:
        color: "#{color}"
        image: #{image}
        android: true
        ios: true

        android_12:
          image: #{image}
          icon_background_color: "#{color}"
    YAML

    yaml_path = File.join(flutter_root, 'flutter_native_splash.yaml')
    File.write(yaml_path, yaml_content)
    log("📝", "flutter_native_splash.yaml created")
    true
  end

  # Generate the native splash
  def self.create_native_splash
    current_project = get_current_project

    unless current_project
      # Generic (single-project) workspace: there's no per-project splash.json
      # like in sermobileboss — instead, the workspace root's
      # flutter_native_splash.yaml (if present) is used.
      yaml_path = File.join(flutter_root, 'flutter_native_splash.yaml')
      return true unless File.exist?(yaml_path) # Silently skip if no config

      log("🎨", "Generating Native Splash (flutter_native_splash.yaml)")
      return run_command("dart run flutter_native_splash:create", "Generate Native Splash")
    end

    log("🎨", "Generating Native Splash: #{current_project}")

    # Generate the yaml file
    unless generate_splash_yaml(current_project)
      log("⚠️", "Splash configuration not found, skipping")
      return true # Don't treat as an error, continue
    end

    # Run the flutter native splash command
    success = run_command("dart run flutter_native_splash:create", "Generate Native Splash")

    # Clean up the temporary yaml file (optional)
    # yaml_path = File.join(flutter_root, 'flutter_native_splash.yaml')
    # File.delete(yaml_path) if File.exist?(yaml_path)

    success
  end

  def self.auto_increment_version
    # First, find the active project
    current_project = get_current_project

    if current_project
      # Read the version from the project's own version.json, falling back to
      # pubspec.yaml, then to a hardcoded default if neither exists yet.
      current = get_project_version(current_project) || get_current_version || DEFAULT_VERSION
      log("📌", "Project version: #{current}")
    else
      # Generic (single-project) workspace: the source of truth is always pubspec.yaml.
      current = get_current_version || DEFAULT_VERSION
      log("📌", "Current version (pubspec.yaml): #{current}")
    end

    # Split the version: 19.0.0+190000000
    parts = current.split('+')
    version_part = parts[0]  # 19.0.0

    # Split the version number: major.minor.patch
    version_nums = version_part.split('.')
    major = version_nums[0].to_i  # 19
    minor = version_nums[1].to_i  # 0
    patch = version_nums[2].to_i  # 0

    # Bump the patch version
    patch += 1

    # Auto-carry: if patch > 9, bump minor
    if patch > 9
      patch = 0
      minor += 1
    end

    # Auto-carry: if minor > 9, bump major
    if minor > 9
      minor = 0
      major += 1
    end

    new_version_part = "#{major}.#{minor}.#{patch}"

    # Compute the build number (9 digits): major * 10000000 + minor * 100000 + patch * 1000
    new_build = major * 10000000 + minor * 100000 + patch * 1000
    new_version = "#{new_version_part}+#{new_build}"

    log("🔼", "Auto-incrementing version: #{current} → #{new_version}")
    update_version(new_version)
    new_version
  end

  # Bumps the version unless the app explicitly opted out (BUMP_VERSION=false),
  # e.g. when the user unchecked "increment version" in the deploy dialog.
  def self.maybe_bump_version
    if ENV['BUMP_VERSION'] == 'false'
      log("📌", "Version bump skipped, deploying with the current version")
    else
      auto_increment_version
    end
  end

  # Whether the user picked the test track (TestFlight / Internal Testing) in the deploy dialog.
  def self.test_track?
    ENV['RELEASE_TRACK'] == 'test'
  end

  # Fastlane lane to run for a store: "beta" (TestFlight / Internal Testing) or "release" (production).
  def self.fastlane_lane
    test_track? ? 'beta' : 'release'
  end

  def self.deploy_ios
    log("🍎", "Starting iOS Deploy...")
    log("📍", "Project: #{flutter_root}")
    log("🎯", "Target: #{test_track? ? 'TestFlight' : 'App Store'}")

    Dir.chdir(flutter_root) do
      # Bump version unless the user opted out
      maybe_bump_version

      # Drop release notes for locales this app doesn't have in the store
      prune_stale_release_notes

      return false unless run_command("flutter clean && flutter pub get", "Flutter setup")

      # Generate native splash (including Android 12+ support)
      create_native_splash

      return false unless run_command("cd #{ios_path} && pod install", "CocoaPods")
      return false unless run_command("cd #{ios_path} && fastlane #{fastlane_lane}", test_track? ? "TestFlight deploy" : "App Store deploy")
    end

    log("✅", "iOS Deploy completed successfully!")
    true
  end

  def self.deploy_android
    log("🤖", "Starting Android Deploy...")
    log("📍", "Project: #{flutter_root}")
    log("🎯", "Target: #{test_track? ? 'Internal Testing' : 'Google Play'}")

    Dir.chdir(flutter_root) do
      # Bump version unless the user opted out
      maybe_bump_version

      # Drop release notes for locales this app doesn't have in the store
      prune_stale_release_notes

      return false unless run_command("flutter clean && flutter pub get", "Flutter setup")

      # Generate native splash (including Android 12+ support)
      create_native_splash

      return false unless run_command("flutter build appbundle", "Build App Bundle")
      return false unless run_command("cd #{android_path} && fastlane #{fastlane_lane}", test_track? ? "Internal Testing deploy" : "Google Play deploy")
    end

    log("✅", "Android Deploy completed successfully!")
    true
  end

  def self.deploy_huawei
    log("📱", "Starting Huawei Deploy...")
    log("⚠️", "Huawei AppGallery support coming soon")
    # TODO: Huawei AppGallery integration
    true
  end

  def self.deploy_all
    log("🚀", "Starting deploy to all platforms...")
    log("📍", "Project: #{flutter_root}")
    log("🎯", "Target: #{test_track? ? 'TestFlight / Internal Testing' : 'App Store / Google Play'}")

    Dir.chdir(flutter_root) do
      # Bump version (once) unless the user opted out
      maybe_bump_version

      # Drop release notes for locales this app doesn't have in the store
      prune_stale_release_notes

      return false unless run_command("flutter clean && flutter pub get", "Flutter setup")

      # Generate native splash (including Android 12+ support)
      create_native_splash

      return false unless run_command("cd #{ios_path} && pod install", "CocoaPods")
      return false unless run_command("flutter build appbundle", "Build App Bundle")

      # Android deploy
      log("🤖", "Uploading to #{test_track? ? 'Internal Testing' : 'Google Play'}...")
      android_success = system("cd #{android_path} && fastlane #{fastlane_lane}")

      # iOS deploy
      log("🍎", "Uploading to #{test_track? ? 'TestFlight' : 'App Store'}...")
      ios_success = system("cd #{ios_path} && fastlane #{fastlane_lane}")

      if android_success && ios_success
        log("✅", "Deploy to all platforms completed successfully!")
        return true
      else
        log_error("An error occurred on some platforms")
        return false
      end
    end
  end

  def self.print_usage
    puts <<~USAGE
      Usage: ruby deploy.rb <platform> [path]

      Platforms:
        ios       Deploy to App Store
        android   Deploy to Google Play
        huawei    Deploy to Huawei AppGallery
        all       Deploy to all platforms

      Examples:
        ruby deploy.rb ios /path/to/project
        ruby deploy.rb android /path/to/project
        ruby deploy.rb all /path/to/project
    USAGE
  end
end

# CLI Entry Point
if __FILE__ == $0
  platform = ARGV[0]
  project_path = ARGV[1] # Optional - project path

  # Use the given project path, otherwise fall back to the current directory
  if project_path && !project_path.empty?
    Deployer.project_root = File.expand_path(project_path)
  else
    Deployer.project_root = Dir.pwd
  end

  case platform
  when 'ios'
    success = Deployer.deploy_ios
    exit(success ? 0 : 1)

  when 'android'
    success = Deployer.deploy_android
    exit(success ? 0 : 1)

  when 'huawei'
    success = Deployer.deploy_huawei
    exit(success ? 0 : 1)

  when 'all'
    success = Deployer.deploy_all
    exit(success ? 0 : 1)

  when nil, 'help', '-h', '--help'
    Deployer.print_usage
    exit 0

  else
    Deployer.log_error("Unknown platform: #{platform}")
    Deployer.print_usage
    exit 1
  end
end
