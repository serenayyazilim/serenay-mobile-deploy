  desc "TestFlight"
  lane :beta do
    sermobile_api_key
    build_ipa
    upload_to_testflight
  end
