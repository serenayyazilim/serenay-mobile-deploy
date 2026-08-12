use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// In sermobileboss (multi sub-app) mode, each workspace has its own bundle ID
/// prefix and Android keystore signing identity — these are not hardcoded,
/// they are read from/written to this file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SermobilebossConfig {
    #[serde(rename = "bundleIdPrefix")]
    pub bundle_id_prefix: String,
    #[serde(rename = "keystoreAliasPrefix")]
    pub keystore_alias_prefix: String,
    #[serde(rename = "keystorePassword")]
    pub keystore_password: String,
    #[serde(rename = "keystoreCommonName")]
    pub keystore_common_name: String,
    #[serde(rename = "keystoreOrgUnit")]
    pub keystore_org_unit: String,
    #[serde(rename = "keystoreOrgName")]
    pub keystore_org_name: String,
    #[serde(rename = "keystoreLocality")]
    pub keystore_locality: String,
    #[serde(rename = "keystoreState")]
    pub keystore_state: String,
    #[serde(rename = "keystoreCountry")]
    pub keystore_country: String,
}

fn config_path(workspace: &str) -> PathBuf {
    Path::new(workspace).join("sermobileboss_config.json")
}

pub fn read_sermobileboss_config(workspace: &str) -> Option<SermobilebossConfig> {
    let file = config_path(workspace);
    if !file.exists() {
        return None;
    }
    let content = std::fs::read_to_string(file).ok()?;
    let config: SermobilebossConfig = serde_json::from_str(&content).ok()?;
    if config.bundle_id_prefix.is_empty()
        || config.keystore_alias_prefix.is_empty()
        || config.keystore_password.is_empty()
    {
        return None;
    }
    Some(config)
}

pub fn write_sermobileboss_config(workspace: &str, config: &SermobilebossConfig) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(config_path(workspace), json)
}

pub fn build_keystore_dname(config: &SermobilebossConfig) -> String {
    format!(
        "CN={}, OU={}, O={}, L={}, S={}, C={}",
        config.keystore_common_name,
        config.keystore_org_unit,
        config.keystore_org_name,
        config.keystore_locality,
        config.keystore_state,
        config.keystore_country
    )
}

pub const MISSING_CONFIG_MESSAGE: &str =
    "Bundle ID / keystore settings are missing. Fill them in via \"Workspace Settings\" in the top menu.";
