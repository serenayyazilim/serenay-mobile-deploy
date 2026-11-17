//! A live, hand-controllable view of the running simulator or emulator in a browser: `run_app`
//! makes a session token, and `GET /sim/<token>` (page), `GET /sim/<token>/stream` (MJPEG) and
//! `POST /sim/<token>/input` (tap, swipe, text, key, reload, restart) work only while that token
//! is live and only from the local network. The token is the credential, so it is never logged.

use serde_json::{json, Value};
use std::net::{IpAddr, SocketAddr};
use std::process::Stdio;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::process::Command;

#[derive(Clone)]
struct Session {
    token: String,
    /// Device id (adb serial or simulator UDID) and Flutter platform string; None = pick the booted one.
    device: Option<(String, String)>,
    started: Instant,
    /// Pixel size of the last Android frame, for turning 0..1 positions into taps.
    size: Option<(u32, u32)>,
}

static SESSION: Mutex<Option<Session>> = Mutex::new(None);

pub fn start_session(device: Option<(String, String)>) -> String {
    let token = format!("{}{}", uuid::Uuid::new_v4().simple(), uuid::Uuid::new_v4().simple());
    *SESSION.lock().unwrap() = Some(Session { token: token.clone(), device, started: Instant::now(), size: None });
    token
}

pub fn stop_session() {
    *SESSION.lock().unwrap() = None;
}

/// The live session's token: the run must be running, or just started (its status shows up a moment later).
fn live(token: &str, status: &Value) -> Option<Session> {
    let s = SESSION.lock().unwrap().clone().filter(|s| s.token == token)?;
    let running = matches!(status.pointer("/run/status").and_then(Value::as_str), Some("activating" | "running"));
    (running || s.started.elapsed() < Duration::from_secs(30)).then_some(s)
}

/// The viewer link of the live session, if any (for run_app, status and sim_link).
pub fn view_url(ip: Option<&str>, status: &Value) -> Option<String> {
    let token = SESSION.lock().unwrap().as_ref()?.token.clone();
    live(&token, status)?;
    Some(format!("http://{}:{}/sim/{token}", ip.unwrap_or("<this Mac's IP>"), crate::mcp::PORT))
}

fn is_lan(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(a) => a.is_loopback() || a.is_private() || a.is_link_local(),
        IpAddr::V6(a) => a.is_loopback() || (a.segments()[0] & 0xfe00) == 0xfc00 || (a.segments()[0] & 0xffc0) == 0xfe80,
    }
}

async fn reply(stream: &mut TcpStream, status: &str, kind: &str, body: &[u8]) {
    let head = format!("HTTP/1.1 {status}\r\nContent-Type: {kind}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n", body.len());
    let _ = stream.write_all(head.as_bytes()).await;
    let _ = stream.write_all(body).await;
}

pub async fn serve(mut stream: TcpStream, app: AppHandle, peer: SocketAddr, status: impl Fn() -> Value) {
    if !is_lan(peer.ip()) {
        return reply(&mut stream, "403 Forbidden", "text/plain", b"").await;
    }
    let Some((method, path, body)) = read(&mut stream).await else {
        return reply(&mut stream, "400 Bad Request", "text/plain", b"").await;
    };
    let mut parts = path.trim_start_matches("/sim/").splitn(2, '/');
    let (token, rest) = (parts.next().unwrap_or(""), parts.next().unwrap_or(""));
    let Some(session) = live(token, &status()) else {
        return reply(&mut stream, "404 Not Found", "text/plain", b"Not found").await;
    };
    match (method.as_str(), rest) {
        ("GET", "") => reply(&mut stream, "200 OK", "text/html; charset=utf-8", PAGE.as_bytes()).await,
        ("GET", "stream") => stream_frames(stream, session, status).await,
        ("POST", "input") => {
            let (code, msg) = match serde_json::from_slice::<Value>(&body) {
                Ok(input) => match input_event(&app, &session, &input).await {
                    Ok(()) => ("200 OK", "ok".to_string()),
                    Err(e) => ("422 Unprocessable Entity", e),
                },
                Err(e) => ("400 Bad Request", e.to_string()),
            };
            reply(&mut stream, code, "text/plain; charset=utf-8", msg.as_bytes()).await
        }
        _ => reply(&mut stream, "404 Not Found", "text/plain", b"Not found").await,
    }
}

