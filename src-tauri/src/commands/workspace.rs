use crate::workspace::detect::detect_workspace_mode;
use crate::workspace::sermobileboss_config::{read_sermobileboss_config, write_sermobileboss_config, SermobilebossConfig};
use crate::workspace::types::WorkspaceMode;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri_plugin_dialog::DialogExt;

const MAX_RECENT: usize = 5;

fn recent_file_path() -> PathBuf {
    dirs::home_dir()
        .expect("could not find home directory")
        .join(".sermobile-recent-workspaces.json")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentWorkspace {
    pub path: String,
    pub name: String,
    #[serde(rename = "lastUsed")]
    pub last_used: String,
}

fn load_recent() -> Vec<RecentWorkspace> {
    let file = recent_file_path();
    let Ok(content) = std::fs::read_to_string(file) else {
        return vec![];
    };
    serde_json::from_str(&content).unwrap_or_default()
}

fn save_recent(recent: &[RecentWorkspace]) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(recent)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(recent_file_path(), json)
}

/// Opens a native folder picker dialog on macOS/Windows/Linux.
/// Replaces the old (Next.js/osascript, macOS-only) implementation.
#[tauri::command]
pub async fn workspace_browse(app: tauri::AppHandle) -> Option<String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .set_title("Select Flutter Project Folder")
        .pick_folder(move |folder| {
            let _ = tx.send(folder);
        });

    match rx.await {
        Ok(Some(path)) => {
            // Strip trailing slash from the path (matches old behavior)
            let s = path.to_string();
            Some(s.trim_end_matches('/').to_string())
        }
        _ => None,
    }
}

#[derive(Debug, Serialize)]
pub struct ValidateResult {
    pub valid: bool,
    pub message: String,
    pub mode: Option<String>,
    #[serde(rename = "projectName")]
    pub project_name: Option<String>,
    #[serde(rename = "projectCount")]
    pub project_count: Option<u32>,
    pub path: Option<String>,
}

#[tauri::command]
pub fn workspace_validate(workspace_path: String) -> ValidateResult {
    let root = Path::new(&workspace_path);

    if !root.exists() {
        return ValidateResult {
            valid: false,
            message: "Folder not found".to_string(),
            mode: None,
            project_name: None,
            project_count: None,
            path: None,
        };
    }

    let pubspec_path = root.join("pubspec.yaml");
    if !pubspec_path.exists() {
        return ValidateResult {
            valid: false,
            message: "This is not a Flutter project (pubspec.yaml not found)".to_string(),
            mode: None,
            project_name: None,
            project_count: None,
            path: None,
        };
    }

    let mode = detect_workspace_mode(&workspace_path);

    let mut project_name = "Flutter Project".to_string();
    if let Ok(content) = std::fs::read_to_string(&pubspec_path) {
        let re = Regex::new(r"(?m)^name:\s*(.+)$").unwrap();
        if let Some(c) = re.captures(&content) {
            project_name = c[1].trim().to_string();
        }
    }

    let mut project_count: u32 = 1;
    if mode == WorkspaceMode::Sermobileboss {
        let projects_json_path = root.join("sermobileboss_projects.json");
        if let Ok(content) = std::fs::read_to_string(&projects_json_path) {
            if let Ok(data) = serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&content) {
                project_count = data.len() as u32;
            }
        }
    }

    let mode_str = match mode {
        WorkspaceMode::Sermobileboss => "sermobileboss",
        WorkspaceMode::Generic => "generic",
    };

    ValidateResult {
        valid: true,
        message: if mode == WorkspaceMode::Sermobileboss {
            "Valid serMobilePro project".to_string()
        } else {
            "Flutter project detected".to_string()
        },
        mode: Some(mode_str.to_string()),
        project_name: Some(project_name),
        project_count: Some(project_count),
        path: Some(workspace_path),
    }
}

#[tauri::command]
pub fn workspace_recent_get() -> Vec<RecentWorkspace> {
    load_recent()
        .into_iter()
        .filter(|r| Path::new(&r.path).exists())
        .collect()
}

#[tauri::command]
pub fn workspace_recent_add(path: String, name: Option<String>) -> Result<Vec<RecentWorkspace>, String> {
    let mut recent = load_recent();
    recent.retain(|r| r.path != path);

    let display_name = name.unwrap_or_else(|| {
        Path::new(&path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.clone())
    });

    recent.insert(
        0,
        RecentWorkspace {
            path,
            name: display_name,
            last_used: chrono::Utc::now().to_rfc3339(),
        },
    );
    recent.truncate(MAX_RECENT);

    save_recent(&recent).map_err(|e| e.to_string())?;
    Ok(recent)
}

#[derive(Debug, Serialize)]
pub struct SermobilebossConfigStatus {
    pub configured: bool,
    pub config: Option<SermobilebossConfig>,
}

#[tauri::command]
pub fn workspace_config_get(workspace: String) -> SermobilebossConfigStatus {
    let config = read_sermobileboss_config(&workspace);
    SermobilebossConfigStatus { configured: config.is_some(), config }
}

#[tauri::command]
pub fn workspace_config_save(workspace: String, config: SermobilebossConfig) -> Result<(), String> {
    let required = [
        &config.bundle_id_prefix,
        &config.keystore_alias_prefix,
        &config.keystore_password,
        &config.keystore_common_name,
        &config.keystore_org_unit,
        &config.keystore_org_name,
        &config.keystore_locality,
        &config.keystore_state,
        &config.keystore_country,
    ];
    if required.iter().any(|v| v.trim().is_empty()) {
        return Err("All fields are required".to_string());
    }
    write_sermobileboss_config(&workspace, &config).map_err(|e| e.to_string())
}
