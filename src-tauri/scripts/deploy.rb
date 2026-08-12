#!/usr/bin/env ruby
# encoding: utf-8

require 'fileutils'
require 'json'

# STDOUT buffering'i kapat - realtime log için zorunlu
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
    STDERR.puts "❌ HATA: #{message}"
  end

  def self.run_command(command, description)
    log("⚙️", description)
    success = system(command)
    if success
      log("✅", "#{description} - Tamamlandı")
    else
      log_error("#{description} - Başarısız")
    end
    success
  end

  # ============ STORE LOCALES ============

  # fastlane metadata dizinlerindeki locale klasörlerini okur.
  # iOS: ios/fastlane/metadata/{locale}/
  # Android: android/fastlane/metadata/android/{locale}/
  # Bulunamazsa serconf.dart'tan fallback yapar.
  def self.get_store_locales(project)
    ios_locales     = read_fastlane_locales(:ios)
    android_locales = read_fastlane_locales(:android)

    if ios_locales.any? || android_locales.any?
      ios_locales     = ios_locales.any?     ? ios_locales     : ['tr']
      android_locales = android_locales.any? ? android_locales : ['tr-TR']
      log("🌍", "Mağaza locale'leri (fastlane metadata): iOS=#{ios_locales.join(', ')}  Android=#{android_locales.join(', ')}")
      return { ios: ios_locales, android: android_locales }
    end

    # Fallback: serconf.dart dil bayraklarından tespit et
    log("⚠️", "fastlane metadata klasörü bulunamadı, serconf.dart kullanılıyor")
    get_store_locales_from_serconf(project)
  end

  # ios/fastlane/metadata/ veya android/fastlane/metadata/android/ altındaki
  # locale klasörlerini listeler (gizli dosyalar ve "default" atlanır).
  def self.read_fastlane_locales(platform)
    metadata_path = case platform
                    when :ios     then File.join(ios_path, 'fastlane', 'metadata')
                    when :android then File.join(android_path, 'fastlane', 'metadata', 'android')
                    end

    return [] unless Dir.exist?(metadata_path)

    Dir.entries(metadata_path)
       .reject { |e| e.start_with?('.') || e == 'default' || e == 'review_information' }
       .select { |e| Dir.exist?(File.join(metadata_path, e)) }
       .sort
  end

  # serconf.dart dil bayraklarından App Store / Google Play locale kodlarına eşleme
  LANGUAGE_LOCALE_MAP = {
    'ENGLISH' => { ios: 'en-US',  android: 'en-US'  },
    'RUSSIAN' => { ios: 'ru',     android: 'ru-RU'  },
    'FRENCH'  => { ios: 'fr-FR',  android: 'fr-FR'  },
    'ITALIAN' => { ios: 'it',     android: 'it-IT'  },
    'ARABIC'  => { ios: 'ar-SA',  android: 'ar'     },
    'SPANISH' => { ios: 'es-ES',  android: 'es-ES'  },
    'KAZAKH'  => { ios: 'kk',     android: 'kk'     },
  }.freeze

  def self.get_store_locales_from_serconf(project)
    ios_locales     = ['tr']
    android_locales = ['tr-TR']

    serconf_path = File.join(flutter_root, PROJECTS_PATH, project.to_s, 'serconf.dart')
    unless File.exist?(serconf_path)
      log("⚠️", "serconf.dart bulunamadı, varsayılan locale kullanılıyor: tr + en-US")
      return { ios: ios_locales + ['en-US'], android: android_locales + ['en-US'] }
    end

    content = File.read(serconf_path)

    LANGUAGE_LOCALE_MAP.each do |flag, locales|
      if content.match?(/const\s+#{flag}\s*=\s*true\s*;/)
        ios_locales     << locales[:ios]
        android_locales << locales[:android]
      end
    end

    log("🌍", "Store locale'leri (serconf.dart): iOS=#{ios_locales.join(', ')}  Android=#{android_locales.join(', ')}")
    { ios: ios_locales, android: android_locales }
  end

  def self.set_store_locale_envs
    # TypeScript tarafından mağazadan fetch edildiyse tekrar tespit etme
    if ENV['STORE_LOCALES_IOS'].to_s.strip.length > 0 &&
       ENV['STORE_LOCALES_ANDROID'].to_s.strip.length > 0
      log("🌍", "Store locale'leri (mağazadan alındı): iOS=#{ENV['STORE_LOCALES_IOS']}  Android=#{ENV['STORE_LOCALES_ANDROID']}")
      return
    end

    project = get_current_project
    locales = get_store_locales(project)
    ENV['STORE_LOCALES_IOS']     = locales[:ios].join(',')
    ENV['STORE_LOCALES_ANDROID'] = locales[:android].join(',')
  end

  # ============ VERSION ============
  MIN_VERSION = "19.0.0"  # Minimum versiyon - tüm projeler bu versiyondan başlar

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

  # Projenin kendi version.json dosyasından versiyonu oku
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

  # Versiyon karşılaştırma: v1 < v2 ise true döner
  def self.version_less_than?(v1, v2)
    return true if v1.nil?

    # Build number'ı ayır (19.0.0+190000 -> 19.0.0)
    v1_part = v1.split('+')[0]
    v2_part = v2.split('+')[0]

    v1_nums = v1_part.split('.').map(&:to_i)
    v2_nums = v2_part.split('.').map(&:to_i)

    # Major karşılaştır
    return true if v1_nums[0] < v2_nums[0]
    return false if v1_nums[0] > v2_nums[0]

    # Minor karşılaştır
    return true if v1_nums[1] < v2_nums[1]
    return false if v1_nums[1] > v2_nums[1]

    # Patch karşılaştır
    return v1_nums[2] < v2_nums[2]
  end

  def self.save_project_version(project, version)
    version_file = File.join(flutter_root, PROJECTS_PATH, project, 'version.json')
    FileUtils.mkdir_p(File.dirname(version_file))
    File.write(version_file, JSON.pretty_generate({ "version" => version }))
    log("💾", "Proje versiyonu kaydedildi: #{project} -> #{version}")
  end

  def self.update_version(new_version)
    log("🔢", "Versiyon güncelleniyor: #{new_version}")

    pubspec = File.join(flutter_root, PUBSPEC_PATH)
    content = File.read(pubspec)
    old_version = get_current_version
    log("📋", "Mevcut versiyon: #{old_version}")

    content.gsub!(/version:\s*[^\s]+/, "version: #{new_version}")
    File.write(pubspec, content)
    log("✅", "pubspec.yaml güncellendi")

    marketing_version = new_version.split('+')[0]
    build_number = new_version.split('+')[1] || '1'

    info_plist = File.join(flutter_root, IOS_CONFIG_PATH, 'Runner', 'Info.plist')
    if File.exist?(info_plist)
      system("/usr/libexec/PlistBuddy -c \"Set :CFBundleShortVersionString #{marketing_version}\" #{info_plist}")
      system("/usr/libexec/PlistBuddy -c \"Set :CFBundleVersion #{build_number}\" #{info_plist}")
      log("✅", "Info.plist güncellendi")
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
        log("✅", "Xcode project güncellendi")
      end
    rescue LoadError
      log("⚠️", "xcodeproj gem bulunamadı")
    end

    current_project = get_current_project
    if current_project
      save_project_version(current_project, new_version)
    end

    new_version
  end

  # ============ NATIVE SPLASH ============

  # Proje için splash konfigürasyonunu oku (splash.json dosyasından)
  def self.get_splash_config(project)
    return nil unless project

    project_path = File.join(flutter_root, PROJECTS_PATH, project)
    splash_config_file = File.join(project_path, 'splash.json')

    # Varsayılan değerler
    default_config = {
      "color" => "#FFFFFF",
      "image" => nil
    }

    # splash.json varsa oku
    if File.exist?(splash_config_file)
      begin
        content = File.read(splash_config_file)
        config = JSON.parse(content)
        default_config.merge(config)
      rescue
        log("⚠️", "splash.json okunamadı, varsayılan değerler kullanılacak")
        default_config
      end
    else
      # splash.json yoksa, Launch klasöründe görsel ara
      launch_path = File.join(project_path, 'Launch')
      if File.exist?(File.join(launch_path, 'splash.png'))
        default_config["image"] = "lib/conf/sermobplus-projects/#{project}/Launch/splash.png"
      elsif File.exist?(File.join(launch_path, '2x.png'))
        default_config["image"] = "lib/conf/sermobplus-projects/#{project}/Launch/2x.png"
      end
      default_config
    end
  end

  # Flutter native splash yaml dosyasını oluştur
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
    log("📝", "flutter_native_splash.yaml oluşturuldu")
    true
  end

  # Native splash'ı oluştur
  def self.create_native_splash
    current_project = get_current_project

    unless current_project
      # Generic (tek proje) workspace: sermobileboss'un proje-bazlı splash.json'u
      # yok, bunun yerine workspace kökündeki flutter_native_splash.yaml (varsa)
      # kullanılır.
      yaml_path = File.join(flutter_root, 'flutter_native_splash.yaml')
      return true unless File.exist?(yaml_path) # Yapılandırma yoksa sessizce atla

      log("🎨", "Native Splash oluşturuluyor (flutter_native_splash.yaml)")
      return run_command("dart run flutter_native_splash:create", "Native Splash oluştur")
    end

    log("🎨", "Native Splash oluşturuluyor: #{current_project}")

    # Yaml dosyasını oluştur
    unless generate_splash_yaml(current_project)
      log("⚠️", "Splash konfigürasyonu bulunamadı, atlanıyor")
      return true # Hata olarak sayma, devam et
    end

    # Flutter native splash komutunu çalıştır
    success = run_command("dart run flutter_native_splash:create", "Native Splash oluştur")

    # Geçici yaml dosyasını temizle (opsiyonel)
    # yaml_path = File.join(flutter_root, 'flutter_native_splash.yaml')
    # File.delete(yaml_path) if File.exist?(yaml_path)

    success
  end

  def self.auto_increment_version
    # Önce aktif projeyi bul
    current_project = get_current_project

    if current_project
      # Projenin kendi version.json'undan versiyonu oku
      project_version = get_project_version(current_project)

      # Eğer proje versiyonu yoksa veya MIN_VERSION'dan küçükse, MIN_VERSION kullan
      if project_version.nil? || version_less_than?(project_version, MIN_VERSION)
        log("⚠️", "Proje versiyonu (#{project_version || 'yok'}) #{MIN_VERSION}'dan küçük, #{MIN_VERSION}'dan başlatılıyor")
        current = "#{MIN_VERSION}+#{19 * 10000000}"  # 19.0.0+190000000
      else
        current = project_version
        log("📌", "Proje versiyonu: #{current}")
      end
    else
      # Generic (tek proje) workspace: kaynak her zaman pubspec.yaml'dır.
      current = get_current_version || "#{MIN_VERSION}+#{19 * 10000000}"
      log("📌", "Mevcut versiyon (pubspec.yaml): #{current}")
    end

    # Version parçala: 19.0.0+190000000
    parts = current.split('+')
    version_part = parts[0]  # 19.0.0

    # Versiyon numarasını parçala: major.minor.patch
    version_nums = version_part.split('.')
    major = version_nums[0].to_i  # 19
    minor = version_nums[1].to_i  # 0
    patch = version_nums[2].to_i  # 0

    # Patch versiyonu artır
    patch += 1

    # Otomatik taşma: patch > 9 ise minor artır
    if patch > 9
      patch = 0
      minor += 1
    end

    # Otomatik taşma: minor > 9 ise major artır
    if minor > 9
      minor = 0
      major += 1
    end

    new_version_part = "#{major}.#{minor}.#{patch}"

    # Build numarasını hesapla (9 hane): major * 10000000 + minor * 100000 + patch * 1000
    new_build = major * 10000000 + minor * 100000 + patch * 1000
    new_version = "#{new_version_part}+#{new_build}"

    log("🔼", "Versiyon otomatik artırılıyor: #{current} → #{new_version}")
    update_version(new_version)
    new_version
  end

  def self.deploy_ios
    log("🍎", "iOS Deploy başlatılıyor...")
    log("📍", "Proje: #{flutter_root}")

    Dir.chdir(flutter_root) do
      # Otomatik versiyon artır
      auto_increment_version

      # Store locale'lerini tespit et ve ENV'e yaz
      set_store_locale_envs

      return false unless run_command("flutter clean && flutter pub get", "Flutter hazırlık")

      # Native splash oluştur (Android 12+ desteği dahil)
      create_native_splash

      return false unless run_command("cd #{ios_path} && pod install", "CocoaPods")
      return false unless run_command("cd #{ios_path} && fastlane release", "App Store deploy")
    end

    log("✅", "iOS Deploy başarıyla tamamlandı!")
    true
  end

  def self.deploy_android
    log("🤖", "Android Deploy başlatılıyor...")
    log("📍", "Proje: #{flutter_root}")

    Dir.chdir(flutter_root) do
      # Otomatik versiyon artır
      auto_increment_version

      # Store locale'lerini tespit et ve ENV'e yaz
      set_store_locale_envs

      return false unless run_command("flutter clean && flutter pub get", "Flutter hazırlık")

      # Native splash oluştur (Android 12+ desteği dahil)
      create_native_splash

      return false unless run_command("flutter build appbundle", "App Bundle oluştur")
      return false unless run_command("cd #{android_path} && fastlane release", "Google Play deploy")
    end

    log("✅", "Android Deploy başarıyla tamamlandı!")
    true
  end

  def self.deploy_huawei
    log("📱", "Huawei Deploy başlatılıyor...")
    log("⚠️", "Huawei AppGallery desteği yakında eklenecek")
    # TODO: Huawei AppGallery entegrasyonu
    true
  end

  def self.deploy_all
    log("🚀", "Tüm platformlara deploy başlatılıyor...")
    log("📍", "Proje: #{flutter_root}")

    Dir.chdir(flutter_root) do
      # Otomatik versiyon artır (bir kez)
      auto_increment_version

      # Store locale'lerini tespit et ve ENV'e yaz
      set_store_locale_envs

      return false unless run_command("flutter clean && flutter pub get", "Flutter hazırlık")

      # Native splash oluştur (Android 12+ desteği dahil)
      create_native_splash

      return false unless run_command("cd #{ios_path} && pod install", "CocoaPods")
      return false unless run_command("flutter build appbundle", "App Bundle oluştur")

      # Android deploy
      log("🤖", "Google Play'e yükleniyor...")
      android_success = system("cd #{android_path} && fastlane release")

      # iOS deploy
      log("🍎", "App Store'a yükleniyor...")
      ios_success = system("cd #{ios_path} && fastlane release")

      if android_success && ios_success
        log("✅", "Tüm platformlara deploy başarıyla tamamlandı!")
        return true
      else
        log_error("Bazı platformlarda hata oluştu")
        return false
      end
    end
  end

  def self.print_usage
    puts <<~USAGE
      Kullanım: ruby deploy.rb <platform> [path]

      Platformlar:
        ios       App Store'a deploy
        android   Google Play'e deploy
        huawei    Huawei AppGallery'e deploy
        all       Tüm platformlara deploy

      Örnekler:
        ruby deploy.rb ios /path/to/project
        ruby deploy.rb android /path/to/project
        ruby deploy.rb all /path/to/project
    USAGE
  end
end

# CLI Entry Point
if __FILE__ == $0
  platform = ARGV[0]
  project_path = ARGV[1] # Opsiyonel - proje yolu

  # Proje yolu verilmişse kullan, yoksa mevcut dizini kullan
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
    Deployer.log_error("Bilinmeyen platform: #{platform}")
    Deployer.print_usage
    exit 1
  end
end
