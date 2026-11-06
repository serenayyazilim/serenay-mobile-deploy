//! An MCP server (streamable HTTP, JSON responses) so an AI assistant can use the
//! app: deploy, run and test apps, change versions and settings, check store setup, manage App
//! Store events and more. Jobs with a screen of their own (deploy, run, tests, switching the
//! workspace) are handed to the frontend and run exactly as if their button was pressed; the
//! frontend reports its state back through `mcp_status_set`, which is also where the open
//! workspace comes from. Everything else calls the Tauri commands directly.

use crate::commands::{appstoreconnect as asc, config, firebase, flutter, git, project, projects, sentry, setup, slack, store_locales, workspace};
use crate::workspace::types::WorkspaceProject;
use serde::Serialize;
use serde_json::{json, Map, Value};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub const PORT: u16 = 47321;
const MAX_REQUEST: usize = 1 << 20;

static STATUS: Mutex<Value> = Mutex::new(Value::Null);
static LOGS_REPLY: Mutex<Option<tokio::sync::oneshot::Sender<Value>>> = Mutex::new(None);

/// The frontend's open workspace and the state of its deploy, run and test jobs, pushed on every change.
#[tauri::command]
pub fn mcp_status_set(status: Value) {
    *STATUS.lock().unwrap() = status;
}

/// The bearer token other devices on the network must send, made once and kept in
/// `~/Library/Application Support/com.serenaymobiledeploy.app/mcp-token`. This Mac needs none.
fn lan_token() -> Option<String> {
    let path = dirs::config_dir()?.join("com.serenaymobiledeploy.app").join("mcp-token");
    if let Ok(token) = std::fs::read_to_string(&path) {
        return Some(token.trim().to_string()).filter(|t| !t.is_empty());
    }
    let token = format!("{}{}", uuid::Uuid::new_v4().simple(), uuid::Uuid::new_v4().simple());
    std::fs::create_dir_all(path.parent()?).ok()?;
    std::fs::write(&path, &token).ok()?;
    Some(token)
}

/// The frontend's answer to `mcp-logs-request`: every log line of the deploy, run and tests.
#[tauri::command]
pub fn mcp_logs_reply(logs: Value) {
    if let Some(reply) = LOGS_REPLY.lock().unwrap().take() {
        let _ = reply.send(logs);
    }
}

pub fn start(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        // Without a token, stay on this Mac only.
        let token = lan_token();
        let host = if token.is_some() { "0.0.0.0" } else { "127.0.0.1" };
        let listener = match TcpListener::bind((host, PORT)).await {
            Ok(l) => l,
            Err(e) => return eprintln!("MCP server could not listen on {PORT}: {e}"),
        };
        while let Ok((stream, peer)) = listener.accept().await {
            let needed = if peer.ip().is_loopback() { None } else { token.clone() };
            tokio::spawn(serve(stream, app.clone(), needed));
        }
    });
}

async fn serve(mut stream: TcpStream, app: AppHandle, token: Option<String>) {
    let (status, body) = match read_request(&mut stream, token.as_deref()).await {
        Ok(body) => match serde_json::from_slice::<Value>(&body) {
            Ok(message) => match handle(&app, &message).await {
                Some(response) => ("200 OK", response.to_string()),
                None => ("202 Accepted", String::new()),
            },
            Err(e) => ("400 Bad Request", rpc_error(Value::Null, -32700, &e.to_string()).to_string()),
        },
        Err(status) => (status, String::new()),
    };
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes()).await;
}

