use crate::deploy::registry::DeployRegistry;
use crate::workspace::detect::detect_workspace_mode;
use crate::workspace::sermobileboss_config::{read_sermobileboss_config, MISSING_CONFIG_MESSAGE};
use crate::workspace::types::WorkspaceMode;
use crate::setup::{self, ProjectKind};
use crate::xcode_gradle;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::Path;
use std::process::Stdio;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlutterDevice {
    pub name: String,
    pub id: String,
    pub platform: String,
    #[serde(rename = "type")]
    pub device_type: String,
}

const FLUTTER_NOT_FOUND_MESSAGE: &str =
    "Flutter SDK not found. Install it from https://flutter.dev/docs/get-started/install and make sure `flutter` is on your PATH, then restart this app.";

fn flutter_not_found_message(e: &std::io::Error) -> String {
    if e.kind() == std::io::ErrorKind::NotFound {
        FLUTTER_NOT_FOUND_MESSAGE.to_string()
    } else {
        e.to_string()
    }
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
    .map_err(|e| flutter_not_found_message(&e))?;

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
    device_platform: Option<String>,
    project_id: Option<String>,
) -> Result<String, String> {
    let root = Path::new(&workspace_path);
    let react_native = matches!(setup::detect_kind(root), Some(ProjectKind::ReactNative | ProjectKind::Expo));
    let (program, run_args) = if react_native {
        ("npx", react_native_run_args(root, device_id.as_deref(), device_platform.as_deref())?)
    } else {
        let mut args = vec!["run".to_string()];
        if let Some(id) = &device_id {
            args.extend(["-d".to_string(), id.clone()]);
        }
        ("flutter", args)
    };

    let job_id = uuid::Uuid::new_v4().to_string();
    let event_name = format!("flutter-build-event-{job_id}");
    let return_job_id = job_id.clone();

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

        if !react_native {
            emit_log(&app, "🔥 Starting FlutterFire configure...");
            let configure_ok = run_logged_command("flutterfire", &["configure", "--yes"], &workspace_path, &app, &event_name).await;
            emit_log(&app, if configure_ok { "✅ FlutterFire configure completed" } else { "⚠️ FlutterFire configure skipped (error or already configured)" });

            emit_log(&app, "📦 Installing dependencies...");
            run_logged_command("flutter", &["pub", "get"], &workspace_path, &app, &event_name).await;
            emit_log(&app, "✅ Dependencies installed");
        }

        emit_log(&app, &format!("🚀 Starting app ({})...", device_id.as_deref().unwrap_or("default device")));

        emit_log(&app, &format!("$ {program} {}", run_args.join(" ")));
        let run_args_ref: Vec<&str> = run_args.iter().map(|s| s.as_str()).collect();

        let mut command = Command::new(program);
        command.args(&run_args_ref).current_dir(&workspace_path).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
        // Own process group so a stop can take down Metro and the build tools with it.
        #[cfg(unix)]
        if react_native {
            command.process_group(0);
        }
        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(e) => {
                let _ = app.emit(&event_name, json!({ "type": "done", "success": false, "message": flutter_not_found_message(&e) }));
                return;
            }
        };

        if let Some(stdin) = child.stdin.take() {
            app.state::<DeployRegistry>().insert(job_id.clone(), stdin);
        }
        if react_native {
            if let Some(pid) = child.id() {
                react_native_runs().lock().unwrap().insert(job_id.clone(), pid);
            }
        }

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
        app.state::<DeployRegistry>().remove(&job_id);
        // Removed from the map only by `flutter_run_stop`, so a missing entry means a user stop.
        let stopped = react_native && react_native_runs().lock().unwrap().remove(&job_id).is_none();
        let success = stopped || status.map(|s| s.success()).unwrap_or(false);
        let message = match (success, react_native) {
            _ if stopped => "App stopped",
            (true, _) => "App started successfully",
            (false, true) => "React Native run failed",
            (false, false) => "Flutter run failed",
        };
        let _ = app.emit(&event_name, json!({ "type": "done", "success": success, "message": message }));
    });

    Ok(return_job_id)
}

