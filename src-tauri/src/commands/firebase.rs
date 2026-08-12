use crate::firebase::retry::{exec, exec_with_retry};
use crate::workspace::sermobileboss_config::{read_sermobileboss_config, MISSING_CONFIG_MESSAGE};
use regex::Regex;
use serde::Serialize;
use serde_json::Value;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Serialize)]
pub struct FirebaseAccountsResult {
    pub accounts: Vec<String>,
    #[serde(rename = "currentAccount")]
    pub current_account: Option<String>,
}

#[tauri::command]
pub async fn firebase_accounts() -> FirebaseAccountsResult {
    let output = exec("firebase login:list 2>/dev/null || echo 'NO_ACCOUNTS'").await;
    let Ok(output) = output else {
        return FirebaseAccountsResult { accounts: vec![], current_account: None };
    };

    if output.stdout.contains("NO_ACCOUNTS") || output.stdout.contains("No authorized accounts") {
        return FirebaseAccountsResult { accounts: vec![], current_account: None };
    }

    let mut accounts = Vec::new();
    let mut current_account = None;

    let logged_in_re = Regex::new(r"(?i)Logged in as\s+(\S+)").unwrap();
    let email_re = Regex::new(r"[\w.-]+@[\w.-]+\.\w+").unwrap();

    for line in output.stdout.lines() {
        if let Some(c) = logged_in_re.captures(line) {
            current_account = Some(c[1].to_string());
            accounts.push(c[1].to_string());
        }
        if let Some(m) = email_re.find(line) {
            let email = m.as_str().to_string();
            if !accounts.contains(&email) {
                accounts.push(email);
            }
        }
    }

    FirebaseAccountsResult { accounts, current_account }
}