/// Method, path and body of one request (headers only as far as Content-Length is concerned).
async fn read(stream: &mut TcpStream) -> Option<(String, String, Vec<u8>)> {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 4096];
    let end = loop {
        let n = stream.read(&mut chunk).await.ok().filter(|&n| n > 0)?;
        buf.extend_from_slice(&chunk[..n]);
        if let Some(i) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            break i + 4;
        }
        if buf.len() > 16384 {
            return None;
        }
    };
    let head = String::from_utf8_lossy(&buf[..end]).to_string();
    let mut line = head.lines().next()?.split(' ');
    let (method, path) = (line.next()?.to_string(), line.next()?.split('?').next()?.to_string());
    let length: usize = head.lines().find_map(|l| l.to_lowercase().strip_prefix("content-length:").and_then(|v| v.trim().parse().ok())).unwrap_or(0);
    if length > 65536 {
        return None;
    }
    let mut body = buf.split_off(end);
    while body.len() < length {
        let n = stream.read(&mut chunk).await.ok().filter(|&n| n > 0)?;
        body.extend_from_slice(&chunk[..n]);
    }
    body.truncate(length);
    Some((method, path, body))
}

async fn run(program: &str, args: &[&str]) -> Result<Vec<u8>, String> {
    let child = Command::new(program).args(args).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).kill_on_drop(true).output();
    let out = tokio::time::timeout(Duration::from_secs(10), child)
        .await
        .map_err(|_| format!("{program} timed out"))?
        .map_err(|e| if e.kind() == std::io::ErrorKind::NotFound { format!("{program} not found on PATH") } else { e.to_string() })?;
    if out.status.success() {
        Ok(out.stdout)
    } else {
        Err(format!("{program}: {}", String::from_utf8_lossy(&out.stderr).trim()))
    }
}

/// idb comes from pipx, whose bin folder is often missing from a GUI app's PATH.
fn idb_path() -> Result<String, String> {
    let path = std::env::var_os("PATH").unwrap_or_default();
    let home = dirs::home_dir().map(|h| h.join(".local/bin"));
    std::env::split_paths(&path)
        .chain(home)
        .map(|d| d.join("idb"))
        .find(|p| p.is_file())
        .map(|p| p.to_string_lossy().to_string())
        .ok_or("idb not found: install it with `pipx install fb-idb` (and idb_companion with brew)".to_string())
}

/// The session's device, or the first Android emulator / booted iOS simulator.
async fn device(s: &Session) -> Result<(String, bool), String> {
    if let Some((id, platform)) = &s.device {
        return Ok((id.clone(), platform.to_lowercase().starts_with("ios")));
    }
    let adb = run("adb", &["devices"]).await.unwrap_or_default();
    if let Some(id) = String::from_utf8_lossy(&adb).lines().find_map(|l| l.strip_suffix("\tdevice").filter(|id| id.starts_with("emulator-"))) {
        return Ok((id.to_string(), false));
    }
    let booted = run("xcrun", &["simctl", "list", "devices", "booted", "-j"]).await.unwrap_or_default();
    let list: Value = serde_json::from_slice(&booted).unwrap_or(Value::Null);
    list["devices"].as_object().into_iter().flat_map(|m| m.values()).flat_map(|v| v.as_array().into_iter().flatten()).find_map(|d| d["udid"].as_str()).map(|u| (u.to_string(), true)).ok_or("No running simulator or emulator found".to_string())
}

