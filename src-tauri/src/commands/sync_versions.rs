use crate::appstoreconnect::config::read_asc_config;
use crate::deploy::find_script_path;
use serde_json::json;
use std::process::Stdio;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

/// `ruby sync_versions.rb <workspace>`, with the App Store Connect API key (when set) so the iOS
/// lane signs in with it, as deploys do (the API key is the only way in; no Apple ID, no 2-step code).
fn sync_command(script_path: &std::path::Path, workspace_path: &str, project: Option<&str>) -> Command {
    let mut command = Command::new("ruby");
    command.env("LANG", "en_US.UTF-8").env("LC_ALL", "en_US.UTF-8").arg(script_path).arg(workspace_path);
    if let Some(project) = project {
        command.arg(project).env("SYNC_PROJECT", project); // the iOS lane reads the env var
    }
    if let Some(asc) = read_asc_config(workspace_path) {
        command.env("ASC_KEY_ID", asc.key_id).env("ASC_ISSUER_ID", asc.issuer_id).env("ASC_PRIVATE_KEY", asc.private_key);
    }
    command
}

async fn stream_lines<R: tokio::io::AsyncRead + Unpin>(stream: R, app: AppHandle, event_name: String, is_error: bool) {
    let mut lines = BufReader::new(stream).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        if line.trim().is_empty() {
            continue;
        }
        if !is_error {
            if let Some(json_str) = line.strip_prefix("SYNC_RESULT=") {
                if let Ok(result) = serde_json::from_str::<serde_json::Value>(json_str) {
                    let _ = app.emit(&event_name, json!({ "type": "result", "result": result }));
                    continue;
                }
            }
        }
        let event_type = if is_error { "error" } else { "log" };
        let _ = app.emit(&event_name, json!({ "type": event_type, "message": line }));
    }
}

/// Equivalent of `POST /api/sync-versions` (SSE) — runs `scripts/sync_versions.rb`,
/// which fetches the real store versions via the Google Play Developer API + the fastlane
/// `fetch_all_versions` lane, and syncs them into local `version.json` files.
#[tauri::command]
pub async fn sync_versions_start(app: AppHandle, workspace_path: String) -> Result<String, String> {
    let script_path = find_script_path(&app, "sync_versions.rb").ok_or("sync_versions.rb not found")?;

    let job_id = uuid::Uuid::new_v4().to_string();
    let event_name = format!("sync-versions-event-{job_id}");

    let mut child = sync_command(&script_path, &workspace_path, None)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    let stdout = child.stdout.take().ok_or("Failed to get stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to get stderr")?;

    tokio::spawn(stream_lines(stdout, app.clone(), event_name.clone(), false));
    tokio::spawn(stream_lines(stderr, app.clone(), event_name.clone(), true));

    let app_wait = app.clone();
    let event_wait = event_name.clone();
    tokio::spawn(async move {
        let status = child.wait().await;
        let success = status.map(|s| s.success()).unwrap_or(false);
        let _ = app_wait.emit(&event_wait, json!({ "type": "done", "success": success }));
    });

    Ok(job_id)
}

/// Runs the same sync without a screen (for MCP) and returns its SYNC_RESULT plus the error
/// lines.
pub async fn sync_versions_run(app: &AppHandle, workspace_path: &str, project: Option<&str>) -> Result<serde_json::Value, String> {
    let script_path = find_script_path(app, "sync_versions.rb").ok_or("sync_versions.rb not found")?;
    let mut child = sync_command(&script_path, workspace_path, project)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| e.to_string())?;

    let mut stderr = BufReader::new(child.stderr.take().ok_or("Failed to get stderr")?).lines();
    let errors = tokio::spawn(async move {
        let mut out = Vec::new();
        while let Ok(Some(line)) = stderr.next_line().await {
            if !line.trim().is_empty() {
                out.push(line);
            }
        }
        out
    });

    let mut stdout = BufReader::new(child.stdout.take().ok_or("Failed to get stdout")?).lines();
    let mut result = None;
    let mut warnings = Vec::new(); // per-project "⚠️ id: reason" lines (why a project got no store version)
    let read = async {
        while let Ok(Some(line)) = stdout.next_line().await {
            if let Some(json_str) = line.strip_prefix("SYNC_RESULT=") {
                result = serde_json::from_str::<serde_json::Value>(json_str).ok();
            } else if line.contains("⚠️") {
                warnings.push(line);
            }
        }
        Ok::<(), String>(())
    };
    tokio::time::timeout(std::time::Duration::from_secs(600), read).await.map_err(|_| "Version sync took over 10 minutes; stopped it.".to_string())??;
    let success = child.wait().await.map(|s| s.success()).unwrap_or(false);
    let errors = errors.await.unwrap_or_default();
    match result {
        Some(result) => Ok(json!({ "result": result, "warnings": warnings, "errors": errors })),
        None if success => Ok(json!({ "warnings": warnings, "errors": errors })),
        None => Err(format!("Version sync failed:\n{}", errors.join("\n"))),
    }
}