/// `run-ios` / `run-android` for the chosen device. The device list comes from
/// `flutter devices`, whose IDs are simulator UDIDs and adb serials — what these take too.
fn react_native_run_args(root: &Path, device_id: Option<&str>, device_platform: Option<&str>) -> Result<Vec<String>, String> {
    let platform = device_platform.unwrap_or_default().to_lowercase();
    let ios = if platform.starts_with("ios") {
        true
    } else if platform.starts_with("android") {
        false
    } else {
        return Err("Choose an iOS or Android device to run a React Native app".to_string());
    };

    let mut args: Vec<String> = if setup::is_expo(root) {
        vec!["expo".into(), if ios { "run:ios" } else { "run:android" }.into()]
    } else {
        vec!["react-native".into(), if ios { "run-ios" } else { "run-android" }.into()]
    };
    if let Some(id) = device_id {
        let flag = match (setup::is_expo(root), ios) {
            (true, _) => "--device",
            (false, true) => "--udid",
            (false, false) => "--deviceId",
        };
        args.extend([flag.to_string(), id.to_string()]);
    }
    Ok(args)
}

/// Sends `r` (hot reload) or `R` (hot restart) to a running `flutter run` process's stdin.
#[tauri::command]
pub async fn flutter_run_hot_reload(registry: State<'_, DeployRegistry>, job_id: String, restart: bool) -> Result<(), String> {
    let stdin_arc = registry.get(&job_id).ok_or("App is not running")?;
    let mut stdin = stdin_arc.lock().await;
    stdin.write_all(if restart { b"R" } else { b"r" }).await.map_err(|e| e.to_string())?;
    stdin.flush().await.map_err(|e| e.to_string())?;
    Ok(())
}

/// Process group IDs of running React Native / Expo runs, by job ID.
fn react_native_runs() -> &'static Mutex<HashMap<String, u32>> {
    static RUNS: OnceLock<Mutex<HashMap<String, u32>>> = OnceLock::new();
    RUNS.get_or_init(Default::default)
}

/// Sends `q` (quit) to a running `flutter run` process's stdin to stop it gracefully.
/// React Native / Expo CLIs ignore stdin when it isn't a TTY, so their process group is terminated instead.
#[tauri::command]
pub async fn flutter_run_stop(registry: State<'_, DeployRegistry>, job_id: String) -> Result<(), String> {
    if let Some(pid) = react_native_runs().lock().unwrap().remove(&job_id) {
        let status = std::process::Command::new("kill").args(["-TERM", &format!("-{pid}")]).status().map_err(|e| e.to_string())?;
        return if status.success() { Ok(()) } else { Err("Could not stop the app".to_string()) };
    }
    let stdin_arc = registry.get(&job_id).ok_or("App is not running")?;
    let mut stdin = stdin_arc.lock().await;
    stdin.write_all(b"q").await.map_err(|e| e.to_string())?;
    stdin.flush().await.map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::react_native_run_args;

    #[test]
    fn picks_react_native_or_expo_run_command_per_device() {
        let root = std::env::temp_dir().join(format!("rn-run-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("package.json"), r#"{"dependencies":{"react-native":"0.76.0"}}"#).unwrap();
        assert_eq!(react_native_run_args(&root, Some("UDID"), Some("ios")).unwrap(), ["react-native", "run-ios", "--udid", "UDID"]);
        assert_eq!(react_native_run_args(&root, Some("emulator-5554"), Some("android-arm64")).unwrap(), ["react-native", "run-android", "--deviceId", "emulator-5554"]);
        assert!(react_native_run_args(&root, Some("macos"), Some("darwin")).is_err());

        std::fs::write(root.join("package.json"), r#"{"dependencies":{"expo":"~51.0.0","react-native":"0.74.0"}}"#).unwrap();
        assert_eq!(react_native_run_args(&root, Some("UDID"), Some("ios")).unwrap(), ["expo", "run:ios", "--device", "UDID"]);
    }
}
