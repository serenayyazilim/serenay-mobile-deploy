//! The token manager in a browser: the `token_manager_link` MCP tool (needs the `admin` permission)
//! makes a link, and `GET /tokens/<session>` (page) plus `GET|POST /tokens/<session>/api` work only
//! while that session is live (15 minutes, one at a time) and only from the local network. The
//! page asks for the main password (`MCP_MAIN_PASSWORD` in `.env`, see mcp_access) with every call;
//! 5 wrong passwords end the session. The session in the link is never logged.

use crate::mcp_access;
use crate::sim::{is_lan, read, reply};
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;

const LIFETIME: Duration = Duration::from_secs(15 * 60);
static SESSION: Mutex<Option<(String, Instant)>> = Mutex::new(None);

/// A new link (the previous one stops working).
pub fn new_link(ip: Option<&str>) -> String {
    let session = format!("{}{}", uuid::Uuid::new_v4().simple(), uuid::Uuid::new_v4().simple());
    *SESSION.lock().unwrap() = Some((session.clone(), Instant::now()));
    *WRONG.lock().unwrap() = 0;
    format!("http://{}:{}/tokens/{session}", ip.unwrap_or("<this Mac's IP>"), crate::mcp::PORT)
}

static WRONG: Mutex<u8> = Mutex::new(0);

/// Counts a wrong password; the fifth ends the session. True when it did.
fn wrong_password() -> bool {
    let mut n = WRONG.lock().unwrap();
    *n += 1;
    if *n >= 5 {
        *SESSION.lock().unwrap() = None;
        return true;
    }
    false
}

fn live(session: &str) -> bool {
    SESSION.lock().unwrap().as_ref().is_some_and(|(s, at)| s == session && at.elapsed() < LIFETIME)
}

pub async fn serve(mut stream: TcpStream, peer: SocketAddr) {
    if !is_lan(peer.ip()) {
        return reply(&mut stream, "403 Forbidden", "text/plain", b"").await;
    }
    let Some((method, path, body)) = read(&mut stream).await else {
        return reply(&mut stream, "400 Bad Request", "text/plain", b"").await;
    };
    let mut parts = path.trim_start_matches("/tokens/").splitn(2, '/');
    let (session, rest) = (parts.next().unwrap_or(""), parts.next().unwrap_or(""));
    if !live(session) {
        return reply(&mut stream, "404 Not Found", "text/plain", b"Link expired or unknown; ask for a new one with token_manager_link.").await;
    }
    match (method.as_str(), rest) {
        ("GET", "") => reply(&mut stream, "200 OK", "text/html; charset=utf-8", PAGE.as_bytes()).await,
        ("POST", "api") => {
            let result = serde_json::from_slice::<Value>(&body).map_err(|e| e.to_string()).and_then(|b| {
                let text = |k: &str| b.get(k).and_then(Value::as_str).unwrap_or("").to_string();
                if !mcp_access::password_matches(&text("password")) {
                    if wrong_password() {
                        return Err("Too many wrong passwords; the link is closed. Ask for a new one.".to_string());
                    }
                    return Err("Wrong password.".to_string());
                }
                let list = match text("action").as_str() {
                    "list" => Ok(mcp_access::list_tokens()),
                    "set_enabled" => {
                        mcp_access::set_enabled(b.get("on").and_then(Value::as_bool).unwrap_or(true));
                        Ok(mcp_access::list_tokens())
                    }
                    action => mcp_access::apply(action, &text("name"), b.get("permissions").and_then(Value::as_array).map(Vec::as_slice)),
                };
                list.map(|mut v| {
                    v["enabled"] = json!(mcp_access::enabled());
                    v
                })
            });
            match result {
                Ok(list) => reply(&mut stream, "200 OK", "application/json", list.to_string().as_bytes()).await,
                Err(e) => reply(&mut stream, "422 Unprocessable Entity", "application/json", json!({ "error": e }).to_string().as_bytes()).await,
            }
        }
        _ => reply(&mut stream, "404 Not Found", "text/plain", b"Not found").await,
    }
}

const PAGE: &str = include_str!("token_page.html");

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    async fn call(port: u16, path: &str, body: Option<Value>) -> (String, String) {
        let mut s = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
        let (method, body) = body.map_or(("GET", String::new()), |b| ("POST", b.to_string()));
        s.write_all(format!("{method} {path} HTTP/1.1\r\nContent-Length: {}\r\n\r\n{body}", body.len()).as_bytes()).await.unwrap();
        let mut out = String::new();
        s.read_to_string(&mut out).await.unwrap();
        let (head, body) = out.split_once("\r\n\r\n").unwrap();
        (head.lines().next().unwrap().to_string(), body.to_string())
    }

    #[tokio::test]
    async fn the_page_needs_the_link_and_the_main_password() {
        let home = std::env::temp_dir().join(format!("tp-{}", uuid::Uuid::new_v4().simple()));
        std::fs::create_dir_all(&home).unwrap();
        std::env::set_var("HOME", &home); // tokens and .env go to a throwaway config dir
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            while let Ok((stream, peer)) = listener.accept().await {
                tokio::spawn(serve(stream, peer));
            }
        });
        let link = new_link(Some("127.0.0.1"));
        let path = format!("/tokens/{}", link.rsplit('/').next().unwrap());
        let api = format!("{path}/api");

        assert!(call(port, "/tokens/wrong", None).await.0.contains("404"));
        let (status, page) = call(port, &path, None).await;
        assert!(status.contains("200") && page.contains("MCP tokens"));

        let password = mcp_access::main_password();
        assert!(call(port, &api, Some(json!({ "action": "list", "password": "nope" }))).await.0.contains("422"));
        let (status, body) = call(port, &api, Some(json!({ "action": "list", "password": password }))).await;
        assert!(status.contains("200") && body.contains("default"));

        let (_, body) = call(port, &api, Some(json!({ "action": "set_enabled", "on": false, "password": password }))).await;
        assert!(body.contains("\"enabled\":false"), "{body}");
        let (_, body) = call(port, &api, Some(json!({ "action": "set_enabled", "on": true, "password": password }))).await;
        assert!(body.contains("\"enabled\":true"), "{body}");

        let (_, body) = call(port, &api, Some(json!({ "action": "create", "name": "ci", "permissions": ["read", "admin"], "password": password }))).await;
        assert!(body.contains("\"ci\""));
        let (status, body) = call(port, &api, Some(json!({ "action": "delete", "name": "default", "password": password }))).await;
        assert!(status.contains("200"), "{body}"); // ci has admin too, so default may go
        let (status, body) = call(port, &api, Some(json!({ "action": "delete", "name": "ci", "password": password }))).await;
        assert!(status.contains("422") && body.contains("last"), "{body}");

        for _ in 0..5 {
            call(port, &api, Some(json!({ "action": "list", "password": "bad" }))).await;
        }
        assert!(call(port, &path, None).await.0.contains("404")); // five wrong passwords closed the link
    }
}
