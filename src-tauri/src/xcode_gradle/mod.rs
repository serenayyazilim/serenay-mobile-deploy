use regex::Regex;
use serde_json::Value;
use std::path::Path;

/// `Info.plist`/`.pbxproj` versiyon alanlarını `1.2.3+45` formatından ayırır.
pub fn split_version(version: &str) -> (String, String) {
    let mut parts = version.splitn(2, '+');
    let semver = parts.next().unwrap_or("1.0.0").to_string();
    let build = parts.next().unwrap_or("1").to_string();
    (semver, build)
}

pub fn update_generated_xcconfig(ios_folder: &Path, semver: &str, build_num: &str) {
    let path = ios_folder.join("Flutter/Generated.xcconfig");
    let Ok(mut content) = std::fs::read_to_string(&path) else { return };
    content = Regex::new(r"(?m)^FLUTTER_BUILD_NAME=.*").unwrap().replace(&content, format!("FLUTTER_BUILD_NAME={semver}")).to_string();
    content = Regex::new(r"(?m)^FLUTTER_BUILD_NUMBER=.*").unwrap().replace(&content, format!("FLUTTER_BUILD_NUMBER={build_num}")).to_string();
    let _ = std::fs::write(&path, content);
}

/// `project.pbxproj`'da bundle ID (Runner + ImageNotification target) ve
/// MARKETING_VERSION/CURRENT_PROJECT_VERSION regex ile patch edilir.
/// Davranış paritesi kritik: sıralama (ImageNotification önce, sonra genel) korunmalı.
pub fn patch_pbxproj(ios_folder: &Path, bundle_name: &str, version: Option<&str>) -> bool {
    let path = ios_folder.join("Runner.xcodeproj/project.pbxproj");
    let Ok(mut content) = std::fs::read_to_string(&path) else { return false };

    content = Regex::new(r"PRODUCT_BUNDLE_IDENTIFIER = [^;]+\.ImageNotification;")
        .unwrap()
        .replace_all(&content, format!("PRODUCT_BUNDLE_IDENTIFIER = {bundle_name}.ImageNotification;").as_str())
        .to_string();

    content = Regex::new(r"PRODUCT_BUNDLE_IDENTIFIER = (?P<rest>[^;]+);")
        .unwrap()
        .replace_all(&content, |caps: &regex::Captures| {
            let rest = &caps["rest"];
            if rest.ends_with(".ImageNotification") {
                caps[0].to_string()
            } else {
                format!("PRODUCT_BUNDLE_IDENTIFIER = {bundle_name};")
            }
        })
        .to_string();

    if let Some(version) = version {
        let (semver, build_num) = split_version(version);
        content = Regex::new(r"MARKETING_VERSION = [^;]+;").unwrap().replace_all(&content, format!("MARKETING_VERSION = {semver};").as_str()).to_string();
        content = Regex::new(r"CURRENT_PROJECT_VERSION = [^;]+;").unwrap().replace_all(&content, format!("CURRENT_PROJECT_VERSION = {build_num};").as_str()).to_string();
    }

    std::fs::write(&path, content).is_ok()
}

/// `Info.plist`'te CFBundleDisplayName + CFBundleShortVersionString/CFBundleVersion patch eder.
pub fn patch_info_plist(ios_folder: &Path, display_name: &str, version: Option<&str>) -> bool {
    let path = ios_folder.join("Runner/Info.plist");
    let Ok(mut content) = std::fs::read_to_string(&path) else { return false };

    content = Regex::new(r"<key>CFBundleDisplayName</key>\s*<string>[^<]*</string>")
        .unwrap()
        .replace(&content, format!("<key>CFBundleDisplayName</key>\n\t<string>{display_name}</string>").as_str())
        .to_string();

    if let Some(version) = version {
        let (semver, build_num) = split_version(version);
        content = Regex::new(r"<key>CFBundleShortVersionString</key>\s*<string>[^<]*</string>")
            .unwrap()
            .replace(&content, format!("<key>CFBundleShortVersionString</key>\n\t<string>{semver}</string>").as_str())
            .to_string();
        content = Regex::new(r"<key>CFBundleVersion</key>\s*<string>[^<]*</string>")
            .unwrap()
            .replace(&content, format!("<key>CFBundleVersion</key>\n\t<string>{build_num}</string>").as_str())
            .to_string();
    }

    std::fs::write(&path, content).is_ok()
}

/// `app/api/projects/versions/route.ts::updateNativeVersionFiles` karşılığı —
/// bundle ID/display name'e dokunmadan sadece versiyon alanlarını günceller.
pub fn patch_pbxproj_version_only(ios_folder: &Path, version: &str) -> bool {
    let path = ios_folder.join("Runner.xcodeproj/project.pbxproj");
    let Ok(mut content) = std::fs::read_to_string(&path) else { return false };
    let (semver, build_num) = split_version(version);
    content = Regex::new(r"MARKETING_VERSION = [^;]+;").unwrap().replace_all(&content, format!("MARKETING_VERSION = {semver};").as_str()).to_string();
    content = Regex::new(r"CURRENT_PROJECT_VERSION = [^;]+;").unwrap().replace_all(&content, format!("CURRENT_PROJECT_VERSION = {build_num};").as_str()).to_string();
    std::fs::write(&path, content).is_ok()
}

