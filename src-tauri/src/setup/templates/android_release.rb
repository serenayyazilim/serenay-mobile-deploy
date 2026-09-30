  desc "Google Play production"
  lane :release do
    require "json"
    require "tmpdir"

    text = ENV["WHATS_NEW"].to_s
    translations = JSON.parse(ENV["WHATS_NEW_TRANSLATIONS"] || "{}")
    locales = ENV["STORE_LOCALES_ANDROID"].to_s.split(",").map(&:strip).reject(&:empty?)

    build_aab

    # supply reads changelogs only from metadata folders. A fresh folder holds exactly
    # the store's languages, so no leftover folder can add a new language to the app.
    Dir.mktmpdir do |metadata|
      locales.each do |locale|
        dir = File.join(metadata, locale, "changelogs")
        FileUtils.mkdir_p(dir)
        File.write(File.join(dir, "default.txt"), translations[locale] || text)
      end

      upload_to_play_store(
        track: "production",{{AAB_ARG}}
        metadata_path: metadata,
        skip_upload_metadata: true,
        skip_upload_changelogs: locales.empty?,
        skip_upload_images: true,
        skip_upload_screenshots: true
      )
    end
  end
