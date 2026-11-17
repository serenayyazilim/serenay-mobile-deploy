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
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::process::{Child, ChildStdin, ChildStdout, Command};

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
    *IOS.lock().unwrap() = None;
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

/// iOS: one idb_companion per session on a private unix socket (not reachable from the network), a
/// persistent Python idb client for input (about 90 ms per tap instead of a new process each time),
/// and an MJPEG stream from the same companion. Dropping it kills both processes.
struct Ios {
    udid: String,
    sock: std::path::PathBuf,
    points: (f64, f64),
    scale: f64,
    helper: Arc<tokio::sync::Mutex<(ChildStdin, BufReader<ChildStdout>)>>,
    children: Mutex<(Child, Child)>, // companion, helper; dropping them kills both
}

impl Drop for Ios {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.sock);
    }
}

static IOS: Mutex<Option<Arc<Ios>>> = Mutex::new(None);
static IOS_START: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

const HELPER: &str = r##"
import asyncio, json, sys
from idb.common.types import DomainSocketAddress, HIDButtonType, HIDDelay, HIDDirection, HIDKey, HIDPress
from idb.grpc.client import Client
import logging
# Turkish QWERTY (PC) as the simulator sees it: char -> (US HID usage, modifiers). 225 = shift, 230 = AltGr.
def tr_map():
    m = {}
    for i, ch in enumerate("abcdefghijklmnopqrstuvwxyz"):
        m[ch] = (4 + i, []); m[ch.upper()] = (4 + i, [225])
    m["i"] = (52, []); m["İ"] = (52, [225]); m["ı"] = (12, []); m["I"] = (12, [225])
    for ch, up, k in (("ğ", "Ğ", 47), ("ü", "Ü", 48), ("ş", "Ş", 51), ("ö", "Ö", 54), ("ç", "Ç", 55)):
        m[ch] = (k, []); m[up] = (k, [225])
    for i, (d, sh) in enumerate(zip("1234567890", "!'^+%&/()=")):
        m[d] = (30 + i, []); m[sh] = (30 + i, [225])
    for k, plain, shifted in ((45, "*", "?"), (46, "-", "_"), (49, ",", ";"), (56, ".", ":"), (53, "<", ">")):
        m[plain] = (k, []); m[shifted] = (k, [225])
    m['"'] = (100, []); m[" "] = (44, []); m["\n"] = (40, [])
    for ch, k in (("@", 20), ("#", 32), ("$", 33), ("{", 36), ("[", 37), ("]", 38), ("}", 39), ("\\", 45), ("~", 48), ("|", 53)):
        m[ch] = (k, [230])
    return m
TR = tr_map()
async def events(text, layout):
    for ch in text:
        k, mods = TR[ch] if layout == "tr" else (None, None)
        for x in mods: yield HIDPress(action=HIDKey(keycode=x), direction=HIDDirection.DOWN)
        yield HIDPress(action=HIDKey(keycode=k), direction=HIDDirection.DOWN)
        yield HIDPress(action=HIDKey(keycode=k), direction=HIDDirection.UP)
        for x in reversed(mods): yield HIDPress(action=HIDKey(keycode=x), direction=HIDDirection.UP)
async def main():
    async with Client.build(address=DomainSocketAddress(path=sys.argv[1]), logger=logging.getLogger()) as c:
        print("ready", flush=True)
        loop = asyncio.get_running_loop()
        while True:
            line = await loop.run_in_executor(None, sys.stdin.readline)
            if not line: return
            try:
                o = json.loads(line)
                op = o["op"]
                if op == "tap": await c.tap(o["x"], o["y"])
                elif op == "swipe": await c.swipe((o["x"], o["y"]), (o["x2"], o["y2"]), o["s"])
                elif op == "text": await c.text(o["text"])
                elif op == "key": await c.key(o["code"])
                elif op == "type":
                    missing = [ch for ch in o["text"] if ch not in TR]
                    if missing: raise Exception("no key for " + "".join(missing))
                    await c.hid(events(o["text"], "tr"))
                elif op == "paste":
                    # Left Cmd (227) + V (25), layout independent
                    async def ev():
                        for k, d in ((227, HIDDirection.DOWN), (25, HIDDirection.DOWN), (25, HIDDirection.UP), (227, HIDDirection.UP)):
                            yield HIDPress(action=HIDKey(keycode=k), direction=d)
                            yield HIDDelay(duration=0.25)
                    await c.hid(ev())
                elif op == "home": await c.button(HIDButtonType.HOME)
                print("ok", flush=True)
            except Exception as e:
                print("err " + str(e).replace("\n", " "), flush=True)
