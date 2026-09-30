  desc "App Store"
  lane :release do
    require "json"

    text = ENV["WHATS_NEW"].to_s
    translations = JSON.parse(ENV["WHATS_NEW_TRANSLATIONS"] || "{}")
    locales = ENV["STORE_LOCALES_IOS"].to_s.split(",").map(&:strip).reject(&:empty?)

    sermobile_api_key
    build_ipa
    deliver(
      force: true,
      skip_screenshots: true,
      skip_metadata: locales.empty?,
      release_notes: locales.to_h { |locale| [locale, translations[locale] || text] },
      precheck_include_in_app_purchases: false,
      automatic_release: true,
      submit_for_review: true,
      submission_information: { add_id_info_uses_idfa: false }
    )
  end
