use crate::deploy::find_script_path;
use serde_json::json;
use std::process::Stdio;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
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
        let event_type = if is_error { "error" } else { "log" };
        let _ = app.emit(&event_name, json!({ "type": event_type, "message": line }));
    }
}

/// `POST /api/sync-versions` (SSE) karşılığı — Google Play Developer API + fastlane
/// `fetch_all_versions` lane'i ile mağazadaki gerçek versiyonları çekip yerel
/// `version.json`'ları senkronize eden `scripts/sync_versions.rb`'yi çalıştırır.
#[tauri::command]
pub async fn sync_versions_start(app: AppHandle, workspace_path: String) -> Result<String, String> {
    let script_path = find_script_path(&app, "sync_versions.rb").ok_or("sync_versions.rb bulunamadı")?;

    let job_id = uuid::Uuid::new_v4().to_string();
    let event_name = format!("sync-versions-event-{job_id}");

    let mut child = Command::new("ruby")
        .arg(&script_path)
        .arg(&workspace_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    let stdout = child.stdout.take().ok_or("stdout alınamadı")?;
    let stderr = child.stderr.take().ok_or("stderr alınamadı")?;

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