asyncio.run(main())
"##;

/// Characters the helper's Turkish layout table knows besides letters.
const TR_CHARS: &str = "ığüşöçİĞÜŞÖÇ 0123456789!'^+%&/()=*?-_,;.:<>\"@#${[]}~|\n\\";

/// The simulator follows the Mac's keyboard layout, and idb sends US key positions.
async fn host_layout_is_turkish() -> bool {
    let out = run("defaults", &["read", "com.apple.HIToolbox", "AppleCurrentKeyboardLayoutInputSourceID"]).await.unwrap_or_default();
    String::from_utf8_lossy(&out).contains("Turkish")
}

/// The Python of the pipx environment idb lives in: it sits next to the idb script.
fn idb_python() -> Result<String, String> {
    let idb = idb_path()?;
    let real = std::fs::canonicalize(&idb).map_err(|e| e.to_string())?;
    let py = real.with_file_name("python");
    py.is_file().then(|| py.to_string_lossy().to_string()).ok_or("could not find idb's Python".to_string())
}

async fn start_ios(id: &str) -> Result<Arc<Ios>, String> {
    let sock = std::env::temp_dir().join(format!("sermobile-idb-{}.sock", &uuid::Uuid::new_v4().simple().to_string()[..12]));
    let _ = std::fs::remove_file(&sock);
    let sock_s = sock.to_string_lossy().to_string();
    let mut companion = Command::new("idb_companion")
        .args(["--udid", id, "--grpc-domain-sock", &sock_s])
        .env("PATH", format!("/opt/homebrew/bin:/usr/local/bin:{}", std::env::var("PATH").unwrap_or_default()))
        .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).kill_on_drop(true)
        .spawn()
        .map_err(|e| if e.kind() == std::io::ErrorKind::NotFound { "idb_companion not found: brew install idb-companion".to_string() } else { e.to_string() })?;
    let mut waited = 0;
    while !sock.exists() {
        if companion.try_wait().ok().flatten().is_some() || waited > 100 {
            return Err("idb_companion did not start".to_string());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
        waited += 1;
    }
    let out = run(&idb_path()?, &["--companion", &sock_s, "describe", "--udid", id, "--json"]).await?;
    let d: Value = serde_json::from_slice(&out).map_err(|e| e.to_string())?;
    let dim = &d["screen_dimensions"];
    let (pw, ph, px) = (dim["width_points"].as_f64(), dim["height_points"].as_f64(), dim["width"].as_f64());
    let (Some(pw), Some(ph)) = (pw, ph) else { return Err("idb describe gave no screen size".to_string()) };
    let mut helper = Command::new(idb_python()?)
        .args(["-c", HELPER, &sock_s])
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null()).kill_on_drop(true)
        .spawn()
        .map_err(|e| e.to_string())?;
    let stdin = helper.stdin.take().ok_or("no helper stdin")?;
    let mut stdout = BufReader::new(helper.stdout.take().ok_or("no helper stdout")?);
    let mut line = String::new();
    tokio::time::timeout(Duration::from_secs(10), stdout.read_line(&mut line)).await.map_err(|_| "idb client timed out")?.map_err(|e| e.to_string())?;
    if line.trim() != "ready" {
        return Err("idb client did not start".to_string());
    }
    // Frames about 500 px wide are plenty for a phone browser.
    let scale = px.filter(|w| *w > 0.0).map(|w| (500.0 / w).min(1.0)).unwrap_or(0.4);
    Ok(Arc::new(Ios { udid: id.to_string(), sock, points: (pw, ph), scale, helper: Arc::new(tokio::sync::Mutex::new((stdin, stdout))), children: Mutex::new((companion, helper)) }))
}

async fn ios_session(id: &str) -> Result<Arc<Ios>, String> {
    let _guard = IOS_START.lock().await;
    // A companion or client that died is dropped and started again.
    if let Some(i) = IOS.lock().unwrap().clone().filter(|i| i.udid == id) {
        let mut c = i.children.lock().unwrap();
        if matches!(c.0.try_wait(), Ok(None)) && matches!(c.1.try_wait(), Ok(None)) {
            drop(c);
            return Ok(i);
        }
    }
    *IOS.lock().unwrap() = None;
    let ios = start_ios(id).await?;
    *IOS.lock().unwrap() = Some(ios.clone());
    Ok(ios)
}

