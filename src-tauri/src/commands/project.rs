use crate::workspace::detect::detect_workspace_mode;
use crate::workspace::sermobileboss_config::{build_keystore_dname, read_sermobileboss_config};
use crate::workspace::types::WorkspaceMode;
use crate::xcode_gradle;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;
use tokio::process::Command;

#[derive(Debug, Deserialize)]
pub struct CreateProjectParams {
    #[serde(rename = "workspacePath")]
    pub workspace_path: String,
    #[serde(rename = "projectId")]
    pub project_id: String,
    #[serde(rename = "appName")]
    pub app_name: String,
    #[serde(rename = "bgColor")]
    pub bg_color: Option<String>,
    #[serde(rename = "icon512Path")]
    pub icon512_path: Option<String>,
    #[serde(rename = "icon1024Path")]
    pub icon1024_path: Option<String>,
    #[serde(rename = "appLogoPath")]
    pub app_logo_path: Option<String>,
    #[serde(rename = "splashPath")]
    pub splash_path: Option<String>,
    pub serconf: Option<BTreeMap<String, Value>>,
    #[serde(rename = "firebaseAndroidPath")]
    pub firebase_android_path: Option<String>,
    #[serde(rename = "firebaseIosPath")]
    pub firebase_ios_path: Option<String>,
    #[serde(rename = "sentryDsn")]
    pub sentry_dsn: Option<String>,
    #[serde(default)]
    pub resume: bool,
}

