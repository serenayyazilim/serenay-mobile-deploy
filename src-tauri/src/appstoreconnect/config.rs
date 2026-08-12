use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const CONFIG_FILENAME: &str = "sermobileboss_appstoreconnect.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AscConfig {
    #[serde(rename = "issuerId")]
    pub issuer_id: String,
    #[serde(rename = "keyId")]
    pub key_id: String,
    #[serde(rename = "privateKey")]
    pub private_key: String,
}

fn config_path(workspace: &str) -> PathBuf {
    Path::new(workspace).join(CONFIG_FILENAME)
}

pub fn read_asc_config(workspace: &str) -> Option<AscConfig> {
    let content = std::fs::read_to_string(config_path(workspace)).ok()?;
    let config: AscConfig = serde_json::from_str(&content).ok()?;
    if config.issuer_id.is_empty() || config.key_id.is_empty() || config.private_key.is_empty() {
        return None;
    }
    Some(config)
}

pub fn write_asc_config(workspace: &str, config: &AscConfig) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(config_path(workspace), json)
}

pub fn delete_asc_config(workspace: &str) {
    let _ = std::fs::remove_file(config_path(workspace));
}
