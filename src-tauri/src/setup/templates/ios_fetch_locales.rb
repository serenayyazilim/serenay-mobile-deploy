  desc "Prints the app's App Store languages as FASTLANE_LOCALES=..."
  lane :fetch_locales do
    sermobile_api_key
    UI.user_error!("App Store Connect API key is not configured") unless Spaceship::ConnectAPI.token

    app = Spaceship::ConnectAPI::App.find(CredentialsManager::AppfileConfig.try_fetch_value(:app_identifier))
    if app.nil?
      puts "FASTLANE_APP_NOT_FOUND"
      next
    end

    version = app.get_app_store_versions(filter: { platform: "IOS" }).first
    UI.user_error!("The app has no App Store version yet") unless version

    puts "FASTLANE_LOCALES=#{version.get_app_store_version_localizations.map(&:locale).uniq.sort.join(',')}"
  end