#[tauri::command]
pub async fn firebase_logout() -> Result<(), String> {
    exec("firebase logout").await?;
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct FirebaseStep {
    pub step: String,
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct FirebaseCreateResult {
    pub success: bool,
    pub partial: bool,
    pub message: String,
    pub steps: Vec<FirebaseStep>,
    #[serde(rename = "firebaseProjectId")]
    pub firebase_project_id: String,
}

fn extract_app_id(stdout: &str) -> Option<String> {
    let re1 = Regex::new(r"(?i)App ID:\s*(\S+)").unwrap();
    if let Some(c) = re1.captures(stdout) {
        return Some(c[1].to_string());
    }
    let re2 = Regex::new(r#""appId":\s*"([^"]+)""#).unwrap();
    re2.captures(stdout).map(|c| c[1].to_string())
}

/// Equivalent of `POST /api/firebase/create-project` — creates a project and
/// Android/iOS apps via the Firebase CLI, writing config files
/// (`google-services.json`, `GoogleService-Info.plist`) into the project folder. Retries on 429 quota errors.
#[tauri::command]
pub async fn firebase_create_project(
    project_id: String,
    app_name: String,
    bundle_id: Option<String>,
    workspace_path: String,
) -> Result<FirebaseCreateResult, String> {
    let package_name = match bundle_id {
        Some(b) if !b.is_empty() => b,
        _ => {
            let boss_config = read_sermobileboss_config(&workspace_path).ok_or(MISSING_CONFIG_MESSAGE)?;
            format!("{}.{}", boss_config.bundle_id_prefix, project_id)
        }
    };

    let firebase_project_id = format!("sermobilepro-{project_id}");
    let project_folder = Path::new(&workspace_path).join("lib/conf/sermobplus-projects").join(&project_id);
    std::fs::create_dir_all(&project_folder).map_err(|e| e.to_string())?;

    let mut steps: Vec<FirebaseStep> = Vec::new();
    let retry_delay = Duration::from_secs(20);

    // 1. Firebase projesi
    let create_cmd = format!("firebase projects:create {firebase_project_id} --display-name \"{app_name}\"");
    let create_output = exec(&create_cmd).await?;
    if create_output.success {
        steps.push(FirebaseStep { step: "project".into(), success: true, message: "Firebase project created".into() });
    } else {
        let msg = format!("{}{}", create_output.stdout, create_output.stderr);
        let lower = msg.to_lowercase();
        if lower.contains("already exists") || lower.contains("already a project") {
            steps.push(FirebaseStep { step: "project".into(), success: true, message: "Firebase project already exists, continuing".into() });
        } else {
            steps.push(FirebaseStep { step: "project".into(), success: false, message: msg });
            return Ok(FirebaseCreateResult {
                success: false,
                partial: false,
                message: "Failed to create Firebase project".into(),
                steps,
                firebase_project_id,
            });
        }
    }

    // 2. Android app
    let mut android_app_id: Option<String> = None;
    let android_cmd = format!("firebase apps:create android --project {firebase_project_id} --package-name {package_name}");
    let android_output = exec_with_retry(&android_cmd, 3, retry_delay).await?;
    if android_output.success {
        android_app_id = extract_app_id(&android_output.stdout);
        steps.push(FirebaseStep { step: "android-app".into(), success: true, message: "Android app created".into() });
    } else {
        let msg = format!("{}{}", android_output.stdout, android_output.stderr);
        let lower = msg.to_lowercase();
        if lower.contains("already exists") || lower.contains("already a project") {
            steps.push(FirebaseStep { step: "android-app".into(), success: true, message: "Android app already exists".into() });
        } else if lower.contains("429") || lower.contains("quota exceeded") || lower.contains("rate_limit_exceeded") || lower.contains("resource_exhausted") {
            steps.push(FirebaseStep { step: "android-app".into(), success: false, message: "Firebase quota limit exceeded (429). Wait a few minutes and try again.".into() });
        } else {
            steps.push(FirebaseStep { step: "android-app".into(), success: false, message: msg });
        }
    }

    if android_app_id.is_none() {
        let list_cmd = format!("firebase apps:list android --project {firebase_project_id} --json");
        if let Ok(list_output) = exec(&list_cmd).await {
            if let Ok(data) = serde_json::from_str::<Value>(&list_output.stdout) {
                let apps = data.get("result").cloned().unwrap_or(data);
                if let Some(arr) = apps.as_array() {
                    let matching = arr
                        .iter()
                        .find(|a| a.get("packageName").and_then(|v| v.as_str()) == Some(package_name.as_str()))
                        .or_else(|| arr.first());
                    android_app_id = matching.and_then(|a| a.get("appId").and_then(|v| v.as_str())).map(String::from);
                }
            }
        }
    }

    // 3. Android SDK config
    let android_sdk_cmd = match &android_app_id {
        Some(id) => format!("firebase apps:sdkconfig android --project {firebase_project_id} {id}"),
        None => format!("firebase apps:sdkconfig android --project {firebase_project_id}"),
    };
    match exec(&android_sdk_cmd).await {
        Ok(output) if output.success => {
            let json_re = Regex::new(r"(?s)\{.*\}").unwrap();
            let content = json_re.find(&output.stdout).map(|m| m.as_str().to_string()).unwrap_or(output.stdout.clone());
            let config_path = project_folder.join("google-services.json");
            let _ = std::fs::write(&config_path, content);
            steps.push(FirebaseStep { step: "android-config".into(), success: true, message: "google-services.json saved".into() });
        }
        Ok(output) => {
            steps.push(FirebaseStep { step: "android-config".into(), success: false, message: format!("{}{}", output.stdout, output.stderr) });
        }
        Err(e) => steps.push(FirebaseStep { step: "android-config".into(), success: false, message: e }),
    }

    // 4. iOS app
    let mut ios_app_id: Option<String> = None;
    let ios_cmd = format!("firebase apps:create ios --project {firebase_project_id} --bundle-id {package_name}");
    let ios_output = exec_with_retry(&ios_cmd, 3, retry_delay).await?;
    if ios_output.success {
        ios_app_id = extract_app_id(&ios_output.stdout);
        steps.push(FirebaseStep { step: "ios-app".into(), success: true, message: "iOS app created".into() });
    } else {
        let msg = format!("{}{}", ios_output.stdout, ios_output.stderr);
        let lower = msg.to_lowercase();
        if lower.contains("already exists") || lower.contains("already a project") {
            steps.push(FirebaseStep { step: "ios-app".into(), success: true, message: "iOS app already exists".into() });
        } else if lower.contains("429") || lower.contains("quota exceeded") || lower.contains("rate_limit_exceeded") || lower.contains("resource_exhausted") {
            steps.push(FirebaseStep { step: "ios-app".into(), success: false, message: "Firebase quota limit exceeded (429). Wait a few minutes and try again.".into() });
        } else {
            steps.push(FirebaseStep { step: "ios-app".into(), success: false, message: msg });
        }
    }

    if ios_app_id.is_none() {
        let list_cmd = format!("firebase apps:list ios --project {firebase_project_id} --json");
        if let Ok(list_output) = exec(&list_cmd).await {
            if let Ok(data) = serde_json::from_str::<Value>(&list_output.stdout) {
                let apps = data.get("result").cloned().unwrap_or(data);
                if let Some(arr) = apps.as_array() {
                    let matching = arr
                        .iter()
                        .find(|a| a.get("bundleId").and_then(|v| v.as_str()) == Some(package_name.as_str()))
                        .or_else(|| arr.first());
                    ios_app_id = matching.and_then(|a| a.get("appId").and_then(|v| v.as_str())).map(String::from);
                }
            }
        }
    }

    // 5. iOS SDK config
    let ios_sdk_cmd = match &ios_app_id {
        Some(id) => format!("firebase apps:sdkconfig ios --project {firebase_project_id} {id}"),
        None => format!("firebase apps:sdkconfig ios --project {firebase_project_id}"),
    };
    match exec(&ios_sdk_cmd).await {
        Ok(output) if output.success => {
            let plist_re = Regex::new(r"(?s)<\?xml.*</plist>").unwrap();
            let content = plist_re.find(&output.stdout).map(|m| m.as_str().to_string()).unwrap_or(output.stdout.clone());
            let config_path = project_folder.join("GoogleService-Info.plist");
            let _ = std::fs::write(&config_path, content);
            steps.push(FirebaseStep { step: "ios-config".into(), success: true, message: "GoogleService-Info.plist saved".into() });
        }
        Ok(output) => {
            steps.push(FirebaseStep { step: "ios-config".into(), success: false, message: format!("{}{}", output.stdout, output.stderr) });
        }
        Err(e) => steps.push(FirebaseStep { step: "ios-config".into(), success: false, message: e }),
    }

    let all_success = steps.iter().all(|s| s.success);
    let some_success = steps.iter().any(|s| s.success);

    Ok(FirebaseCreateResult {
        success: all_success,
        partial: !all_success && some_success,
        message: if all_success {
            "Firebase project and apps created successfully".to_string()
        } else if some_success {
            "Firebase setup partially completed".to_string()
        } else {
            "Firebase setup failed".to_string()
        },
        steps,
        firebase_project_id,
    })
}