pub fn patch_info_plist_version_only(ios_folder: &Path, version: &str) -> bool {
    let path = ios_folder.join("Runner/Info.plist");
    let Ok(mut content) = std::fs::read_to_string(&path) else { return false };
    let (semver, build_num) = split_version(version);
    content = Regex::new(r"<key>CFBundleShortVersionString</key>\s*<string>[^<]*</string>")
        .unwrap()
        .replace(&content, format!("<key>CFBundleShortVersionString</key>\n\t<string>{semver}</string>").as_str())
        .to_string();
    content = Regex::new(r"<key>CFBundleVersion</key>\s*<string>[^<]*</string>")
        .unwrap()
        .replace(&content, format!("<key>CFBundleVersion</key>\n\t<string>{build_num}</string>").as_str())
        .to_string();
    std::fs::write(&path, content).is_ok()
}

pub fn patch_android_manifest(android_folder: &Path, display_name: &str) -> bool {
    let path = android_folder.join("app/src/main/AndroidManifest.xml");
    let Ok(content) = std::fs::read_to_string(&path) else { return false };
    let patched = Regex::new(r#"android:label="[^"]+""#)
        .unwrap()
        .replace_all(&content, format!("android:label=\"{display_name}\"").as_str())
        .to_string();
    std::fs::write(&path, patched).is_ok()
}

pub fn patch_android_gradle(android_folder: &Path, bundle_name: &str) -> bool {
    let path = android_folder.join("app/build.gradle");
    let Ok(content) = std::fs::read_to_string(&path) else { return false };
    let patched = Regex::new(r#"applicationId "[^"]+""#)
        .unwrap()
        .replace(&content, format!("applicationId \"{bundle_name}\"").as_str())
        .to_string();
    std::fs::write(&path, patched).is_ok()
}

/// `google-services.json` + `GoogleService-Info.plist`'i FlutterFire CLI'sız
/// parse edip `firebase_options.dart`'ı manuel template ile üretir.
pub fn generate_firebase_options(google_services_json: &Path, google_service_info_plist: &Path, output_path: &Path) -> bool {
    let Ok(gs_content) = std::fs::read_to_string(google_services_json) else { return false };
    let Ok(plist_content) = std::fs::read_to_string(google_service_info_plist) else { return false };
    let Ok(gs_json): Result<Value, _> = serde_json::from_str(&gs_content) else { return false };

    let project_id = gs_json["project_info"]["project_id"].as_str().unwrap_or("");
    let storage_bucket = gs_json["project_info"]["storage_bucket"].as_str().unwrap_or("");
    let messaging_sender_id = gs_json["project_info"]["project_number"].as_str().unwrap_or("");
    let android_app_id = gs_json["client"][0]["client_info"]["mobilesdk_app_id"].as_str().unwrap_or("");
    let android_api_key = gs_json["client"][0]["api_key"][0]["current_key"].as_str().unwrap_or("");

    let plist_value = |key: &str| -> String {
        let re = Regex::new(&format!(r"<key>{key}</key>\s*<string>([^<]+)</string>")).unwrap();
        re.captures(&plist_content).map(|c| c[1].to_string()).unwrap_or_default()
    };
    let ios_api_key = plist_value("API_KEY");
    let ios_app_id = plist_value("GOOGLE_APP_ID");
    let ios_bundle_id = plist_value("BUNDLE_ID");

    let dart_content = format!(
        r#"// File generated by FlutterFire CLI.
// ignore_for_file: type=lint
import 'package:firebase_core/firebase_core.dart' show FirebaseOptions;
import 'package:flutter/foundation.dart'
    show defaultTargetPlatform, kIsWeb, TargetPlatform;

/// Default [FirebaseOptions] for use with your Firebase apps.
class DefaultFirebaseOptions {{
  static FirebaseOptions get currentPlatform {{
    if (kIsWeb) {{
      throw UnsupportedError(
        'DefaultFirebaseOptions have not been configured for web - '
        'you can reconfigure this by running the FlutterFire CLI again.',
      );
    }}
    switch (defaultTargetPlatform) {{
      case TargetPlatform.android:
        return android;
      case TargetPlatform.iOS:
        return ios;
      default:
        throw UnsupportedError(
          'DefaultFirebaseOptions are not supported for this platform.',
        );
    }}
  }}

  static const FirebaseOptions android = FirebaseOptions(
    apiKey: '{android_api_key}',
    appId: '{android_app_id}',
    messagingSenderId: '{messaging_sender_id}',
    projectId: '{project_id}',
    storageBucket: '{storage_bucket}',
  );

  static const FirebaseOptions ios = FirebaseOptions(
    apiKey: '{ios_api_key}',
    appId: '{ios_app_id}',
    messagingSenderId: '{messaging_sender_id}',
    projectId: '{project_id}',
    storageBucket: '{storage_bucket}',
    iosBundleId: '{ios_bundle_id}',
  );
}}
"#
    );

    std::fs::write(output_path, dart_content).is_ok()
}