impl Ios {
    async fn send(&self, op: Value) -> Result<(), String> {
        let mut h = self.helper.lock().await;
        let mut line = op.to_string();
        line.push('\n');
        h.0.write_all(line.as_bytes()).await.map_err(|e| e.to_string())?;
        let mut reply = String::new();
        let n = tokio::time::timeout(Duration::from_secs(10), h.1.read_line(&mut reply)).await.map_err(|_| "idb client timed out")?.map_err(|e| e.to_string())?;
        match reply.trim() {
            "ok" => Ok(()),
            _ if n == 0 => Err("idb client stopped".to_string()),
            e => Err(e.trim_start_matches("err ").to_string()),
        }
    }
}

/// Pulls whole JPEGs (start FFD8, end FFD9) out of the raw MJPEG stream of `idb video-stream`.
fn next_jpeg(buf: &mut Vec<u8>) -> Option<Vec<u8>> {
    let start = buf.windows(2).position(|w| w == [0xff, 0xd8])?;
    let end = buf[start..].windows(2).position(|w| w == [0xff, 0xd9])? + start + 2;
    let frame = buf[start..end].to_vec();
    buf.drain(..end);
    Some(frame)
}

/// Forwards the JPEGs of a raw MJPEG pipe as multipart parts. Ok(frames) when the pipe ended (or the viewer left,
/// then `gone` is set); Err when no frame ever came.
async fn pump(stream: &mut TcpStream, s: &Session, status: &impl Fn() -> Value, out: &mut (impl AsyncReadExt + Unpin), gone: &mut bool) -> Result<u32, String> {
    let (mut buf, mut chunk, mut frames) = (Vec::new(), vec![0u8; 65536], 0u32);
    loop {
        if live(&s.token, &status()).is_none() {
            *gone = true;
            return Ok(frames);
        }
        // A still screen sends nothing, so a quiet few seconds are normal once frames have flowed.
        let wait = if frames == 0 { Duration::from_secs(15) } else { Duration::from_secs(3) };
        match tokio::time::timeout(wait, out.read(&mut chunk)).await {
            Ok(Ok(n)) if n > 0 => buf.extend_from_slice(&chunk[..n]),
            Ok(_) if frames > 0 => return Ok(frames),
            Ok(_) => return Err("stream ended".to_string()),
            Err(_) if frames == 0 => return Err("no frames".to_string()),
            Err(_) => continue,
        }
        // Only the newest frame matters if the browser is slower than the stream.
        let mut last = None;
        while let Some(f) = next_jpeg(&mut buf) {
            last = Some(f);
        }
        if let Some(f) = last {
            frames += 1;
            let part = format!("--frame\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\n\r\n", f.len());
            if stream.write_all(part.as_bytes()).await.is_err() || stream.write_all(&f).await.is_err() || stream.write_all(b"\r\n").await.is_err() {
                *gone = true;
                return Ok(frames);
            }
        }
        if buf.len() > 8_000_000 {
            return Err("stream garbled".to_string());
        }
    }
}

/// Ok(()) when the viewer left; Err means the stream failed before any frame (fall back to screenshots).
async fn stream_ios(stream: &mut TcpStream, s: &Session, ios: &Ios, status: &impl Fn() -> Value) -> Result<(), String> {
    let mut child = Command::new(idb_path()?)
        .args(["--companion", &ios.sock.to_string_lossy(), "video-stream", "--udid", &ios.udid, "--format", "mjpeg", "--fps", "30", "--compression-quality", "0.6", "--scale-factor", &format!("{:.3}", ios.scale)])
        .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null()).kill_on_drop(true)
        .spawn()
        .map_err(|e| e.to_string())?;
    let mut out = child.stdout.take().ok_or("no stream")?;
    let mut gone = false;
    pump(stream, s, status, &mut out, &mut gone).await.map(drop)
}