/// A frame (PNG for Android, JPEG for iOS) and its content type.
async fn frame(id: &str, ios: bool) -> Result<(Vec<u8>, &'static str), String> {
    if !ios {
        return run("adb", &["-s", id, "exec-out", "screencap", "-p"]).await.map(|b| (b, "image/png"));
    }
    // simctl treats "-" as a file name, so go through a temp file.
    let file = std::env::temp_dir().join(format!("sermobile-sim-{id}.jpg"));
    let path = file.to_string_lossy().to_string();
    run("xcrun", &["simctl", "io", id, "screenshot", "--type=jpeg", &path]).await?;
    let bytes = std::fs::read(&file).map_err(|e| e.to_string());
    let _ = std::fs::remove_file(&file);
    bytes.map(|b| (b, "image/jpeg"))
}

fn png_size(b: &[u8]) -> Option<(u32, u32)> {
    let n = |i: usize| b.get(i..i + 4).map(|x| u32::from_be_bytes([x[0], x[1], x[2], x[3]]));
    Some((n(16)?, n(20)?)).filter(|_| b.starts_with(b"\x89PNG"))
}

async fn stream_frames(mut stream: TcpStream, s: Session, status: impl Fn() -> Value) {
    let head = "HTTP/1.1 200 OK\r\nContent-Type: multipart/x-mixed-replace; boundary=frame\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n";
    if stream.write_all(head.as_bytes()).await.is_err() {
        return;
    }
    let (mut last, mut sent) = (0u64, Instant::now());
    let mut errors = 0;
    while live(&s.token, &status()).is_some() {
        let t = Instant::now();
        match device(&s).await {
            Ok((id, ios)) => match frame(&id, ios).await {
                Ok((bytes, kind)) => {
                    errors = 0;
                    if let Some(size) = png_size(&bytes) {
                        if let Some(cur) = SESSION.lock().unwrap().as_mut().filter(|c| c.token == s.token) {
                            cur.size = Some(size);
                        }
                    }
                    let hash = {
                        use std::hash::{Hash, Hasher};
                        let mut h = std::collections::hash_map::DefaultHasher::new();
                        bytes.hash(&mut h);
                        h.finish()
                    };
                    // An unchanged screen is repeated now and then so the connection stays alive.
                    if hash != last || sent.elapsed() > Duration::from_secs(5) {
                        let part = format!("--frame\r\nContent-Type: {kind}\r\nContent-Length: {}\r\n\r\n", bytes.len());
                        if stream.write_all(part.as_bytes()).await.is_err() || stream.write_all(&bytes).await.is_err() || stream.write_all(b"\r\n").await.is_err() {
                            return;
                        }
                        (last, sent) = (hash, Instant::now());
                    }
                }
                Err(_) => errors += 1,
            },
            Err(_) => errors += 1,
        }
        if errors > 20 {
            return;
        }
        tokio::time::sleep(Duration::from_millis(120).saturating_sub(t.elapsed())).await;
    }
}

/// The screen size input is sent in: Android pixels, iOS points.
async fn screen(s: &Session, id: &str, ios: bool) -> Result<(f64, f64), String> {
    if ios {
        let out = run(&idb_path()?, &["describe", "--udid", id, "--json"]).await?;
        let d: Value = serde_json::from_slice(&out).map_err(|e| e.to_string())?;
        let dim = &d["screen_dimensions"];
        return match (dim["width_points"].as_f64(), dim["height_points"].as_f64()) {
            (Some(w), Some(h)) => Ok((w, h)),
            _ => Err("idb describe gave no screen size".to_string()),
        };
    }
    if let Some((w, h)) = SESSION.lock().unwrap().as_ref().filter(|c| c.token == s.token).and_then(|c| c.size) {
        return Ok((w as f64, h as f64));
    }
    let out = String::from_utf8_lossy(&run("adb", &["-s", id, "shell", "wm", "size"]).await?).to_string();
    let size = out.lines().last().and_then(|l| l.split(':').nth(1)).and_then(|v| v.trim().split_once('x')).and_then(|(w, h)| Some((w.parse().ok()?, h.parse().ok()?)));
    size.ok_or("Could not read the screen size".to_string())
}

