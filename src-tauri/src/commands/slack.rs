use serde_json::json;

#[tauri::command]
pub async fn slack_notify(
    project_name: String,
    project_id: String,
    platform: String,
    version: String,
    success: bool,
    message: Option<String>,
    duration: Option<u64>,
) -> Result<(), String> {
    let webhook_url = std::env::var("SLACK_WEBHOOK_URL").unwrap_or_default();
    if webhook_url.is_empty() {
        return Err("Slack webhook URL tanımlı değil".to_string());
    }

    let (emoji, text) = match platform.as_str() {
        "ios" => ("🍎", "App Store"),
        "android" => ("🤖", "Google Play"),
        "huawei" => ("📱", "AppGallery"),
        "all" => ("🚀", "Tüm Platformlar"),
        _ => ("📦", platform.as_str()),
    };

    let duration_text = duration.map(|d| format!("{}dk {}sn", d / 60, d % 60)).unwrap_or_default();

    let mut blocks = vec![
        json!({
            "type": "header",
            "text": { "type": "plain_text", "text": if success { "✅ Deploy Başarılı!" } else { "❌ Deploy Başarısız!" }, "emoji": true }
        }),
        json!({
            "type": "section",
            "fields": [
                { "type": "mrkdwn", "text": format!("*Uygulama:*\n{project_name}") },
                { "type": "mrkdwn", "text": format!("*Platform:*\n{emoji} {text}") },
                { "type": "mrkdwn", "text": format!("*Versiyon:*\n{version}") },
                { "type": "mrkdwn", "text": format!("*Proje ID:*\n{project_id}") },
            ]
        }),
    ];

    if !duration_text.is_empty() {
        blocks.push(json!({
            "type": "context",
            "elements": [{ "type": "mrkdwn", "text": format!("⏱️ Süre: {duration_text}") }]
        }));
    }

    if let Some(msg) = &message {
        if !success {
            blocks.push(json!({
                "type": "section",
                "text": { "type": "mrkdwn", "text": format!("*Hata:*\n```{msg}```") }
            }));
        }
    }

    let now = chrono::Local::now().format("%d.%m.%Y %H:%M:%S").to_string();
    blocks.push(json!({
        "type": "context",
        "elements": [{ "type": "mrkdwn", "text": format!("📅 {now}") }]
    }));

    let client = reqwest::Client::new();
    let res = client
        .post(&webhook_url)
        .json(&json!({ "blocks": blocks }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().is_success() {
        Ok(())
    } else {
        let text = res.text().await.unwrap_or_default();
        Err(format!("Slack hatası: {text}"))
    }
}
