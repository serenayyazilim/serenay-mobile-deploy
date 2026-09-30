use crate::appstoreconnect::config::read_asc_config;
use crate::deploy::registry::DeployRegistry;
use crate::deploy::locales::{get_store_locales, APP_NOT_FOUND};
use crate::setup::{self, ProjectKind};
use crate::deploy::{find_script_path, is_two_factor_prompt, translate::build_translations};
use serde_json::json;
use std::path::Path;
use std::process::Stdio;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

/// Mirrors `deploy.rb`'s `get_splash_config`: resolves whether a usable splash image
/// exists for the project, without actually generating the native splash assets.
/// Deploy silently skips splash regeneration when no image resolves, so the frontend
/// uses this to warn the user before that happens instead of after.
#[tauri::command]
pub fn deploy_check_splash_image(workspace_path: String, project_id: String) -> bool {
    let project_folder = Path::new(&workspace_path).join("lib/conf/sermobplus-projects").join(&project_id);

    let splash_json = project_folder.join("splash.json");
    if splash_json.exists() {
        let image = std::fs::read_to_string(&splash_json)
            .ok()
            .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok())
            .and_then(|json| json.get("image").and_then(|v| v.as_str()).map(String::from));

        return match image {
            Some(image) => Path::new(&workspace_path).join(image).exists(),
            None => false,
        };
    }

    let launch = project_folder.join("Launch");
    launch.join("splash.png").exists() || launch.join("2x.png").exists()
}

pub(crate) async fn stream_lines<R: tokio::io::AsyncRead + Unpin>(stream: R, app: AppHandle, event_name: String, is_error: bool) {
    let mut lines = BufReader::new(stream).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        if line.trim().is_empty() {
            continue;
        }
        if is_two_factor_prompt(&line) {
            let _ = app.emit(&event_name, json!({ "type": "input_required", "prompt": line }));
        } else {
            let event_type = if is_error { "error" } else { "log" };
            let _ = app.emit(&event_name, json!({ "type": event_type, "message": line }));
        }
    }
}