/// `input text` takes `%s` for a space and runs through the device's shell, so quote it; it cannot type non-ASCII.
fn android_text(text: &str) -> Result<String, String> {
    if !text.is_ascii() {
        return Err("Android's input tool types only ASCII text".to_string());
    }
    Ok(format!("'{}'", text.replace('\'', "'\\''").replace('%', "%25").replace(' ', "%s")))
}

async fn input_event(app: &AppHandle, s: &Session, input: &Value) -> Result<(), String> {
    let kind = input["type"].as_str().unwrap_or("");
    match kind {
        "reload" | "restart" => return app.emit("mcp-action", json!({ "action": if kind == "reload" { "hotReload" } else { "hotRestart" } })).map_err(|e| e.to_string()),
        _ => {}
    }
    let (id, ios) = device(s).await?;
    let ratio = |k: &str| input[k].as_f64().filter(|v| (0.0..=1.0).contains(v)).ok_or(format!("{k} must be between 0 and 1"));
    let (w, h) = if matches!(kind, "tap" | "swipe") { screen(s, &id, ios).await? } else { (0.0, 0.0) };
    let pos = |x: &str, y: &str| -> Result<(String, String), String> { Ok((((ratio(x)? * w).round() as i64).to_string(), ((ratio(y)? * h).round() as i64).to_string())) };
    match (kind, ios) {
        ("tap", false) => {
            let (x, y) = pos("x", "y")?;
            run("adb", &["-s", &id, "shell", "input", "tap", &x, &y]).await.map(drop)
        }
        ("tap", true) => {
            let (x, y) = pos("x", "y")?;
            run(&idb_path()?, &["ui", "tap", "--udid", &id, &x, &y]).await.map(drop)
        }
        ("swipe", _) => {
            let ((x, y), (x2, y2)) = (pos("x", "y")?, pos("x2", "y2")?);
            let ms = input["ms"].as_u64().unwrap_or(300).clamp(50, 2000);
            if ios {
                run(&idb_path()?, &["ui", "swipe", "--udid", &id, "--duration", &format!("{}", ms as f64 / 1000.0), &x, &y, &x2, &y2]).await.map(drop)
            } else {
                run("adb", &["-s", &id, "shell", "input", "swipe", &x, &y, &x2, &y2, &ms.to_string()]).await.map(drop)
            }
        }
        ("text", _) => {
            let text = input["text"].as_str().filter(|t| !t.is_empty() && t.len() <= 500).ok_or("text must be 1 to 500 characters")?;
            if ios {
                run(&idb_path()?, &["ui", "text", "--udid", &id, text]).await.map(drop)
            } else {
                run("adb", &["-s", &id, "shell", "input", "text", &android_text(text)?]).await.map(drop)
            }
        }
        ("key", _) => {
            let key = input["key"].as_str().unwrap_or("");
            // idb: HID usage codes for enter (40) and backspace (42).
            match (key, ios) {
                ("home", true) => run(&idb_path()?, &["ui", "button", "--udid", &id, "HOME"]).await.map(drop),
                ("enter", true) => run(&idb_path()?, &["ui", "key", "--udid", &id, "40"]).await.map(drop),
                ("backspace", true) => run(&idb_path()?, &["ui", "key", "--udid", &id, "42"]).await.map(drop),
                ("back" | "app_switch", true) => Err(format!("{key} does not exist on iOS")),
                ("home" | "back" | "enter" | "backspace" | "app_switch", false) => {
                    let code = match key { "home" => "KEYCODE_HOME", "back" => "KEYCODE_BACK", "enter" => "KEYCODE_ENTER", "app_switch" => "KEYCODE_APP_SWITCH", _ => "KEYCODE_DEL" };
                    run("adb", &["-s", &id, "shell", "input", "keyevent", code]).await.map(drop)
                }
                _ => Err("key must be home, back, enter, backspace or app_switch".to_string()),
            }
        }
        _ => Err("type must be tap, swipe, text, key, reload or restart".to_string()),
    }
}

