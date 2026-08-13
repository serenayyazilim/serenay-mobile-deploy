use crate::deploy::registry::DeployRegistry;
use crate::deploy::{find_script_path, is_two_factor_prompt};
use serde_json::json;
use std::process::Stdio;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

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
        if is_two_factor_prompt(&line) {
            let _ = app.emit(&event_name, json!({ "type": "input_required", "prompt": line }));
            continue;
        }
        let event_type = if is_error { "error" } else { "log" };
        let _ = app.emit(&event_name, json!({ "type": event_type, "message": line }));
    }
}

/// Equivalent of `POST /api/sync-versions` (SSE) — runs `scripts/sync_versions.rb`,
/// which fetches the real store versions via the Google Play Developer API + the fastlane
/// `fetch_all_versions` lane, and syncs them into local `version.json` files.
#[tauri::command]
pub async fn sync_versions_start(app: AppHandle, registry: State<'_, DeployRegistry>, workspace_path: String) -> Result<String, String> {
    let script_path = find_script_path(&app, "sync_versions.rb").ok_or("sync_versions.rb not found")?;

    let job_id = uuid::Uuid::new_v4().to_string();
    let event_name = format!("sync-versions-event-{job_id}");

    let mut child = Command::new("ruby")
        .arg(&script_path)
        .arg(&workspace_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    let stdin = child.stdin.take().ok_or("Failed to get stdin")?;
    registry.insert(job_id.clone(), stdin);

    let stdout = child.stdout.take().ok_or("Failed to get stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to get stderr")?;

    tokio::spawn(stream_lines(stdout, app.clone(), event_name.clone(), false));
    tokio::spawn(stream_lines(stderr, app.clone(), event_name.clone(), true));

    let app_wait = app.clone();
    let event_wait = event_name.clone();
    let job_id_wait = job_id.clone();
    tokio::spawn(async move {
        let status = child.wait().await;
        app_wait.state::<DeployRegistry>().remove(&job_id_wait);
        let success = status.map(|s| s.success()).unwrap_or(false);
        let _ = app_wait.emit(&event_wait, json!({ "type": "done", "success": success }));
    });

    Ok(job_id)
}

/// Writes the 2FA code to the running sync process's stdin (forwarded to the fastlane
/// `fetch_all_versions` lane, which prompts for it when Apple requires 2-step verification).
#[tauri::command]
pub async fn sync_versions_submit_two_factor_code(registry: State<'_, DeployRegistry>, job_id: String, code: String) -> Result<(), String> {
    let stdin_arc = registry.get(&job_id).ok_or("Process not found or already finished")?;
    let mut stdin = stdin_arc.lock().await;
    stdin.write_all(format!("{}\n", code.trim()).as_bytes()).await.map_err(|e| e.to_string())?;
    Ok(())
}
