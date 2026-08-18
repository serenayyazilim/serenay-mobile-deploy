use crate::slack::config::{delete_slack_config, read_slack_config, write_slack_config, SlackConfig};
use serde::Serialize;
use serde_json::{json, Value};

fn resolve_webhook_url(workspace: &str) -> Option<String> {
    if let Some(config) = read_slack_config(workspace) {
        return Some(config.webhook_url);
    }
    std::env::var("SLACK_WEBHOOK_URL").ok().filter(|v| !v.is_empty())
}

async fn post_to_slack(webhook_url: &str, payload: Value) -> Result<(), String> {
    let client = reqwest::Client::new();
    let res = client.post(webhook_url).json(&payload).send().await.map_err(|e| e.to_string())?;

    if res.status().is_success() {
        Ok(())
    } else {
        let text = res.text().await.unwrap_or_default();
        Err(format!("Slack error: {text}"))
    }
}

#[derive(Debug, Serialize)]
pub struct SlackConfigStatus {
    pub configured: bool,
}

#[tauri::command]
pub fn slack_config_get(workspace: String) -> SlackConfigStatus {
    SlackConfigStatus { configured: read_slack_config(&workspace).is_some() }
}

#[tauri::command]
pub fn slack_config_save(workspace: String, webhook_url: String) -> Result<(), String> {
    let webhook_url = webhook_url.trim().to_string();
    if webhook_url.is_empty() {
        return Err("Webhook URL is required".to_string());
    }
    write_slack_config(&workspace, &SlackConfig { webhook_url }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn slack_config_delete(workspace: String) {
    delete_slack_config(&workspace);
}

#[tauri::command]
pub async fn slack_test(workspace: String) -> Result<(), String> {
    let webhook_url = resolve_webhook_url(&workspace).ok_or("Slack webhook is not configured")?;
    post_to_slack(
        &webhook_url,
        json!({
            "blocks": [
                {
                    "type": "section",
                    "text": { "type": "mrkdwn", "text": "✅ *Serenay Mobile Deploy* is connected to this channel." }
                }
            ]
        }),
    )
    .await
}

#[tauri::command]
pub async fn slack_notify(
    workspace: String,
    project_name: String,
    project_id: String,
    platform: String,
    version: String,
    success: bool,
    message: Option<String>,
    duration: Option<u64>,
) -> Result<(), String> {
    let webhook_url = resolve_webhook_url(&workspace).ok_or("Slack webhook is not configured")?;

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

    post_to_slack(&webhook_url, json!({ "blocks": blocks })).await
}
