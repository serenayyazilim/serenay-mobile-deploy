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
        return Err("SLACK_WEBHOOK_URL is not set".to_string());
    }

    let (emoji, text) = match platform.as_str() {
        "ios" => ("🍎", "App Store"),
        "android" => ("🤖", "Google Play"),
        "huawei" => ("📱", "AppGallery"),
        "all" => ("🚀", "All Platforms"),
        _ => ("📦", platform.as_str()),
    };

    let duration_text = duration.map(|d| format!("{}m {}s", d / 60, d % 60)).unwrap_or_default();

    let mut blocks = vec![
        json!({
            "type": "header",
            "text": { "type": "plain_text", "text": if success { "✅ Deploy Succeeded!" } else { "❌ Deploy Failed!" }, "emoji": true }
        }),
        json!({
            "type": "section",
            "fields": [
                { "type": "mrkdwn", "text": format!("*App:*\n{project_name}") },
                { "type": "mrkdwn", "text": format!("*Platform:*\n{emoji} {text}") },
                { "type": "mrkdwn", "text": format!("*Version:*\n{version}") },
                { "type": "mrkdwn", "text": format!("*Project ID:*\n{project_id}") },
            ]
        }),
    ];

    if !duration_text.is_empty() {
        blocks.push(json!({
            "type": "context",
            "elements": [{ "type": "mrkdwn", "text": format!("⏱️ Duration: {duration_text}") }]
        }));
    }

    if let Some(msg) = &message {
        if !success {
            blocks.push(json!({
                "type": "section",
                "text": { "type": "mrkdwn", "text": format!("*Error:*\n```{msg}```") }
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
        Err(format!("Slack error: {text}"))
    }
}
