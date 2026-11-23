  # Signs in with the App Store Connect API key the app passes in. The key is the only way in:
  # no Apple ID session, so no 2-step code is ever asked.
  private_lane :sermobile_api_key do
    if ENV["ASC_KEY_ID"] && ENV["ASC_ISSUER_ID"] && ENV["ASC_PRIVATE_KEY"]
      app_store_connect_api_key(
        key_id: ENV["ASC_KEY_ID"],
        issuer_id: ENV["ASC_ISSUER_ID"],
        key_content: ENV["ASC_PRIVATE_KEY"],
        is_key_content_base64: false,
        in_house: false
      )
    else
      UI.user_error!("App Store Connect API key is not set; add it in the app (Workspace Settings).")
    end
  end