/// Android: `screenrecord` (H.264, only sends when the screen changes, stops after 3 minutes) decoded to MJPEG by ffmpeg.
/// There is no scrcpy dependency; both tools are optional and the screencap loop takes over without them.
async fn stream_android(stream: &mut TcpStream, s: &Session, id: &str, status: &impl Fn() -> Value) -> Result<(), String> {
    let (w, h) = screen(s, id, false).await?;
    let width = 540.0_f64.min(w);
    let size = format!("{}x{}", (width / 2.0).round() as u32 * 2, (width * h / w / 2.0).round() as u32 * 2);
    let ffmpeg = ["/opt/homebrew/bin/ffmpeg", "/usr/local/bin/ffmpeg"].into_iter().find(|p| std::path::Path::new(p).is_file()).unwrap_or("ffmpeg");
    let mut total = 0;
    loop {
        let mut rec = Command::new("adb")
            .args(["-s", id, "exec-out", "screenrecord", "--output-format=h264", "--size", &size, "--bit-rate", "3000000", "-"])
            .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null()).kill_on_drop(true)
            .spawn()
            .map_err(|e| e.to_string())?;
        let input: Stdio = rec.stdout.take().ok_or("no screenrecord")?.try_into().map_err(|e: std::io::Error| e.to_string())?;
        let mut ff = Command::new(ffmpeg)
            .args(["-loglevel", "quiet", "-probesize", "20000", "-analyzeduration", "0", "-f", "h264", "-i", "-", "-pix_fmt", "yuvj420p", "-f", "mjpeg", "-q:v", "6", "-"])
            .stdin(input).stdout(Stdio::piped()).stderr(Stdio::null()).kill_on_drop(true)
            .spawn()
            .map_err(|e| if e.kind() == std::io::ErrorKind::NotFound { "ffmpeg not found".to_string() } else { e.to_string() })?;
        let mut out = ff.stdout.take().ok_or("no ffmpeg output")?;
        let mut gone = false;
        match pump(stream, s, status, &mut out, &mut gone).await {
            Ok(n) => total += n,
            Err(e) if total == 0 => return Err(e),
            Err(_) => {}
        }
        if gone {
            return Ok(());
        }
    }
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
    if let Ok((id, false)) = device(&s).await {
        match stream_android(&mut stream, &s, &id, &status).await {
            Ok(()) => return,
            Err(e) => eprintln!("sim: Android stream failed ({e}), using screencap"),
        }
    }
    if let Ok((id, true)) = device(&s).await {
        match ios_session(&id).await {
            Ok(ios) => match stream_ios(&mut stream, &s, &ios, &status).await {
                Ok(()) => return,
                Err(e) => eprintln!("sim: iOS stream failed ({e}), using screenshots"),
            },
            Err(e) => eprintln!("sim: idb companion failed ({e}), using screenshots"),
        }
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
        return Ok(ios_session(id).await?.points);
    }
    if let Some((w, h)) = SESSION.lock().unwrap().as_ref().filter(|c| c.token == s.token).and_then(|c| c.size) {
        return Ok((w as f64, h as f64));
    }
    let out = String::from_utf8_lossy(&run("adb", &["-s", id, "shell", "wm", "size"]).await?).to_string();
    let size = out.lines().last().and_then(|l| l.split(':').nth(1)).and_then(|v| v.trim().split_once('x')).and_then(|(w, h)| Some((w.parse().ok()?, h.parse().ok()?)));
    size.ok_or("Could not read the screen size".to_string())
}

