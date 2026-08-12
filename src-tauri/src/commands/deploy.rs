use crate::deploy::registry::DeployRegistry;
use crate::deploy::{find_script_path, is_two_factor_prompt, locales::get_store_locales, translate::build_translations};
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
) -> Result<String, String> {
    if !["ios", "android", "huawei", "all"].contains(&platform.as_str()) {
        return Err("Invalid platform. Must be ios, android, huawei or all".to_string());
    }

    let script_path = find_script_path(&app, "deploy.rb").ok_or("Deploy script not found")?;

    let whats_new_text = whats_new
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "Bug fixes and performance improvements.".to_string());

    let (ios_locales, android_locales) = get_store_locales(&workspace_path).await;
    let translations = build_translations(&whats_new_text, &ios_locales, &android_locales).await;

    let process_id = uuid::Uuid::new_v4().to_string();
    let event_name = format!("deploy-event-{process_id}");

    let mut child = Command::new("ruby")
        .arg(&script_path)
        .arg(&platform)
        .arg(&workspace_path)
        .current_dir(&workspace_path)
        .env("LANG", "en_US.UTF-8")
        .env("WHATS_NEW", &whats_new_text)
        .env("WHATS_NEW_TRANSLATIONS", serde_json::to_string(&translations).unwrap_or_default())
        .env("STORE_LOCALES_IOS", ios_locales.join(","))
        .env("STORE_LOCALES_ANDROID", android_locales.join(","))
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
