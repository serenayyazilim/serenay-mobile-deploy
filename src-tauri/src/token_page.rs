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
                match text("action").as_str() {
                    "list" => Ok(mcp_access::list_tokens()),
                    action => mcp_access::apply(action, &text("name"), b.get("permissions").and_then(Value::as_array).map(Vec::as_slice)),
                }
            });
            match result {
                Ok(list) => reply(&mut stream, "200 OK", "application/json", list.to_string().as_bytes()).await,
                Err(e) => reply(&mut stream, "422 Unprocessable Entity", "application/json", json!({ "error": e }).to_string().as_bytes()).await,
            }
        }
        _ => reply(&mut stream, "404 Not Found", "text/plain", b"Not found").await,
    }
}

const PAGE: &str = r##"<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>MCP tokens</title>
<style>
:root{color-scheme:light dark;--bg:#fff;--fg:#111;--mut:#666;--line:#ddd;--acc:#2563eb}
@media(prefers-color-scheme:dark){:root{--bg:#111;--fg:#eee;--mut:#999;--line:#333;--acc:#60a5fa}}
body{margin:0;padding:16px;background:var(--bg);color:var(--fg);font:14px system-ui,sans-serif;max-width:720px;margin-inline:auto}
h1{font-size:18px}.card{border:1px solid var(--line);border-radius:10px;padding:12px;margin:12px 0}
code{word-break:break-all}label{display:inline-flex;gap:4px;margin:2px 10px 2px 0;align-items:center}
button{font:inherit;border:1px solid var(--line);background:none;color:var(--fg);border-radius:6px;padding:4px 10px;cursor:pointer}
button.p{background:var(--acc);border-color:var(--acc);color:#fff}input[type=text],input[type=password]{font:inherit;padding:4px 8px;border:1px solid var(--line);border-radius:6px;background:none;color:var(--fg)}
.row{display:flex;gap:8px;flex-wrap:wrap;align-items:center;margin-top:8px}.mut{color:var(--mut)}#err{color:#dc2626}
</style></head><body>
<h1>MCP tokens</h1><p class="mut">Each token may call only the tools its permissions allow. Changes apply to the next request.</p>
<div id="err"></div>
<div class="card" id="login"><b>Main password</b><div class="row"><input type="password" id="pw" placeholder="MCP_MAIN_PASSWORD"><button class="p" id="go">Open</button></div></div>
<div id="list"></div>
<div class="card"><b>New token</b><div class="row"><input type="text" id="name" placeholder="name"></div><div id="newperms" class="row"></div><div class="row"><button class="p" id="create">Create</button></div></div>
<script>
const api=location.pathname.replace(/\/$/,'')+'/api';let perms=[];let pw='';
const $=(t,p={},...k)=>{const e=Object.assign(document.createElement(t),p);k.forEach(c=>e.append(c));return e};
async function call(body){const r=await fetch(api,{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({...body,password:pw})});const j=await r.json();document.getElementById('err').textContent=j.error||'';if(!r.ok)throw 0;return j}
function boxes(sel,on){return perms.map(p=>$('label',{},$('input',{type:'checkbox',value:p,checked:on.includes(p)}),p))}
function checked(box){return[...box.querySelectorAll('input:checked')].map(i=>i.value)}
function draw(j){perms=j.permissions;const L=document.getElementById('list');L.replaceChildren();
for(const t of j.tokens){const boxesEl=$('div',{},...boxes(0,t.permissions||[]));const tok=$('code',{textContent:'•'.repeat(12)});let shown=0;
L.append($('div',{className:'card'},$('b',{textContent:t.name}),$('div',{className:'row'},tok,
$('button',{textContent:'Show',onclick(e){shown=!shown;tok.textContent=shown?t.token:'•'.repeat(12);e.target.textContent=shown?'Hide':'Show'}}),
$('button',{textContent:'Copy',onclick:()=>navigator.clipboard&&navigator.clipboard.writeText(t.token)})),boxesEl,
$('div',{className:'row'},$('button',{className:'p',textContent:'Save permissions',onclick:()=>call({action:'update',name:t.name,permissions:checked(boxesEl)}).then(draw)}),
$('button',{textContent:'New secret',onclick:()=>confirm('The old secret stops working. Continue?')&&call({action:'regenerate',name:t.name}).then(draw)}),
$('button',{textContent:'Delete',onclick:()=>confirm('Delete '+t.name+'?')&&call({action:'delete',name:t.name}).then(draw)}))))}
const np=document.getElementById('newperms');np.replaceChildren(...boxes(0,['read']))}
document.getElementById('create').onclick=()=>call({action:'create',name:document.getElementById('name').value,permissions:checked(document.getElementById('newperms'))}).then(j=>{document.getElementById('name').value='';draw(j)});
document.getElementById('go').onclick=()=>{pw=document.getElementById('pw').value;call({action:'list'}).then(j=>{document.getElementById('login').remove();draw(j)}).catch(()=>{})};
</script></body></html>"##;

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
