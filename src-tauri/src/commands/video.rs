//! A loopback-only HTTP server for the corner video player's page. YouTube rejects embeds that
//! come without a Referer (error 153), and the app's `tauri://` page sends none; a page served from
//! `http://127.0.0.1` does.

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::OnceCell;

const PAGE: &str = include_str!("video_player.html");

static PORT: OnceCell<u16> = OnceCell::const_new();

/// Port of the player page server, started on first use and kept for the app's lifetime.
#[tauri::command]
pub async fn video_player_port() -> Result<u16, String> {
    PORT.get_or_try_init(|| async {
        let listener = TcpListener::bind("127.0.0.1:0").await.map_err(|e| e.to_string())?;
        let port = listener.local_addr().map_err(|e| e.to_string())?.port();
        tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                tokio::spawn(serve_page(stream));
            }
        });
        Ok(port)
    })
    .await
    .copied()
}

/// Answers any request with the static player page; the page reads its query string itself.
async fn serve_page(mut stream: TcpStream) {
    let mut request = [0u8; 4096];
    let _ = stream.read(&mut request).await;
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n{PAGE}",
        PAGE.len()
    );
    let _ = stream.write_all(response.as_bytes()).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn serves_the_player_page() {
        let port = video_player_port().await.unwrap();
        assert_eq!(video_player_port().await.unwrap(), port);

        let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
        stream.write_all(b"GET /?v=rFZHOHl-L8A&volume=35 HTTP/1.1\r\nHost: x\r\n\r\n").await.unwrap();
        let mut body = String::new();
        stream.read_to_string(&mut body).await.unwrap();
        assert!(body.starts_with("HTTP/1.1 200 OK"));
        assert!(body.contains("youtube-nocookie.com/embed/"));
    }
}