/// Equivalent of `POST /api/deploy` (SSE). The command returns a `processId` immediately;
/// deploy progress is published via the `deploy-event-{processId}` event.
#[tauri::command]
pub async fn deploy_start(
    app: AppHandle,
    registry: State<'_, DeployRegistry>,
    platform: String,
    workspace_path: String,
    whats_new: Option<String>,
    bump_version: Option<bool>,
    track: Option<String>,
) -> Result<String, String> {
    if !["ios", "android", "huawei", "all"].contains(&platform.as_str()) {
        return Err("Invalid platform. Must be ios, android, huawei or all".to_string());
    }

    let track = track.unwrap_or_else(|| "production".to_string());
    if !["production", "test"].contains(&track.as_str()) {
        return Err("Invalid track. Must be production or test".to_string());
    }

    let script_path = find_script_path(&app, "deploy.rb").ok_or("Deploy script not found")?;

    let whats_new_text = whats_new
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "Bug fixes and performance improvements.".to_string());

    // Only the production lanes upload release notes, and deliver/supply create any locale
    // they're given as a new store language — so without the store's real list we stop
    // here instead of guessing.
    let (ios_locales, android_locales) = if track == "production" {
        let (ios, android) = get_store_locales(&workspace_path).await;
        let require = |needed: bool, store: &str, platform: &str, result: Result<Vec<String>, String>| match result {
            Err(e) if needed && e == APP_NOT_FOUND => {
                Err(format!("The app doesn't exist on {store} yet ({APP_NOT_FOUND}:{platform})"))
            }
            Err(e) if needed => Err(format!(
                "Could not read the app's {store} languages ({e}). Deploy stopped so no new store languages get created."
            )),
            result => Ok(result.unwrap_or_default()),
        };
        (
            require(platform == "ios" || platform == "all", "App Store", "ios", ios)?,
            require(platform == "android" || platform == "all", "Google Play", "android", android)?,
        )
    } else {
        (vec![], vec![])
    };
    let translations = build_translations(&whats_new_text, &ios_locales, &android_locales).await;

    // Non-Flutter projects keep their version in the native projects and their Fastfile
    // lanes build the app themselves, so deploy.rb only runs the lanes for them.
    let root = Path::new(&workspace_path);
    let kind = setup::detect_kind(root).ok_or("No supported mobile project found")?;
    let lanes_only = kind != ProjectKind::Flutter;
    let (ios_dir, android_dir) = setup::platform_dirs(root, kind);
    if lanes_only {
        if kind == ProjectKind::Expo {
            return Err("This Expo project has no ios/ and android/ folders yet. Open it again from the project selector to run expo prebuild.".to_string());
        }
        let missing = match platform.as_str() {
            "ios" => ios_dir.is_none(),
            "android" => android_dir.is_none(),
            "all" => ios_dir.is_none() || android_dir.is_none(),
            _ => true,
        };
        if missing {
            return Err(format!("A {} project can't be deployed to {platform}", kind.label()));
        }
        if bump_version.unwrap_or(true) {
            let root = root.to_path_buf();
            tokio::task::spawn_blocking(move || {
                let current = setup::read_version(&root, kind).ok_or("Could not find the app's current version")?;
                setup::write_version(&root, kind, &setup::next_version(&current))
            })
            .await
            .map_err(|e| e.to_string())??;
        }
    }

    let process_id = uuid::Uuid::new_v4().to_string();
    let event_name = format!("deploy-event-{process_id}");

    let mut command = Command::new("ruby");
    command
        .arg(&script_path)
        .arg(&platform)
        .arg(&workspace_path)
        .current_dir(&workspace_path)
        .env("LANG", "en_US.UTF-8")
        .env("WHATS_NEW", &whats_new_text)
        .env("WHATS_NEW_TRANSLATIONS", serde_json::to_string(&translations).unwrap_or_default())
        .env("STORE_LOCALES_IOS", ios_locales.join(","))
        .env("STORE_LOCALES_ANDROID", android_locales.join(","))
        .env("BUMP_VERSION", if bump_version.unwrap_or(true) && !lanes_only { "true" } else { "false" })
        .env("PROJECT_KIND", if lanes_only { "lanes" } else { "flutter" })
        .env("FASTLANE_IOS_DIR", ios_dir.map(|d| d.to_string_lossy().to_string()).unwrap_or_default())
        .env("FASTLANE_ANDROID_DIR", android_dir.map(|d| d.to_string_lossy().to_string()).unwrap_or_default())
        .env("RELEASE_TRACK", &track);

    // Pass the App Store Connect API key through to fastlane so it authenticates
    // with a key instead of the legacy Apple ID/app-specific-password session flow,
    // which is prone to intermittent "Could not receive latest API key" failures.
    if let Some(asc) = read_asc_config(&workspace_path) {
        command
            .env("ASC_KEY_ID", asc.key_id)
            .env("ASC_ISSUER_ID", asc.issuer_id)
            .env("ASC_PRIVATE_KEY", asc.private_key);
    }

    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Process error: {e}"))?;

    let stdin = child.stdin.take().ok_or("Failed to get stdin")?;
    registry.insert(process_id.clone(), stdin);

    let stdout = child.stdout.take().ok_or("Failed to get stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to get stderr")?;

    tokio::spawn(stream_lines(stdout, app.clone(), event_name.clone(), false));
    tokio::spawn(stream_lines(stderr, app.clone(), event_name.clone(), true));

    let app_wait = app.clone();
    let event_wait = event_name.clone();
    let process_id_wait = process_id.clone();
    tokio::spawn(async move {
        let status = child.wait().await;
        app_wait.state::<DeployRegistry>().remove(&process_id_wait);
        let code = status.ok().and_then(|s| s.code());
        let _ = app_wait.emit(&event_wait, json!({ "type": "done", "success": code == Some(0), "exitCode": code }));
    });

    let _ = app.emit(&event_name, json!({ "type": "started", "processId": process_id }));

    Ok(process_id)
}

/// Equivalent of `POST /api/deploy/input` — writes the 2FA code to the running process's stdin.
#[tauri::command]
pub async fn deploy_submit_two_factor_code(registry: State<'_, DeployRegistry>, process_id: String, code: String) -> Result<(), String> {
    let stdin_arc = registry.get(&process_id).ok_or("Process not found or already finished")?;
    let mut stdin = stdin_arc.lock().await;
    stdin.write_all(format!("{}\n", code.trim()).as_bytes()).await.map_err(|e| e.to_string())?;
    Ok(())
}
