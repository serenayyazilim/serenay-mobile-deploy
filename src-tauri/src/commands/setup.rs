use crate::commands::deploy::stream_lines;
use crate::deploy::locales::{get_store_locales, APP_NOT_FOUND};
use crate::setup::{self, Applied, ExpoConfig, ProjectKind, Readiness};
use serde::Serialize;
use serde_json::{json, Value};
use std::path::Path;
use std::process::Stdio;
use tauri::{AppHandle, Emitter};
use tokio::process::Command;

fn kind_of(workspace_path: &str) -> Result<ProjectKind, String> {
    setup::detect_kind(Path::new(workspace_path)).ok_or_else(|| "No supported mobile project found".to_string())
}

#[tauri::command]
pub fn setup_expo_config(workspace_path: String) -> ExpoConfig {
    setup::read_expo_config(Path::new(&workspace_path))
}

/// Writes the identifiers the confirm modal collected to app.json, then runs
/// `expo prebuild`. Output is published via `setup-event-{processId}`, like a deploy's.
#[tauri::command]
pub async fn setup_expo_prebuild(
    app: AppHandle,
    workspace_path: String,
    bundle_identifier: Option<String>,
    package: Option<String>,
) -> Result<String, String> {
    setup::write_expo_ids(Path::new(&workspace_path), bundle_identifier.as_deref(), package.as_deref())?;

    // The confirm modal already asked, so Expo's git-status prompt is skipped and CI=1
    // keeps it from asking anything else.
    let mut child = Command::new("npx")
        .args(["expo", "prebuild"])
        .current_dir(&workspace_path)
        .env("CI", "1")
        .env("EXPO_NO_GIT_STATUS", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Could not run npx: {e}"))?;

    let process_id = uuid::Uuid::new_v4().to_string();
    let event_name = format!("setup-event-{process_id}");
    let stdout = tokio::spawn(stream_lines(child.stdout.take().unwrap(), app.clone(), event_name.clone(), false));
    let stderr = tokio::spawn(stream_lines(child.stderr.take().unwrap(), app.clone(), event_name.clone(), true));

    tokio::spawn(async move {
        let code = child.wait().await.ok().and_then(|s| s.code());
        // Let the last lines (usually the error) reach the modal before it switches state.
        let _ = tokio::join!(stdout, stderr);
        let _ = app.emit(&event_name, json!({ "type": "done", "success": code == Some(0), "exitCode": code }));
    });

    Ok(process_id)
}

#[tauri::command]
pub fn setup_apply(workspace_path: String) -> Result<Vec<Applied>, String> {
    setup::apply(Path::new(&workspace_path), kind_of(&workspace_path)?)
}

#[tauri::command]
pub fn setup_readiness(workspace_path: String) -> Result<Readiness, String> {
    Ok(setup::readiness(Path::new(&workspace_path), kind_of(&workspace_path)?))
}

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
enum StoreRecord {
    Found { locales: Vec<String> },
    Missing,
    Error { message: String },
}

/// Whether the app exists in each store, via the same `fetch_locales` lanes a deploy uses,
/// so a success here also proves the fastlane setup and keys work.
#[tauri::command]
pub async fn setup_store_check(workspace_path: String) -> Result<Value, String> {
    let kind = kind_of(&workspace_path)?;
    let (ios_dir, android_dir) = setup::platform_dirs(Path::new(&workspace_path), kind);
    let (ios, android) = get_store_locales(&workspace_path).await;

    let record = |result: Result<Vec<String>, String>| match result {
        Ok(locales) => StoreRecord::Found { locales },
        Err(e) if e == APP_NOT_FOUND => StoreRecord::Missing,
        Err(message) => StoreRecord::Error { message },
    };
    Ok(json!({
        "ios": ios_dir.map(|_| record(ios)),
        "android": android_dir.map(|_| record(android)),
    }))
}

#[tauri::command]
pub fn setup_set_play_key(workspace_path: String, source_path: String) -> Result<(), String> {
    setup::set_play_key(Path::new(&workspace_path), kind_of(&workspace_path)?, Path::new(&source_path))
}

#[tauri::command]
pub fn setup_set_team_id(workspace_path: String, team_id: String) -> Result<(), String> {
    let root = Path::new(&workspace_path);
    let (ios_dir, _) = setup::platform_dirs(root, kind_of(&workspace_path)?);
    setup::write_team_id(&ios_dir.ok_or("Not an iOS project")?, team_id.trim())
}

/// Builds the release bundle for the first, manual Play Console upload and returns its path.
#[tauri::command]
pub async fn setup_build_aab(workspace_path: String) -> Result<String, String> {
    let root = Path::new(&workspace_path);
    let kind = kind_of(&workspace_path)?;
    let command = setup::aab_build_command(kind).ok_or("Not an Android project")?;

    let output = Command::new("sh")
        .args(["-c", command])
        .current_dir(root)
        .stdin(Stdio::null())
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let log = format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
        let tail: Vec<&str> = log.lines().filter(|l| !l.trim().is_empty()).collect();
        return Err(tail[tail.len().saturating_sub(15)..].join("\n"));
    }

    let aab = setup::aab_path(root, kind).filter(|p| p.exists()).ok_or("Build finished but the .aab file was not found")?;
    Ok(aab.to_string_lossy().to_string())
}