const PAGE: &str = r##"<!doctype html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1,maximum-scale=1,user-scalable=no">
<title>Simulator</title><style>
body{margin:0;background:#111;color:#eee;font:14px system-ui;display:flex;flex-direction:column;align-items:center;gap:8px;padding:8px}
#bar{display:flex;gap:6px;flex-wrap:wrap;justify-content:center}button,input{font:inherit;padding:6px 10px;border-radius:6px;border:1px solid #444;background:#222;color:#eee}
#screen{max-width:100%;max-height:78vh;touch-action:none;user-select:none;-webkit-user-drag:none;background:#000;cursor:pointer}#msg{color:#f88;min-height:1.2em}
</style></head><body>
<div id="bar"><button data-t="reload">Hot reload</button><button data-t="restart">Hot restart</button><button data-k="back">Back</button><button data-k="home">Home</button><button data-k="app_switch">Apps</button><button data-k="enter">Enter</button><button data-k="backspace">⌫</button></div>
<img id="screen" alt="screen" draggable="false">
<form id="f" style="display:flex;gap:6px"><input id="txt" placeholder="Type text" autocomplete="off"><button>Send</button></form>
<div id="msg"></div>
<script>
const base=location.pathname.replace(/\/$/,''),img=document.getElementById('screen'),msg=document.getElementById('msg');
img.src=base+'/stream';
async function send(o){try{const r=await fetch(base+'/input',{method:'POST',body:JSON.stringify(o)});msg.textContent=r.ok?'':await r.text()}catch(e){msg.textContent=String(e)}}
document.querySelectorAll('button[data-t]').forEach(b=>b.onclick=()=>send({type:b.dataset.t}));
document.querySelectorAll('button[data-k]').forEach(b=>b.onclick=()=>send({type:'key',key:b.dataset.k}));
document.getElementById('f').onsubmit=e=>{e.preventDefault();const t=txt.value;if(t){send({type:'text',text:t});txt.value=''}};
let down=null;
const pos=e=>{const r=img.getBoundingClientRect();return{x:Math.min(1,Math.max(0,(e.clientX-r.left)/r.width)),y:Math.min(1,Math.max(0,(e.clientY-r.top)/r.height))}};
img.onpointerdown=e=>{img.setPointerCapture(e.pointerId);down={...pos(e),t:Date.now()}};
img.onpointerup=e=>{if(!down)return;const p=pos(e),d=down;down=null;
 if(Math.hypot(p.x-d.x,p.y-d.y)<0.015)send({type:'tap',x:d.x,y:d.y});
 else send({type:'swipe',x:d.x,y:d.y,x2:p.x,y2:p.y,ms:Date.now()-d.t})};
img.onpointercancel=()=>{down=null};
img.onerror=()=>{msg.textContent='Stream ended (app stopped?)'};
</script></body></html>"##;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn android_text_is_quoted() {
        assert_eq!(android_text("a b'c%").unwrap(), "'a%sb'\\''c%25'");
        assert!(android_text("şeker").is_err());
    }

    #[test]
    fn session_dies_with_stop() {
        let t = start_session(None);
        assert!(live(&t, &Value::Null).is_some());
        assert!(live("wrong", &Value::Null).is_none());
        stop_session();
        assert!(live(&t, &Value::Null).is_none());
    }

    #[tokio::test]
    #[ignore] // needs a booted iOS simulator + idb
    async fn ios_live() {
        let id = "04F12959-54BB-45D2-86E0-17B781D0399B";
        let (b, k) = frame(id, true).await.unwrap();
        println!("frame {} {k}", b.len());
        let s = Session { token: "x".into(), device: None, started: Instant::now(), size: None };
        println!("screen {:?}", screen(&s, id, true).await);
        println!("tap {:?}", run(&idb_path().unwrap(), &["ui", "tap", "--udid", id, "200", "800"]).await.map(drop));
    }
}
