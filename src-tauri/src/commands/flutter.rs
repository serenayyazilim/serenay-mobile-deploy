use crate::workspace::detect::detect_workspace_mode;
use crate::workspace::sermobileboss_config::{read_sermobileboss_config, MISSING_CONFIG_MESSAGE};
use crate::workspace::types::WorkspaceMode;
use crate::xcode_gradle;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::Path;
use std::process::Stdio;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlutterDevice {
    pub name: String,
    pub id: String,
    pub platform: String,
    #[serde(rename = "type")]
    pub device_type: String,
}

struct DeviceCache {
    devices: Vec<FlutterDevice>,
    cached_at: Instant,
}

static DEVICE_CACHE: Mutex<Option<DeviceCache>> = Mutex::new(None);
const CACHE_TTL: Duration = Duration::from_secs(10);

#[tauri::command]
pub async fn flutter_devices(refresh: bool) -> Result<Vec<FlutterDevice>, String> {
    if !refresh {
        if let Some(cache) = DEVICE_CACHE.lock().unwrap().as_ref() {
            if cache.cached_at.elapsed() < CACHE_TTL {
                return Ok(cache.devices.clone());
            }
        }
    }

    let output = tokio::time::timeout(
        Duration::from_secs(30),
        Command::new("flutter").args(["devices", "--machine"]).output(),
    )
    .await
    .map_err(|_| "Failed to get devices".to_string())?
    .map_err(|e| e.to_string())?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let raw: Vec<serde_json::Value> = serde_json::from_str(&stdout).map_err(|e| e.to_string())?;

    let devices: Vec<FlutterDevice> = raw
        .into_iter()
        .map(|d| {
            let name = d.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let id = d.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let platform = d
                .get("targetPlatform")
                .or_else(|| d.get("platform"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let is_emulator = d.get("emulator").and_then(|v| v.as_bool()).unwrap_or(false);
            let is_supported = d.get("isSupported").and_then(|v| v.as_bool()).unwrap_or(false);
            let device_type = if is_emulator { "simulator" } else if is_supported { "mobile" } else { "desktop" };
            FlutterDevice { name: if name.is_empty() { id.clone() } else { name }, id, platform, device_type: device_type.to_string() }
        })
        .collect();

    *DEVICE_CACHE.lock().unwrap() = Some(DeviceCache { devices: devices.clone(), cached_at: Instant::now() });

    Ok(devices)
}

fn apply_project_setup(workspace_path: &str, project_id: &str, app: &AppHandle, event_name: &str) {
    let root = Path::new(workspace_path);
    let project_folder = root.join("lib/conf/sermobplus-projects").join(project_id);
    let android_folder = root.join("android");

    let log = |msg: String| {
        let _ = app.emit(event_name, json!({ "type": "log", "message": msg }));
    };

    if !project_folder.exists() {
        log(format!("⚠️ Project folder not found: {project_id}"));
        return;
    }

    let Some(boss_config) = read_sermobileboss_config(workspace_path) else {
        log(format!("⚠️ {MISSING_CONFIG_MESSAGE}"));
        return;
    };

    let src_key_props = project_folder.join("key.properties");
    let dest_key_props = android_folder.join("key.properties");
    if src_key_props.exists() && std::fs::copy(&src_key_props, &dest_key_props).is_ok() {
        log(format!("🔑 key.properties updated ({project_id})"));
    } else {
        log(format!("⚠️ key.properties not found: {project_id}"));
    }

    let bundle_name = format!("{}.{}", boss_config.bundle_id_prefix, project_id);
    if xcode_gradle::patch_android_gradle(&android_folder, &bundle_name) {
        log(format!("📦 applicationId updated: {bundle_name}"));
    }

    let ios_folder = root.join("ios");
    if xcode_gradle::patch_pbxproj(&ios_folder, &bundle_name, None) {
        log(format!("🍎 iOS Bundle ID updated: {bundle_name}"));
    }

    let version_json_path = project_folder.join("version.json");
    let pubspec_path = root.join("pubspec.yaml");
    if version_json_path.exists() && pubspec_path.exists() {
        if let Some(version) = std::fs::read_to_string(&version_json_path)
            .ok()
            .and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok())
            .and_then(|v| v.get("version").and_then(|s| s.as_str()).map(String::from))
        {
            if let Ok(mut pubspec_content) = std::fs::read_to_string(&pubspec_path) {
                pubspec_content = Regex::new(r"(?m)^version:\s*.+$")
                    .unwrap()
                    .replace(&pubspec_content, format!("version: {version}"))
                    .to_string();
                let _ = std::fs::write(&pubspec_path, pubspec_content);
                log(format!("🔢 Version set: {version}"));
            }
        }
    }
}

async fn run_logged_command(cmd: &str, args: &[&str], cwd: &str, app: &AppHandle, event_name: &str) -> bool {
    let Ok(mut child) = Command::new(cmd).args(args).current_dir(cwd).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn() else {
        return false;
    };

    if let Some(stdout) = child.stdout.take() {
        let app = app.clone();
        let event_name = event_name.to_string();
        tokio::spawn(async move {
            let mut lines = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if !line.trim().is_empty() {
                    let _ = app.emit(&event_name, json!({ "type": "log", "message": line }));
                }
            }
        });
    }
    if let Some(stderr) = child.stderr.take() {
        let app = app.clone();
        let event_name = event_name.to_string();
        tokio::spawn(async move {
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if !line.trim().is_empty() {
                    let _ = app.emit(&event_name, json!({ "type": "log", "message": format!("⚠️ {line}") }));
                }
            }
        });
    }

    child.wait().await.map(|s| s.success()).unwrap_or(false)
}

