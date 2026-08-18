use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const CONFIG_FILENAME: &str = "sermobileboss_slack.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackConfig {
    #[serde(rename = "webhookUrl")]
    pub webhook_url: String,
}

fn config_path(workspace: &str) -> PathBuf {
    Path::new(workspace).join(CONFIG_FILENAME)
}

pub fn read_slack_config(workspace: &str) -> Option<SlackConfig> {
    let content = std::fs::read_to_string(config_path(workspace)).ok()?;
    let config: SlackConfig = serde_json::from_str(&content).ok()?;
    if config.webhook_url.is_empty() {
        return None;
    }
    Some(config)
}

pub fn write_slack_config(workspace: &str, config: &SlackConfig) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(config_path(workspace), json)
}

pub fn delete_slack_config(workspace: &str) {
    let _ = std::fs::remove_file(config_path(workspace));
}
