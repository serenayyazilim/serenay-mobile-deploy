  private_lane :build_ipa do
{{PREPARE}}
    build_app(
      {{BUILD_ARGS}},
      clean: true,
      xcargs: "-allowProvisioningUpdates",
      export_xcargs: "-allowProvisioningUpdates"
    )
  end