/// Returns the body of a `POST` with a JSON content type and no `Origin`. Both checks keep web
/// pages out: a browser always sends `Origin` on such a request, and a JSON content type would
/// need a CORS preflight this server never answers. With a `token`, it must also come as
/// `Authorization: Bearer <token>`.
async fn read_request(stream: &mut TcpStream, token: Option<&str>) -> Result<Vec<u8>, &'static str> {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 8192];
    let header_end = loop {
        let n = stream.read(&mut chunk).await.map_err(|_| "400 Bad Request")?;
        if n == 0 {
            return Err("400 Bad Request");
        }
        buf.extend_from_slice(&chunk[..n]);
        if let Some(i) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            break i + 4;
        }
        if buf.len() > MAX_REQUEST {
            return Err("413 Payload Too Large");
        }
    };

    let head = String::from_utf8_lossy(&buf[..header_end]).to_lowercase();
    if !head.starts_with("post ") {
        return Err("405 Method Not Allowed");
    }
    let header = |name: &str| head.lines().find_map(|l| l.strip_prefix(name).map(|v| v.trim().to_string()));
    if header("origin:").is_some() {
        return Err("403 Forbidden");
    }
    if let Some(token) = token {
        if header("authorization:").as_deref() != Some(&format!("bearer {}", token.to_lowercase())) {
            return Err("401 Unauthorized");
        }
    }
    if !header("content-type:").is_some_and(|t| t.starts_with("application/json")) {
        return Err("415 Unsupported Media Type");
    }
    let length: usize = header("content-length:").and_then(|v| v.parse().ok()).ok_or("411 Length Required")?;
    if length > MAX_REQUEST {
        return Err("413 Payload Too Large");
    }

    let mut body = buf.split_off(header_end);
    while body.len() < length {
        let n = stream.read(&mut chunk).await.map_err(|_| "400 Bad Request")?;
        if n == 0 {
            return Err("400 Bad Request");
        }
        body.extend_from_slice(&chunk[..n]);
    }
    body.truncate(length);
    Ok(body)
}