/// Non-ASCII text on Android needs ADB Keyboard (com.android.adbkeyboard) as the active keyboard: `input text` cannot
/// type it and `cmd clipboard` does not exist on the emulators tried (Android 14), so there is no built-in way.
async fn android_paste(id: &str, text: &str) -> Result<(), String> {
    use base64::Engine;
    let ime = String::from_utf8_lossy(&run("adb", &["-s", id, "shell", "settings", "get", "secure", "default_input_method"]).await?).to_string();
    if !ime.contains("adbkeyboard") {
        return Err("Typing Turkish or other non-ASCII text on Android needs ADB Keyboard: install it (github.com/senzhk/ADBKeyBoard) and pick it with `adb shell ime set com.android.adbkeyboard/.AdbIME`".to_string());
    }
    let b64 = base64::engine::general_purpose::STANDARD.encode(text);
    run("adb", &["-s", id, "shell", "am", "broadcast", "-a", "ADB_INPUT_B64", "--es", "msg", &b64]).await.map(drop)
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
            ios_session(&id).await?.send(json!({"op": "tap", "x": x.parse::<f64>().unwrap_or(0.0), "y": y.parse::<f64>().unwrap_or(0.0)})).await
        }
        ("swipe", _) => {
            let ((x, y), (x2, y2)) = (pos("x", "y")?, pos("x2", "y2")?);
            let ms = input["ms"].as_u64().unwrap_or(300).clamp(50, 2000);
            if ios {
                let n = |v: &str| v.parse::<f64>().unwrap_or(0.0);
                ios_session(&id).await?.send(json!({"op": "swipe", "x": n(&x), "y": n(&y), "x2": n(&x2), "y2": n(&y2), "s": ms as f64 / 1000.0})).await
            } else {
                run("adb", &["-s", &id, "shell", "input", "swipe", &x, &y, &x2, &y2, &ms.to_string()]).await.map(drop)
            }
        }
        ("text", _) => {
            let text = input["text"].as_str().filter(|t| !t.is_empty() && t.len() <= 500).ok_or("text must be 1 to 500 characters")?;
            if ios {
                // Typing goes through the pasteboard: `idb ui text` follows the simulator's keyboard layout.
                let mut pb = Command::new("xcrun").args(["simctl", "pbcopy", &id]).stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::null()).kill_on_drop(true).spawn().map_err(|e| e.to_string())?;
                pb.stdin.take().ok_or("no pbcopy stdin")?.write_all(text.as_bytes()).await.map_err(|e| e.to_string())?;
                pb.wait().await.map_err(|e| e.to_string())?;
                // Some simulator runtimes ignore pbcopy (nothing comes back); then fall back to typing keys, which follows the keyboard layout.
                let back = run("xcrun", &["simctl", "pbpaste", &id]).await.unwrap_or_default();
                let op = if back == text.as_bytes() {
                    json!({"op": "paste"})
                } else if host_layout_is_turkish().await && text.chars().all(|c| TR_CHARS.contains(c) || c.is_ascii_alphabetic()) {
                    json!({"op": "type", "text": text})
                } else {
                    json!({"op": "text", "text": text})
                };
                ios_session(&id).await?.send(op).await
            } else if text.is_ascii() {
                run("adb", &["-s", &id, "shell", "input", "text", &android_text(text)?]).await.map(drop)
            } else {
                android_paste(&id, text).await
            }
        }
        ("key", _) => {
            let key = input["key"].as_str().unwrap_or("");
            // idb: HID usage codes for enter (40) and backspace (42).
            match (key, ios) {
                ("home", true) => ios_session(&id).await?.send(json!({"op": "home"})).await,
                ("enter", true) => ios_session(&id).await?.send(json!({"op": "key", "code": 40})).await,
                ("backspace", true) => ios_session(&id).await?.send(json!({"op": "key", "code": 42})).await,
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
    fn jpegs_are_cut_from_a_raw_stream() {
        let mut b = vec![1, 2, 0xff, 0xd8, 9, 9, 0xff, 0xd9, 0xff, 0xd8, 7];
        assert_eq!(next_jpeg(&mut b).unwrap(), vec![0xff, 0xd8, 9, 9, 0xff, 0xd9]);
        assert!(next_jpeg(&mut b).is_none());
        assert_eq!(b, vec![0xff, 0xd8, 7]);
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
        let t = Instant::now();
        println!("tap {:?} in {:?}", ios_session(id).await.unwrap().send(json!({"op": "tap", "x": 200.0, "y": 800.0})).await, t.elapsed());
        stop_session();
    }

    #[tokio::test]
    #[ignore] // needs a booted iOS simulator + idb: measures frames per second while the screen moves, and tap latency
    async fn ios_fps() {
        let id = "04F12959-54BB-45D2-86E0-17B781D0399B";
        let token = start_session(Some((id.to_string(), "ios".to_string())));
        let session = SESSION.lock().unwrap().clone().unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut sock, _) = listener.accept().await.unwrap();
            let ios = ios_session(id).await.unwrap();
            let _ = stream_ios(&mut sock, &session, &ios, &|| json!({"run": {"status": "running"}})).await;
        });
        let mut client = TcpStream::connect(addr).await.unwrap();
        let mover = tokio::spawn(async move {
            let mut lat = vec![];
            for i in 0..12 {
                let ios = ios_session(id).await.unwrap();
                let t = Instant::now();
                let (y1, y2) = if i % 2 == 0 { (700.0, 300.0) } else { (300.0, 700.0) };
                let _ = ios.send(json!({"op": "swipe", "x": 200.0, "y": y1, "x2": 200.0, "y2": y2, "s": 0.3})).await;
                lat.push(t.elapsed());
                let t = Instant::now();
                let _ = ios.send(json!({"op": "tap", "x": 5.0, "y": 5.0})).await;
                println!("tap {:?}", t.elapsed());
            }
            lat
        });
        let (mut bytes, mut chunk, start) = (Vec::new(), vec![0u8; 65536], Instant::now());
        while start.elapsed() < Duration::from_secs(8) {
            if let Ok(Ok(n)) = tokio::time::timeout(Duration::from_secs(1), client.read(&mut chunk)).await {
                bytes.extend_from_slice(&chunk[..n]);
            }
        }
        let frames = bytes.windows(8).filter(|w| w == b"--frame\r").count();
        println!("{token:.4} frames {frames} in 8s, {} bytes, first frame {:?}", bytes.len(), start.elapsed());
        let _ = mover.await;
        stop_session();
        server.abort();
    }

    #[tokio::test]
    #[ignore] // needs a running Android emulator + ffmpeg: counts frames of the screenrecord stream while the screen scrolls
    async fn android_fps() {
        let id = "emulator-5554";
        start_session(Some((id.to_string(), "android".to_string())));
        let session = SESSION.lock().unwrap().clone().unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut sock, _) = listener.accept().await.unwrap();
            println!("stream result {:?}", stream_android(&mut sock, &session, id, &|| json!({"run": {"status": "running"}})).await);
        });
        let mut client = TcpStream::connect(addr).await.unwrap();
        let mover = tokio::spawn(async move {
            for i in 0..10 {
                let (a, b) = if i % 2 == 0 { ("1500", "500") } else { ("500", "1500") };
                let t = Instant::now();
                let _ = run("adb", &["-s", id, "shell", "input", "swipe", "500", a, "500", b, "300"]).await;
                println!("adb swipe round trip {:?}", t.elapsed());
            }
        });
        let (mut bytes, mut chunk, start) = (Vec::new(), vec![0u8; 65536], Instant::now());
        while start.elapsed() < Duration::from_secs(24) {
            if let Ok(Ok(n)) = tokio::time::timeout(Duration::from_secs(1), client.read(&mut chunk)).await {
                bytes.extend_from_slice(&chunk[..n]);
            }
        }
        println!("frames {} in 24s, {} bytes", bytes.windows(8).filter(|w| w == b"--frame\r").count(), bytes.len());
        let _ = mover.await;
        stop_session();
        server.abort();
    }

    #[tokio::test]
    #[ignore] // needs a booted iOS simulator with a focused text field: pastes Turkish text
    async fn ios_paste() {
        let id = "04F12959-54BB-45D2-86E0-17B781D0399B";
        let mut pb = Command::new("xcrun").args(["simctl", "pbcopy", id]).stdin(Stdio::piped()).spawn().unwrap();
        pb.stdin.take().unwrap().write_all("x@y.com şğ".as_bytes()).await.unwrap();
        pb.wait().await.unwrap();
        println!("paste {:?}", ios_session(id).await.unwrap().send(json!({"op": "paste"})).await);
        stop_session();
    }

    #[tokio::test]
    #[ignore] // needs a booted iOS simulator with a focused text field and a Turkish Mac keyboard layout
    async fn ios_type() {
        let id = "04F12959-54BB-45D2-86E0-17B781D0399B";
        let ios = ios_session(id).await.unwrap();
        for chunk in std::env::var("CHUNKS").unwrap().split('|') {
            println!("type {chunk} {:?}", ios.send(json!({"op": "type", "text": chunk})).await);
            tokio::time::sleep(Duration::from_millis(800)).await;
            let _ = run("xcrun", &["simctl", "io", id, "screenshot", "--type=jpeg", &format!("{}/chunk-{}.jpg", std::env::var("OUT").unwrap(), chunk.chars().next().map(|c| c as u32).unwrap_or(0))]).await;
            ios.send(json!({"op": "key", "code": 42})).await.ok();
        }
        stop_session();
    }
}