fn copy_if_present(src: &Option<String>, dest: &Path) -> std::io::Result<bool> {
    match src {
        Some(p) if Path::new(p).exists() => {
            std::fs::copy(p, dest)?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

#[tauri::command]
pub async fn project_create(params: CreateProjectParams) -> Result<String, String> {
    let workspace_path = params.workspace_path.clone();
    let project_id = params.project_id.clone();
    let app_name = params.app_name.clone();
    let bg_color = params.bg_color.clone().unwrap_or_else(|| "#FFFFFF".to_string());

    let boss_config = read_sermobileboss_config(&workspace_path)
        .ok_or("Bundle ID / keystore settings are missing. Fill them in via \"Workspace Settings\" in the top menu.")?;

    if !Regex::new(r"^[a-z0-9]+$").unwrap().is_match(&project_id) {
        return Err("Project ID may only contain lowercase letters and digits".to_string());
    }

    let root = Path::new(&workspace_path);
    let projects_json_path = root.join("sermobileboss_projects.json");
    let mut projects_data: BTreeMap<String, String> = std::fs::read_to_string(&projects_json_path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default();

    if projects_data.contains_key(&project_id) && !params.resume {
        return Err("This project ID already exists".to_string());
    }

    let projects_folder = root.join("lib/conf/sermobplus-projects");
    let project_folder = projects_folder.join(&project_id);

    // 1. Project folders
    std::fs::create_dir_all(project_folder.join("Launch")).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(project_folder.join("Store")).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(project_folder.join("Graphic")).map_err(|e| e.to_string())?;

    // 2. Icon files
    let launchers_dir = root.join("assets/launchers");
    std::fs::create_dir_all(&launchers_dir).map_err(|e| e.to_string())?;

    if copy_if_present(&params.icon512_path, &launchers_dir.join(format!("{project_id}.png"))).map_err(|e| e.to_string())? {
        copy_if_present(&params.icon512_path, &project_folder.join("Graphic/512x512 icon.png")).map_err(|e| e.to_string())?;
    }

    let logo_source = params.app_logo_path.clone().or_else(|| params.icon512_path.clone());
    if let Some(logo) = &logo_source {
        let images_dir = root.join("assets/images");
        std::fs::create_dir_all(&images_dir).map_err(|e| e.to_string())?;
        copy_if_present(&Some(logo.clone()), &images_dir.join(format!("logo-{project_id}.png"))).map_err(|e| e.to_string())?;
    }

    if copy_if_present(&params.icon1024_path, &launchers_dir.join(format!("{project_id}-ios.png"))).map_err(|e| e.to_string())? {
        copy_if_present(&params.icon1024_path, &project_folder.join("Graphic/1024x1024 icon.png")).map_err(|e| e.to_string())?;
    }

    // 3. Splash + Firebase
    copy_if_present(&params.splash_path, &project_folder.join("Launch/splash.png")).map_err(|e| e.to_string())?;
    copy_if_present(&params.firebase_android_path, &project_folder.join("google-services.json")).map_err(|e| e.to_string())?;
    copy_if_present(&params.firebase_ios_path, &project_folder.join("GoogleService-Info.plist")).map_err(|e| e.to_string())?;

    // 4. Copy serconf.dart template
    let active_project = std::fs::read_to_string(root.join("sermobileboss.txt")).ok().map(|s| s.trim().to_string());
    let mut template_project = active_project.clone();
    if template_project.as_deref().map(|p| !projects_folder.join(p).join("serconf.dart").exists()).unwrap_or(true) {
        template_project = projects_data.keys().find(|k| projects_folder.join(k).join("serconf.dart").exists()).cloned();
    }

    let serconf_path = project_folder.join("serconf.dart");
    if let Some(tpl) = &template_project {
        if !serconf_path.exists() {
            if let Ok(content) = std::fs::read_to_string(projects_folder.join(tpl).join("serconf.dart")) {
                std::fs::write(&serconf_path, content).map_err(|e| e.to_string())?;
            }
        }
    }

    let colors_path = project_folder.join("colors.dart");
    if !colors_path.exists() {
        let hex = bg_color.trim_start_matches('#');
        let content = format!(
            "import 'package:flutter/material.dart';\n\nconst fallbackPrimary = Color(0xFF{hex});\nconst fallbackAccent = Color(0xFFFFFBFA);\nconst fallbackDark = Color(0xFF37474F);\n"
        );
        std::fs::write(&colors_path, content).map_err(|e| e.to_string())?;
    }

    // 4b. Apply serconf fields
    if let Some(mut serconf) = params.serconf {
        if serconf_path.exists() {
            let mut content = std::fs::read_to_string(&serconf_path).map_err(|e| e.to_string())?;

            if !matches!(serconf.get("COMPANY_NAME"), Some(Value::String(s)) if !s.is_empty()) {
                serconf.insert("COMPANY_NAME".to_string(), Value::String(app_name.clone()));
            }

            let api_url = serconf.get("API_URL").and_then(|v| v.as_str()).map(String::from);
            if let Some(url) = &api_url {
                if !url.is_empty() && !url.starts_with("http") {
                    serconf.insert("API_URL".to_string(), Value::String(format!("https://{url}/sermobileboss/")));
                }
            }
            let iletisim_url = serconf.get("ILETISIM_URL").and_then(|v| v.as_str()).map(String::from);
            match iletisim_url {
                Some(url) if !url.is_empty() && !url.starts_with("http") => {
                    serconf.insert("ILETISIM_URL".to_string(), Value::String(format!("https://{url}/sermobileboss/")));
                }
                None => {
                    if let Some(Value::String(url)) = serconf.get("API_URL").cloned() {
                        serconf.insert("ILETISIM_URL".to_string(), Value::String(url));
                    }
                }
                _ => {}
            }

            for (key, value) in &serconf {
                let dart_value = match value {
                    Value::Bool(b) => (if *b { "true" } else { "false" }).to_string(),
                    Value::String(s) if s.starts_with("LoginType.") => s.clone(),
                    Value::String(s) => format!("'{}'", s.replace('\'', "\\'")),
                    other => other.to_string(),
                };
                let re = Regex::new(&format!(r"(const\s+{key}\s*=\s*)[^;]+(;)")).unwrap();
                if re.is_match(&content) {
                    content = re.replace_all(&content, format!("${{1}}{dart_value}${{2}}").as_str()).to_string();
                }
            }

            let logo_path = format!("assets/images/logo-{project_id}.png");
            let logo_re = Regex::new(r"const\s+COMPANY_IMAGE_LOGO\s*=\s*[^;]+;").unwrap();
            content = logo_re.replace_all(&content, format!("const COMPANY_IMAGE_LOGO = '{logo_path}';").as_str()).to_string();

            if let Some(dsn) = &params.sentry_dsn {
                if !dsn.is_empty() {
                    let dsn_re = Regex::new(r"const\s+SENTRY_DSN\s*=\s*[^;]+;").unwrap();
                    content = dsn_re
                        .replace_all(&content, format!("const SENTRY_DSN =\n    '{dsn}';").as_str())
                        .to_string();
                }
            }

            std::fs::write(&serconf_path, content).map_err(|e| e.to_string())?;
        }
    }

    // 5. icons-<projectId>.yaml
    let icons_yaml = format!(
        "flutter_icons:\n  android: true\n  ios: true\n  image_path: \"assets/launchers/{project_id}.png\"\n  remove_alpha_ios: true\n"
    );
    std::fs::write(root.join(format!("icons-{project_id}.yaml")), icons_yaml).map_err(|e| e.to_string())?;

    // 5.5. Initial version
    let version_path = project_folder.join("version.json");
    let mut version_data: serde_json::Map<String, Value> = std::fs::read_to_string(&version_path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default();
    version_data.insert("version".to_string(), Value::String("1.0.0+1".to_string()));
    std::fs::write(&version_path, serde_json::to_string_pretty(&version_data).unwrap()).map_err(|e| e.to_string())?;

    // 6. Update sermobileboss_projects.json
    projects_data.insert(project_id.clone(), app_name.clone());
    std::fs::write(&projects_json_path, serde_json::to_string_pretty(&projects_data).unwrap()).map_err(|e| e.to_string())?;

    // 7. Android keystore (if missing)
    let keystore_path = project_folder.join(format!("{project_id}.keystore"));
    let keystore_alias = format!("{}{}", boss_config.keystore_alias_prefix, project_id);
    let keystore_password = boss_config.keystore_password.clone();
    let keystore_dname = build_keystore_dname(&boss_config);

    if !keystore_path.exists() {
        let _ = Command::new("keytool")
            .args([
                "-genkey", "-v",
                "-keystore", keystore_path.to_string_lossy().as_ref(),
                "-alias", &keystore_alias,
                "-keyalg", "RSA",
                "-keysize", "2048",
                "-validity", "10000",
                "-dname", &keystore_dname,
                "-storepass", &keystore_password,
                "-keypass", &keystore_password,
            ])
            .output()
            .await;
    }

    // 8. key.properties
    let store_file_relative = format!("../../lib/conf/sermobplus-projects/{project_id}/{project_id}.keystore");
    let key_properties = format!(
        "storePassword={keystore_password}\nkeyPassword={keystore_password}\nkeyAlias={keystore_alias}\nstoreFile={store_file_relative}\n"
    );
    std::fs::write(project_folder.join("key.properties"), key_properties).map_err(|e| e.to_string())?;

    // 9-10. native splash + splash.json
    let splash_yaml_path = root.join(format!("flutter_native_splash-{project_id}.yaml"));
    let splash_image_path = project_folder.join("Launch/splash.png");
    let splash_relative_path = format!("lib/conf/sermobplus-projects/{project_id}/Launch/splash.png");

    if splash_image_path.exists() {
        let splash_yaml = format!(
            "flutter_native_splash:\n  color: \"{bg_color}\"\n  image: {splash_relative_path}\n  android: true\n  ios: true\n\n  android_12:\n    color: \"{bg_color}\"\n    image: {splash_relative_path}\n"
        );
        std::fs::write(&splash_yaml_path, splash_yaml).map_err(|e| e.to_string())?;

        let splash_json = serde_json::json!({ "color": bg_color, "image": splash_relative_path });
        std::fs::write(project_folder.join("splash.json"), serde_json::to_string_pretty(&splash_json).unwrap())
            .map_err(|e| e.to_string())?;

        let _ = Command::new("dart")
            .args(["run", "flutter_native_splash:create", &format!("--path={}", splash_yaml_path.to_string_lossy())])
            .current_dir(&workspace_path)
            .output()
            .await;
    }

    Ok(format!("{app_name} ({project_id}) project created successfully"))
}

fn copy_file_content(src: &Path, dest: &Path) -> bool {
    std::fs::read_to_string(src).map(|c| std::fs::write(dest, c).is_ok()).unwrap_or(false)
}

fn copy_file(src: &Path, dest: &Path) -> bool {
    src.exists() && std::fs::copy(src, dest).is_ok()
}

fn get_project_name(projects_json_path: &Path, project_id: &str) -> String {
    std::fs::read_to_string(projects_json_path)
        .ok()
        .and_then(|c| serde_json::from_str::<BTreeMap<String, String>>(&c).ok())
        .and_then(|m| m.get(project_id).cloned())
        .unwrap_or_else(|| "serMobilePro".to_string())
}

fn read_project_version(project_folder: &Path) -> Option<String> {
    let content = std::fs::read_to_string(project_folder.join("version.json")).ok()?;
    let data: Value = serde_json::from_str(&content).ok()?;
    data.get("version").and_then(|v| v.as_str()).map(String::from)
}

#[derive(Debug, Serialize)]
pub struct ActivateDetails {
    #[serde(rename = "colorsCopied")]
    colors_copied: bool,
    #[serde(rename = "configCopied")]
    config_copied: bool,
    #[serde(rename = "firebaseOptionsGenerated")]
    firebase_options_generated: bool,
    #[serde(rename = "splashGenerated")]
    splash_generated: bool,
    #[serde(rename = "androidKeysCopied")]
    android_keys_copied: bool,
    #[serde(rename = "firebaseConfigured")]
    firebase_configured: bool,
    #[serde(rename = "androidManifestUpdated")]
    android_manifest_updated: bool,
    #[serde(rename = "androidGradleUpdated")]
    android_gradle_updated: bool,
    #[serde(rename = "iosBundleUpdated")]
    ios_bundle_updated: bool,
    #[serde(rename = "launcherIconsUpdated")]
    launcher_icons_updated: bool,
}

#[derive(Debug, Serialize)]
pub struct ActivateResult {
    pub message: String,
    pub details: ActivateDetails,
}

/// Port of `app/api/project/activate/route.ts` — activates a sub-app and patches
/// native (Android/Gradle, iOS/pbxproj) project files in place via regex.
/// Behavioral parity is critical: the regex order and patterns must exactly match the original.
#[tauri::command]
pub async fn project_activate(workspace_path: String, project_id: String, backup: bool) -> Result<ActivateResult, String> {
    if detect_workspace_mode(&workspace_path) == WorkspaceMode::Generic {
        return Ok(ActivateResult {
            message: "Project is already active".to_string(),
            details: ActivateDetails {
                colors_copied: false, config_copied: false, firebase_options_generated: false, splash_generated: false,
                android_keys_copied: false, firebase_configured: false, android_manifest_updated: false,
                android_gradle_updated: false, ios_bundle_updated: false, launcher_icons_updated: false,
            },
        });
    }

    let boss_config = read_sermobileboss_config(&workspace_path)
        .ok_or("Bundle ID / keystore settings are missing. Fill them in via \"Workspace Settings\" in the top menu.")?;

    let root = Path::new(&workspace_path);
    let config_folder = root.join("lib/conf");
    let projects_folder = config_folder.join("sermobplus-projects");
    let ios_folder = root.join("ios");
    let android_folder = root.join("android");
    let data_file = root.join("sermobileboss.txt");
    let projects_json = root.join("sermobileboss_projects.json");
    let project_folder = projects_folder.join(&project_id);

    if !project_folder.exists() {
        return Err("Project folder not found".to_string());
    }

    let mut details = ActivateDetails {
        colors_copied: false, config_copied: false, firebase_options_generated: false, splash_generated: false,
        android_keys_copied: false, firebase_configured: false, android_manifest_updated: false,
        android_gradle_updated: false, ios_bundle_updated: false, launcher_icons_updated: false,
    };

    // 1. Back up the previously active project + save the pubspec version
    let active_project = std::fs::read_to_string(&data_file).ok().map(|s| s.trim().to_string());
    let pubspec_path = root.join("pubspec.yaml");

    if backup {
        if let Some(active) = &active_project {
            let active_folder = projects_folder.join(active);
            if active_folder.exists() {
                copy_file_content(&config_folder.join("colors.dart"), &active_folder.join("colors.dart"));
                copy_file_content(&config_folder.join("serconf.dart"), &active_folder.join("serconf.dart"));
            }
        }
    }

    if let Some(active) = &active_project {
        if active != &project_id {
            let active_folder = projects_folder.join(active);
            if active_folder.exists() && pubspec_path.exists() {
                if let Ok(pubspec_content) = std::fs::read_to_string(&pubspec_path) {
                    if let Some(c) = Regex::new(r"(?m)^version:\s*(.+)$").unwrap().captures(&pubspec_content) {
                        let version_json_path = active_folder.join("version.json");
                        let mut existing: serde_json::Map<String, Value> = std::fs::read_to_string(&version_json_path)
                            .ok()
                            .and_then(|c| serde_json::from_str(&c).ok())
                            .unwrap_or_default();
                        existing.insert("version".to_string(), Value::String(c[1].trim().to_string()));
                        let _ = std::fs::write(&version_json_path, serde_json::to_string_pretty(&existing).unwrap());
                    }
                }
            }
        }
    }

    // 2-3. Copy colors.dart / serconf.dart
    details.colors_copied = copy_file_content(&project_folder.join("colors.dart"), &config_folder.join("colors.dart"));
    details.config_copied = copy_file_content(&project_folder.join("serconf.dart"), &config_folder.join("serconf.dart"));

    // 4. Active project pointer
    std::fs::write(&data_file, &project_id).map_err(|e| e.to_string())?;

    // 4.5. Sync pubspec.yaml version + Generated.xcconfig
    let saved_version = read_project_version(&project_folder);
    if let Some(version) = &saved_version {
        if pubspec_path.exists() {
            if let Ok(mut pubspec_content) = std::fs::read_to_string(&pubspec_path) {
                pubspec_content = Regex::new(r"(?m)^version:\s*.+$")
                    .unwrap()
                    .replace(&pubspec_content, format!("version: {version}").as_str())
                    .to_string();
                let _ = std::fs::write(&pubspec_path, pubspec_content);

                let (semver, build_num) = xcode_gradle::split_version(version);
                xcode_gradle::update_generated_xcconfig(&ios_folder, &semver, &build_num);
            }
        }
    }

    // 5. Splash (flutter_native_splash)
    let splash_yaml_path = root.join(format!("flutter_native_splash-{project_id}.yaml"));
    let splash_bg_color = std::fs::read_to_string(project_folder.join("splash.json"))
        .ok()
        .and_then(|c| serde_json::from_str::<Value>(&c).ok())
        .and_then(|v| v.get("color").and_then(|c| c.as_str()).map(String::from))
        .unwrap_or_else(|| "#FFFFFF".to_string());

    let splash_image_new = project_folder.join("Launch/splash.png");
    let splash_image_old = project_folder.join("Launch/2x.png");
    let splash_image = if splash_image_new.exists() {
        Some(splash_image_new)
    } else if splash_image_old.exists() {
        Some(splash_image_old)
    } else {
        None
    };

    if let Some(splash_image) = &splash_image {
        let splash_relative = splash_image.strip_prefix(root).unwrap_or(splash_image).to_string_lossy().replace('\\', "/");
        let yaml = format!(
            "flutter_native_splash:\n  color: \"{splash_bg_color}\"\n  image: {splash_relative}\n  android: true\n  ios: true\n\n  android_12:\n    color: \"{splash_bg_color}\"\n    image: {splash_relative}\n"
        );
        std::fs::write(&splash_yaml_path, yaml).map_err(|e| e.to_string())?;
    } else if !splash_yaml_path.exists() {
        let yaml = format!("flutter_native_splash:\n  color: \"{splash_bg_color}\"\n  android: true\n  ios: true\n\n  android_12:\n    color: \"{splash_bg_color}\"\n");
        std::fs::write(&splash_yaml_path, yaml).map_err(|e| e.to_string())?;
    } else if let Ok(mut yaml) = std::fs::read_to_string(&splash_yaml_path) {
        yaml = Regex::new(r##"color:\s*"#[0-9A-Fa-f]+""##)
            .unwrap()
            .replace_all(&yaml, format!("color: \"{splash_bg_color}\"").as_str())
            .to_string();
        std::fs::write(&splash_yaml_path, yaml).map_err(|e| e.to_string())?;
    }

    if splash_yaml_path.exists() {
        let output = Command::new("dart")
            .args(["run", "flutter_native_splash:create", &format!("--path={}", splash_yaml_path.to_string_lossy())])
            .current_dir(&workspace_path)
            .output()
            .await;
        details.splash_generated = output.map(|o| o.status.success()).unwrap_or(false);
    }

    // 6. Android key.properties (copy)
    details.android_keys_copied = copy_file(&project_folder.join("key.properties"), &android_folder.join("key.properties"));

    // 7. Firebase files
    let google_services_updated = copy_file(&project_folder.join("google-services.json"), &android_folder.join("app/google-services.json"));
    let google_service_info_updated = copy_file(&project_folder.join("GoogleService-Info.plist"), &ios_folder.join("Runner/GoogleService-Info.plist"));
    details.firebase_configured = google_services_updated || google_service_info_updated;

    // 7.5. firebase_options.dart
    details.firebase_options_generated = xcode_gradle::generate_firebase_options(
        &project_folder.join("google-services.json"),
        &project_folder.join("GoogleService-Info.plist"),
        &root.join("lib/firebase_options.dart"),
    );

    // 8-9. Android manifest + gradle
    let display_name = get_project_name(&projects_json, &project_id);
    details.android_manifest_updated = xcode_gradle::patch_android_manifest(&android_folder, &display_name);

    let bundle_name = format!("{}.{}", boss_config.bundle_id_prefix, project_id);
    details.android_gradle_updated = xcode_gradle::patch_android_gradle(&android_folder, &bundle_name);

    // 10-10.5. iOS pbxproj + Info.plist
    details.ios_bundle_updated = xcode_gradle::patch_pbxproj(&ios_folder, &bundle_name, saved_version.as_deref());
    xcode_gradle::patch_info_plist(&ios_folder, &display_name, saved_version.as_deref());

    // 11. flutter_launcher_icons (runs in the background, result not awaited — matches original behavior)
    let icons_config_path = root.join(format!("icons-{project_id}.yaml"));
    if icons_config_path.exists() {
        let workspace_path_clone = workspace_path.clone();
        let project_id_clone = project_id.clone();
        tokio::spawn(async move {
            let _ = Command::new("flutter")
                .args(["pub", "run", "flutter_launcher_icons:main", "-f", &format!("icons-{project_id_clone}.yaml")])
                .current_dir(&workspace_path_clone)
                .output()
                .await;
        });
        details.launcher_icons_updated = true;
    }

    Ok(ActivateResult { message: format!("{project_id} project activated"), details })
}
