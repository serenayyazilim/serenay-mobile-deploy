  desc "Prints the app's Google Play languages as FASTLANE_LOCALES=..."
  lane :fetch_locales do
    require "google/apis/androidpublisher_v3"
    require "googleauth"

    package_name = CredentialsManager::AppfileConfig.try_fetch_value(:package_name)
    json_key = CredentialsManager::AppfileConfig.try_fetch_value(:json_key_file)
    UI.user_error!("Google Play service account key not found: #{json_key}") unless json_key && File.exist?(json_key)

    service = Google::Apis::AndroidpublisherV3::AndroidPublisherService.new
    service.authorization = Google::Auth::ServiceAccountCredentials.make_creds(
      json_key_io: File.open(json_key),
      scope: "https://www.googleapis.com/auth/androidpublisher"
    )

    begin
      edit = service.insert_edit(package_name)
    rescue Google::Apis::ClientError => e
      raise unless e.status_code == 404
      puts "FASTLANE_APP_NOT_FOUND"
      next
    end

    locales = (service.list_edit_listings(package_name, edit.id).listings || []).map(&:language).uniq.sort
    service.delete_edit(package_name, edit.id) rescue nil
    puts "FASTLANE_LOCALES=#{locales.join(',')}"
  end