/// Equivalent of `POST /api/flutter/build` (SSE) — streams project setup (in sermobileboss
/// mode) + `flutterfire configure` + `flutter pub get` + `flutter run` via the
/// `flutter-build-event-{jobId}` event.
#[tauri::command]
pub async fn flutter_build_start(
    app: AppHandle,
    workspace_path: String,
    device_id: Option<String>,
    project_id: Option<String>,
) -> Result<String, String> {
    let job_id = uuid::Uuid::new_v4().to_string();
    let event_name = format!("flutter-build-event-{job_id}");

    tokio::spawn(async move {
        let emit_log = |app: &AppHandle, msg: &str| {
            let _ = app.emit(&event_name, json!({ "type": "log", "message": msg }));
        };

        if detect_workspace_mode(&workspace_path) == WorkspaceMode::Sermobileboss {
            let active_project_id = project_id.or_else(|| {
                std::fs::read_to_string(Path::new(&workspace_path).join("sermobileboss.txt")).ok().map(|s| s.trim().to_string())
            });
            if let Some(active_project_id) = active_project_id {
                if !active_project_id.is_empty() {
                    emit_log(&app, &format!("⚙️ Setting up project: {active_project_id}"));
                    apply_project_setup(&workspace_path, &active_project_id, &app, &event_name);
                }
            }
        }

        emit_log(&app, "🔥 Starting FlutterFire configure...");
        let configure_ok = run_logged_command("flutterfire", &["configure", "--yes"], &workspace_path, &app, &event_name).await;
        emit_log(&app, if configure_ok { "✅ FlutterFire configure completed" } else { "⚠️ FlutterFire configure skipped (error or already configured)" });

        emit_log(&app, "📦 Installing dependencies...");
        run_logged_command("flutter", &["pub", "get"], &workspace_path, &app, &event_name).await;
        emit_log(&app, "✅ Dependencies installed");

        emit_log(&app, &format!("🚀 Starting app ({})...", device_id.as_deref().unwrap_or("default device")));

        let mut run_args = vec!["run".to_string()];
        if let Some(id) = &device_id {
            run_args.push("-d".to_string());
            run_args.push(id.clone());
        }
        let run_args_ref: Vec<&str> = run_args.iter().map(|s| s.as_str()).collect();

        let Ok(mut child) = Command::new("flutter")
            .args(&run_args_ref)
            .current_dir(&workspace_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        else {
            let _ = app.emit(&event_name, json!({ "type": "done", "success": false, "message": "Failed to start flutter run" }));
            return;
        };

        if let Some(stdout) = child.stdout.take() {
            let app = app.clone();
            let event_name = event_name.clone();
            tokio::spawn(async move {
                let mut lines = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    if !line.trim().is_empty() {
                        let _ = app.emit(&event_name, json!({ "type": "log", "message": line }));
                    }
                }
            });
        }
        if let Some(stderr) = child.stderr.take() {
            let app = app.clone();
            let event_name = event_name.clone();
            tokio::spawn(async move {
                let mut lines = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    if !line.trim().is_empty() {
                        let _ = app.emit(&event_name, json!({ "type": "log", "message": format!("⚠️ {line}") }));
                    }
                }
            });
        }

        let status = child.wait().await;
        let success = status.map(|s| s.success()).unwrap_or(false);
        let message = if success { "App started successfully" } else { "Flutter run failed" };
        let _ = app.emit(&event_name, json!({ "type": "done", "success": success, "message": message }));
    });

    Ok(job_id)
}