fn rpc_error(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

/// Answers one JSON-RPC message; notifications (no `id`) get no answer.
async fn handle(app: &AppHandle, message: &Value) -> Option<Value> {
    let id = message.get("id")?.clone();
    let params = message.get("params").cloned().unwrap_or(Value::Null);
    let result = match message.get("method").and_then(Value::as_str).unwrap_or("") {
        "initialize" => json!({
            "protocolVersion": params.get("protocolVersion").cloned().unwrap_or(json!("2025-06-18")),
            "capabilities": { "tools": {} },
            "serverInfo": { "name": "serenay-mobile-deploy", "version": env!("CARGO_PKG_VERSION") },
        }),
        "ping" => json!({}),
        "tools/list" => json!({ "tools": tools() }),
        "tools/call" => {
            let args = params.get("arguments").cloned().unwrap_or(json!({}));
            let (text, is_error) = match call_tool(app, params.get("name").and_then(Value::as_str).unwrap_or(""), &args).await {
                Ok(Value::String(text)) => (text, false),
                Ok(value) => (serde_json::to_string_pretty(&value).unwrap_or_default(), false),
                Err(text) => (text, true),
            };
            json!({ "content": [{ "type": "text", "text": text }], "isError": is_error })
        }
        method => return Some(rpc_error(id, -32601, &format!("Unknown method: {method}"))),
    };
    Some(json!({ "jsonrpc": "2.0", "id": id, "result": result }))
}

// ============ TOOLS ============

/// A parameter: name, JSON type, required, description.
type Param = (&'static str, &'static str, bool, &'static str);

const PROJECT: Param = ("project", "string", false, "Project id or app name from list_projects; may be left out when the workspace has one app");

#[rustfmt::skip]
const TOOLS: &[(&str, &str, &[Param])] = &[
    // Workspace
    ("status", "The open workspace and the state of the deploy, the app run and the tests: status, progress, message, latest log lines, test results, and whether a 2FA code is awaited.", &[]),
    ("logs", "Every log line of the deploy, the app run or the tests (the last failed one's, once the app has cleared them), to find out why something failed. Lines are numbered; without contains or tail, the last 500 come back. With path it reads a log file under ~/Library/Logs instead, such as the gym log fastlane names when an iOS build fails, which holds the full xcodebuild output.", &[
        ("job", "string", false, "deploy (default), run or tests"),
        ("contains", "string", false, "Keep only lines containing this, ignoring case; separate alternatives with |, e.g. error|failed|warning"),
        ("tail", "integer", false, "Keep only the last this many lines (after contains)"),
        ("path", "string", false, "A log file under ~/Library/Logs to read instead of the app's logs"),
    ]),
    ("list_workspaces", "Workspaces opened before in the app, which open_workspace can switch to.", &[]),
    ("open_workspace", "Switches the app to a workspace opened before (see list_workspaces).", &[("path", "string", true, "Workspace folder")]),
    ("git_status", "The workspace's git branch, its branches (local and remote), how many files have uncommitted changes, and commits ahead of / behind the remote as of the last fetch.", &[]),
    ("git_switch", "Switches the workspace to a branch with its latest commits: fetches, puts uncommitted changes aside in a git stash (the app's project setup changes many files; git stash pop brings them back), switches, pulls (fast-forward only) and runs flutter pub get. pod install runs on the next deploy. Takes up to a few minutes.", &[("branch", "string", true, "Branch name, e.g. main or develop")]),
    ("list_projects", "Projects in the open workspace with their bundle ids and versions (x.y.z+build).", &[]),

    // Deploy, run, test (shown in the app)
    ("deploy", "Starts a store deploy, the same as the deploy button. Returns right away; follow it with status.", &[
        PROJECT,
        ("platform", "string", false, "ios, android or all (default all)"),
        ("track", "string", false, "production (App Store / Google Play, default) or test (TestFlight / internal testing)"),
        ("whats_new", "string", false, "Release notes in Turkish; they are translated to the store's other languages"),
        ("bump_version", "boolean", false, "Bump the patch version and build number first (default true)"),
    ]),
    ("list_devices", "Simulators, emulators and connected devices the app can run on.", &[("refresh", "boolean", false, "Look again instead of using the cached list")]),
    ("run_app", "Runs the app on a device in debug mode, the same as the start button. Follow it with status.", &[PROJECT, ("device", "string", false, "Device id or name from list_devices; left out = Flutter's default")]),
    ("hot_reload", "Hot reloads the running app.", &[]),
    ("hot_restart", "Hot restarts the running app.", &[]),
    ("stop_app", "Stops the running app.", &[]),
    ("run_tests", "Runs the project's tests. Follow it with status for the results.", &[PROJECT]),

    // Version and project settings
    ("set_version", "Writes the app version without deploying.", &[PROJECT, ("version", "string", true, "x.y.z+build, e.g. 3.0.2+44; the build number must stay above the stores' latest")]),
    ("rename_project", "Renames an app in a multi-app workspace.", &[PROJECT, ("app_name", "string", true, "New app name")]),
    ("get_project_config", "The app's settings (serconf: API URL, contact URL, languages, feature flags...).", &[PROJECT]),
    ("update_project_config", "Changes some of the app's settings; keys left out keep their value.", &[PROJECT, ("config", "object", true, "Keys and values to change, as get_project_config shows them")]),
    ("get_colors", "The app's theme colors.", &[PROJECT]),
    ("update_colors", "Changes some of the app's theme colors; colors left out keep their value.", &[PROJECT, ("colors", "object", true, "Color name → hex value, e.g. {\"primary\": \"#FF0000\"}")]),
    ("get_splash_color", "The splash screen background color.", &[PROJECT]),
    ("set_splash_color", "Sets the splash screen background color.", &[PROJECT, ("color", "string", true, "Hex color, e.g. #FFFFFF")]),
    ("set_project_asset", "Copies a file into the app as one of its assets.", &[
        PROJECT,
        ("asset_type", "string", true, "Multi-app workspace: logo, splash, icon512, icon1024, firebaseAndroid, firebaseIos. Single app: icon, splash, firebaseAndroid, firebaseIos"),
        ("file_path", "string", true, "Absolute path of the file to copy"),
    ]),
    ("check_splash_image", "Whether the app has a splash image (without one the deploy skips the splash screen).", &[PROJECT]),
    ("create_project", "Creates a new app in a multi-app workspace.", &[
        ("projectId", "string", true, "New project id (the bundle id's last part)"),
        ("appName", "string", true, "App name"),
        ("bgColor", "string", false, "Splash background color"),
        ("icon512Path", "string", false, "512x512 icon file"),
        ("icon1024Path", "string", false, "1024x1024 icon file"),
        ("appLogoPath", "string", false, "Logo file"),
        ("splashPath", "string", false, "Splash image file"),
        ("serconf", "object", false, "Initial settings"),
        ("firebaseAndroidPath", "string", false, "google-services.json"),
        ("firebaseIosPath", "string", false, "GoogleService-Info.plist"),
        ("sentryDsn", "string", false, "Sentry DSN"),
    ]),

    // Store setup
    ("setup_readiness", "What the open single-app project still needs for store deploys (fastlane, signing, keys, team id...).", &[]),
    ("setup_apply", "Creates the missing fastlane setup and signing config of the open single-app project.", &[]),
    ("complete_fastlane", "Adds the lanes and Appfile settings an existing fastlane setup lacks, without touching the rest.", &[]),
    ("set_team_id", "Sets the Apple Developer Team ID of the open single-app project.", &[("team_id", "string", true, "10-character team id")]),
    ("set_play_key", "Copies a Google Play service account JSON key into the project's fastlane folder.", &[("source_path", "string", true, "Absolute path of the JSON key")]),
    ("store_check", "Whether the app exists on the App Store and Google Play, through the same fastlane lanes a deploy uses (proves keys and setup work).", &[]),
    ("store_locales", "The languages the app's store listings have.", &[]),
    ("build_aab", "Builds a release Android App Bundle, for the first manual upload to Google Play. Returns its path.", &[]),

    // App Store Connect
    ("asc_status", "Whether an App Store Connect API key is set for the workspace.", &[]),
    ("asc_apps", "The apps in App Store Connect, with their ids.", &[]),
    ("asc_territories", "App Store territory codes.", &[]),
    ("asc_events_list", "The app's In-App Events.", &[PROJECT]),
    ("asc_event_get", "One In-App Event with its localizations.", &[("id", "string", true, "Event id")]),
    ("asc_event_create", "Creates an In-App Event.", &[PROJECT, ("attributes", "object", true, "App Store Connect appEvents attributes (referenceName, badge, deepLink, purpose, territorySchedules...)"), ("primary_localization", "object", false, "First localization's attributes (locale, name, shortDescription, longDescription)")]),
    ("asc_event_update", "Changes an In-App Event.", &[("id", "string", true, "Event id"), ("attributes", "object", true, "Attributes to change")]),
    ("asc_event_delete", "Deletes an In-App Event.", &[("id", "string", true, "Event id")]),
    ("asc_event_submit", "Submits an In-App Event for review.", &[("id", "string", true, "Event id"), ("app_id", "string", true, "App id from asc_apps")]),
    ("asc_localization_create", "Adds a language to an In-App Event.", &[("event_id", "string", true, "Event id"), ("attributes", "object", true, "locale, name, shortDescription, longDescription")]),
    ("asc_localization_update", "Changes an In-App Event localization.", &[("id", "string", true, "Localization id"), ("attributes", "object", true, "Attributes to change")]),
    ("asc_localization_delete", "Deletes an In-App Event localization.", &[("id", "string", true, "Localization id")]),
    ("asc_screenshots", "Images of an In-App Event localization.", &[("localization_id", "string", true, "Localization id")]),
    ("asc_screenshot_upload", "Uploads an In-App Event image.", &[("localization_id", "string", true, "Localization id"), ("asset_type", "string", true, "EVENT_CARD or EVENT_DETAILS_PAGE"), ("file_path", "string", true, "Absolute path of the image")]),
    ("asc_screenshot_delete", "Deletes an In-App Event image.", &[("id", "string", true, "Image id")]),

    // Integrations
    ("firebase_accounts", "Firebase CLI sign-in status.", &[]),
    ("firebase_create_project", "Creates a Firebase project with Android and iOS apps for the app and saves their config files.", &[PROJECT]),
    ("sentry_check", "Sentry token and organizations.", &[]),
    ("sentry_create_project", "Creates a Sentry project for the app.", &[PROJECT, ("org_slug", "string", true, "Organization slug from sentry_check")]),
    ("slack_test", "Sends a test message to the workspace's Slack webhook.", &[]),
];

fn tools() -> Value {
    TOOLS
        .iter()
        .map(|(name, description, params)| {
            let properties: Map<String, Value> = params.iter().map(|(n, t, _, d)| (n.to_string(), json!({ "type": t, "description": d }))).collect();
            let required: Vec<&str> = params.iter().filter(|p| p.2).map(|p| p.0).collect();
            json!({ "name": name, "description": description, "inputSchema": { "type": "object", "properties": properties, "required": required } })
        })
        .collect()
}

fn out<T: Serialize>(value: T) -> Result<Value, String> {
    serde_json::to_value(value).map_err(|e| e.to_string())
}

fn text(args: &Value, key: &str) -> Option<String> {
    args.get(key).and_then(Value::as_str).map(str::to_string)
}

fn required(args: &Value, key: &str) -> Result<String, String> {
    text(args, key).filter(|s| !s.is_empty()).ok_or(format!("{key} is required"))
}

fn object(args: &Value, key: &str) -> Result<Value, String> {
    args.get(key).filter(|v| v.is_object()).cloned().ok_or(format!("{key} must be an object"))
}

/// The project the `project` argument names, or the only one when it's left out.
fn find_project(workspace: &str, args: &Value) -> Result<WorkspaceProject, String> {
    let list = projects::projects_list(workspace.to_string())?;
    match text(args, "project").map(|p| p.to_lowercase()) {
        Some(wanted) => list
            .into_iter()
            .find(|p| p.id.to_lowercase() == wanted || p.app_name.to_lowercase() == wanted)
            .ok_or(format!("No project \"{wanted}\" in the open workspace. Use list_projects for the ids.")),
        None if list.len() == 1 => Ok(list.into_iter().next().unwrap()),
        None => Err("This workspace has several apps; say which one with project.".to_string()),
    }
}

/// Merges `changes` into `current`, so keys left out keep their value.
fn merged<V: Clone + serde::de::DeserializeOwned>(mut current: std::collections::BTreeMap<String, V>, changes: Value) -> Result<std::collections::BTreeMap<String, V>, String> {
    let changes: std::collections::BTreeMap<String, V> = serde_json::from_value(changes).map_err(|e| e.to_string())?;
    current.extend(changes);
    Ok(current)
}

async fn logs(app: &AppHandle, args: &Value) -> Result<String, String> {
    let lines: Vec<String> = if let Some(path) = text(args, "path") {
        let base = dirs::home_dir().ok_or("No home folder")?.join("Library/Logs");
        let path = std::fs::canonicalize(&path).map_err(|e| format!("{path}: {e}"))?;
        if !path.starts_with(std::fs::canonicalize(&base).unwrap_or(base)) {
            return Err("Only files under ~/Library/Logs can be read".to_string());
        }
        let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
        String::from_utf8_lossy(&bytes).lines().map(str::to_string).collect()
    } else {
        let job = text(args, "job").unwrap_or_else(|| "deploy".to_string());
        let (reply, answer) = tokio::sync::oneshot::channel();
        *LOGS_REPLY.lock().unwrap() = Some(reply);
        app.emit("mcp-logs-request", ()).map_err(|e| e.to_string())?;
        let all = tokio::time::timeout(std::time::Duration::from_secs(5), answer)
            .await
            .map_err(|_| "The app did not answer; open a workspace in it.".to_string())?
            .map_err(|_| "The app did not answer.".to_string())?;
        let lines = all.get(&job).and_then(Value::as_array).ok_or(format!("Unknown job \"{job}\"; use deploy, run or tests."))?;
        lines.iter().filter_map(Value::as_str).map(str::to_string).collect()
    };
    let tail = args.get("tail").and_then(Value::as_u64).map(|n| n as usize);
    Ok(pick_lines(&lines, text(args, "contains").as_deref(), tail))
}

/// Numbers the lines, drops color codes, keeps those containing any of the `|`-separated
/// `contains` alternatives, then the last `tail` (500 when neither is given).
fn pick_lines(lines: &[String], contains: Option<&str>, tail: Option<usize>) -> String {
    let ansi = regex::Regex::new(r"\x1b\[[0-9;]*[A-Za-z]").unwrap();
    let wanted: Vec<String> = contains.map(|c| c.split('|').map(|w| w.trim().to_lowercase()).filter(|w| !w.is_empty()).collect()).unwrap_or_default();
    let kept: Vec<String> = lines
        .iter()
        .enumerate()
        .map(|(i, line)| (i + 1, ansi.replace_all(line, "")))
        .filter(|(_, line)| wanted.is_empty() || wanted.iter().any(|w| line.to_lowercase().contains(w)))
        .map(|(n, line)| format!("{n}: {line}"))
        .collect();
    let tail = tail.or(if wanted.is_empty() { Some(500) } else { None }).unwrap_or(kept.len());
    let shown = &kept[kept.len().saturating_sub(tail)..];
    if lines.is_empty() {
        return "No log lines.".to_string();
    }
    format!("{} lines in all, {} matching, showing the last {}:\n{}", lines.len(), kept.len(), shown.len(), shown.join("\n"))
}

fn running(status: &Value, job: &str) -> bool {
    matches!(status.pointer(&format!("/{job}/status")).and_then(Value::as_str), Some("activating" | "deploying" | "running"))
}

/// Hands a job to the frontend, which runs it like its button was pressed.
fn ui(app: &AppHandle, action: Value, started: String) -> Result<Value, String> {
    app.emit("mcp-action", action).map_err(|e| e.to_string())?;
    Ok(Value::String(started))
}

async fn call_tool(app: &AppHandle, name: &str, args: &Value) -> Result<Value, String> {
    let status = STATUS.lock().unwrap().clone();
    match name {
        "status" => return Ok(status),
        "logs" => return logs(app, args).await.map(Value::String),
        "list_workspaces" => return out(workspace::workspace_recent_get()),
        "open_workspace" => {
            let path = required(args, "path")?;
            if !workspace::workspace_recent_get().iter().any(|w| w.path == path) {
                return Err("Only workspaces opened before can be switched to (see list_workspaces); open a new one in the app once.".to_string());
            }
            let mode = workspace::workspace_validate(path.clone()).mode.ok_or("This folder is no longer a valid workspace")?;
            return ui(app, json!({ "action": "openWorkspace", "path": path, "mode": mode }), format!("Switched to {path}."));
        }
        "list_devices" => return out(flutter::flutter_devices(args.get("refresh").and_then(Value::as_bool).unwrap_or(false)).await?),
        "firebase_accounts" => return out(firebase::firebase_accounts().await),
        "sentry_check" => return out(sentry::sentry_check().await),
        _ => {}
    }

    let ws = status
        .get("workspace")
        .and_then(Value::as_str)
        .filter(|w| !w.is_empty())
        .ok_or("No workspace is open in Serenay Mobile Deploy. Open the app and pick a workspace first.")?
        .to_string();
    let project = {
        let ws = ws.clone();
        move || find_project(&ws, args)
    };

    match name {
        "list_projects" => {
            let versions = projects::projects_versions(ws.clone()).versions;
            let list: Vec<Value> = projects::projects_list(ws.clone())?
                .into_iter()
                .map(|p| json!({ "id": p.id, "appName": p.app_name, "bundleId": p.bundle_id, "kind": p.kind, "version": versions.get(&p.id) }))
                .collect();
            Ok(json!({ "workspace": ws, "projects": list }))
        }

        "git_status" => out(git::git_status(ws).await?),
        "git_switch" => {
            if running(&status, "deploy") || running(&status, "run") || running(&status, "tests") {
                return Err("A deploy, run or test is going on; switch branches when it's done.".to_string());
            }
            out(git::git_switch(app.clone(), ws, required(args, "branch")?).await?)
        }

        "deploy" => {
            if running(&status, "deploy") {
                return Err("A deploy is already running. Check it with status.".to_string());
            }
            let p = project()?;
            let (platform, track) = (text(args, "platform").unwrap_or("all".into()), text(args, "track").unwrap_or("production".into()));
            if !["ios", "android", "all"].contains(&platform.as_str()) {
                return Err("platform must be ios, android or all".to_string());
            }
            if !["production", "test"].contains(&track.as_str()) {
                return Err("track must be production or test".to_string());
            }
            let action = json!({
                "action": "deploy", "projectId": p.id, "platform": platform, "track": track,
                "whatsNew": text(args, "whats_new").unwrap_or_default(),
                "bumpVersion": args.get("bump_version").and_then(Value::as_bool).unwrap_or(true),
            });
            ui(app, action, format!("Deploy of {} ({platform}, {track}) started. Follow it with status.", p.app_name))
        }
        "run_app" => {
            if running(&status, "run") {
                return Err("An app is already running; stop it with stop_app first.".to_string());
            }
            let p = project()?;
            let device = match text(args, "device") {
                Some(wanted) => Some(
                    flutter::flutter_devices(false)
                        .await?
                        .into_iter()
                        .find(|d| d.id == wanted || d.name.eq_ignore_ascii_case(&wanted))
                        .ok_or(format!("No device \"{wanted}\"; see list_devices"))?,
                ),
                None => None,
            };
            let on = device.as_ref().map(|d| format!(" on {}", d.name)).unwrap_or_default();
            ui(app, json!({ "action": "run", "projectId": p.id, "device": device }), format!("Starting {}{on}. Follow it with status.", p.app_name))
        }
        "hot_reload" | "hot_restart" | "stop_app" => {
            if !running(&status, "run") {
                return Err("No app is running.".to_string());
            }
            let action = match name { "hot_reload" => "hotReload", "hot_restart" => "hotRestart", _ => "stop" };
            ui(app, json!({ "action": action }), format!("{name} sent."))
        }
        "run_tests" => {
            if running(&status, "tests") {
                return Err("Tests are already running. Check them with status.".to_string());
            }
            let p = project()?;
            ui(app, json!({ "action": "tests", "projectId": p.id }), format!("Running the tests of {}. Follow them with status.", p.app_name))
        }

        "set_version" => out(projects::projects_version_set(ws, project()?.id, required(args, "version")?)?),
        "rename_project" => out(projects::projects_rename(ws, project()?.id, required(args, "app_name")?)?),
        "get_project_config" => out(config::config_serconf_get(ws, project()?.id)?),
        "update_project_config" => {
            let id = project()?.id;
            let config = merged(config::config_serconf_get(ws.clone(), id.clone())?, object(args, "config")?)?;
            config::config_serconf_save(ws, id, config.clone())?;
            out(config)
        }
        "get_colors" => out(config::config_colors_get(ws, project()?.id)),
        "update_colors" => {
            let id = project()?.id;
            let colors = merged(config::config_colors_get(ws.clone(), id.clone()), object(args, "colors")?)?;
            config::config_colors_save(ws, id, colors.clone())?;
            out(colors)
        }
        "get_splash_color" => out(projects::project_splash_color_get(ws, project()?.id)),
        "set_splash_color" => out(projects::project_splash_color_save(ws, project()?.id, required(args, "color")?)?),
        "set_project_asset" => {
            let data = std::fs::read(required(args, "file_path")?).map_err(|e| e.to_string())?;
            out(projects::project_asset_upload(ws, project()?.id, required(args, "asset_type")?, data)?)
        }
        "check_splash_image" => out(project::project_check_splash_image(ws, project()?.id)),
        "create_project" => {
            let mut params = args.clone();
            params["workspacePath"] = Value::String(ws);
            out(project::project_create(serde_json::from_value(params).map_err(|e| e.to_string())?).await?)
        }

        "setup_readiness" => out(setup::setup_readiness(ws)?),
        "setup_apply" => out(setup::setup_apply(ws)?),
        "complete_fastlane" => out(setup::setup_complete_fastlane(ws)?),
        "set_team_id" => out(setup::setup_set_team_id(ws, required(args, "team_id")?)?),
        "set_play_key" => out(setup::setup_set_play_key(ws, required(args, "source_path")?)?),
        "store_check" => out(setup::setup_store_check(ws).await?),
        "store_locales" => out(store_locales::store_locales_fetch(ws).await),
        "build_aab" => out(setup::setup_build_aab(ws).await?),

        "asc_status" => out(asc::asc_config_get(ws)),
        "asc_apps" => out(asc::asc_apps_list(ws).await?),
        "asc_territories" => out(asc::asc_territories_list(ws).await?),
        "asc_events_list" => out(asc::asc_events_list(ws, project()?.bundle_id).await?),
        "asc_event_get" => out(asc::asc_event_get(ws, required(args, "id")?).await?),
        "asc_event_create" => out(asc::asc_event_create(ws, project()?.bundle_id, object(args, "attributes")?, object(args, "primary_localization").ok()).await?),
        "asc_event_update" => out(asc::asc_event_update(ws, required(args, "id")?, object(args, "attributes")?).await?),
        "asc_event_delete" => out(asc::asc_event_delete(ws, required(args, "id")?).await?),
        "asc_event_submit" => out(asc::asc_event_submit(ws, required(args, "id")?, required(args, "app_id")?).await?),
        "asc_localization_create" => out(asc::asc_localization_create(ws, required(args, "event_id")?, object(args, "attributes")?).await?),
        "asc_localization_update" => out(asc::asc_localization_update(ws, required(args, "id")?, object(args, "attributes")?).await?),
        "asc_localization_delete" => out(asc::asc_localization_delete(ws, required(args, "id")?).await?),
        "asc_screenshots" => out(asc::asc_localization_screenshots(ws, required(args, "localization_id")?).await?),
        "asc_screenshot_upload" => out(asc::asc_screenshot_upload(ws, required(args, "localization_id")?, required(args, "asset_type")?, required(args, "file_path")?).await?),
        "asc_screenshot_delete" => out(asc::asc_screenshot_delete(ws, required(args, "id")?).await?),

        "firebase_create_project" => {
            let p = project()?;
            out(firebase::firebase_create_project(p.id, p.app_name, Some(p.bundle_id), ws).await?)
        }
        "sentry_create_project" => out(sentry::sentry_create_project(project()?.id, required(args, "org_slug")?).await?),
        "slack_test" => out(slack::slack_test(ws).await?),

        _ => Err(format!("Unknown tool: {name}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn send(raw: &str) -> Result<Vec<u8>, &'static str> {
        send_with(raw, None).await
    }

    async fn send_with(raw: &str, token: Option<&str>) -> Result<Vec<u8>, &'static str> {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let mut client = TcpStream::connect(listener.local_addr().unwrap()).await.unwrap();
        client.write_all(raw.as_bytes()).await.unwrap();
        let (mut server, _) = listener.accept().await.unwrap();
        read_request(&mut server, token).await
    }

    #[tokio::test]
    async fn only_takes_json_posts_without_origin() {
        let body = r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#;
        let post = |extra: &str, kind: &str| format!("POST /mcp HTTP/1.1\r\nContent-Type: {kind}\r\n{extra}Content-Length: {}\r\n\r\n{body}", body.len());
        assert_eq!(send(&post("", "application/json")).await.unwrap(), body.as_bytes());
        assert_eq!(send(&post("Origin: https://evil.example\r\n", "application/json")).await, Err("403 Forbidden"));
        assert_eq!(send(&post("", "text/plain")).await, Err("415 Unsupported Media Type"));
        assert_eq!(send("GET /mcp HTTP/1.1\r\n\r\n").await, Err("405 Method Not Allowed"));
        assert_eq!(send_with(&post("", "application/json"), Some("abc")).await, Err("401 Unauthorized"));
        assert_eq!(send_with(&post("Authorization: Bearer xyz\r\n", "application/json"), Some("abc")).await, Err("401 Unauthorized"));
        assert_eq!(send_with(&post("Authorization: Bearer abc\r\n", "application/json"), Some("abc")).await.unwrap(), body.as_bytes());
    }

    #[test]
    fn picks_numbered_matching_lines_without_colors() {
        let lines: Vec<String> = ["ok", "\x1b[35merror: no profile\x1b[0m", "warning: x", "done"].map(String::from).to_vec();
        assert_eq!(pick_lines(&lines, Some("ERROR|warning"), None), "4 lines in all, 2 matching, showing the last 2:\n2: error: no profile\n3: warning: x");
        assert_eq!(pick_lines(&lines, None, Some(1)), "4 lines in all, 4 matching, showing the last 1:\n4: done");
        assert_eq!(pick_lines(&[], None, None), "No log lines.");
    }

    #[test]
    fn every_tool_is_handled_and_named_once() {
        let names: Vec<&str> = TOOLS.iter().map(|t| t.0).collect();
        let mut unique = names.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), names.len());
        let source = include_str!("mcp.rs");
        for name in names {
            assert!(source.matches(&format!("\"{name}\"")).count() >= 2, "{name} has no handler");
        }
    }
}
