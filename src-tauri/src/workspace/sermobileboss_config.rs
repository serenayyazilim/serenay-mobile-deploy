use super::detect::detect_workspace_mode;
use super::types::WorkspaceMode;
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
    /// Apps below this version jump up to it on their next deploy. Empty turns it off.
    #[serde(rename = "minVersion", default = "default_min_version")]
    pub min_version: String,
}

fn default_min_version() -> String {
    "19.0.0".to_string()
}

/// The minimum version a deploy applies. Only sermobileboss (serMobilePro) workspaces have one.
pub fn min_version(workspace: &str) -> String {
    if detect_workspace_mode(workspace) != WorkspaceMode::Sermobileboss {
        return String::new();
    }
    read_sermobileboss_config(workspace).map_or_else(default_min_version, |c| c.min_version)
}

pub fn is_valid_min_version(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    version.is_empty()
        || (parts.len() == 3 && parts.iter().all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit())))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn min_version_is_empty_or_three_numbers() {
        assert!(is_valid_min_version(""));
        assert!(is_valid_min_version("19.0.0"));
        assert!(!is_valid_min_version("19.0"));
        assert!(!is_valid_min_version("19.0.0+1"));
        assert!(!is_valid_min_version("19..0"));
    }

    #[test]
    fn min_version_defaults_for_saved_configs_without_it() {
        let dir = std::env::temp_dir().join(format!("min-version-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let workspace = dir.to_str().unwrap();
        assert_eq!(min_version(workspace), "", "generic workspaces have no minimum");

        std::fs::write(dir.join("sermobileboss_projects.json"), "{}").unwrap();
        assert_eq!(min_version(workspace), "19.0.0");

        let config = r#"{"bundleIdPrefix":"a","keystoreAliasPrefix":"a","keystorePassword":"a","keystoreCommonName":"a",
            "keystoreOrgUnit":"a","keystoreOrgName":"a","keystoreLocality":"a","keystoreState":"a","keystoreCountry":"a"}"#;
        std::fs::write(dir.join("sermobileboss_config.json"), config).unwrap();
        assert_eq!(min_version(workspace), "19.0.0");

        std::fs::write(dir.join("sermobileboss_config.json"), config.replacen('{', r#"{"minVersion":"","#, 1)).unwrap();
        assert_eq!(min_version(workspace), "");
        std::fs::remove_dir_all(dir).unwrap();
    }
}
