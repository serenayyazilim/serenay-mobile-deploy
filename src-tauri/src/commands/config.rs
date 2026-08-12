use crate::config::{colors, serconf};
use serde_json::Value;
use std::collections::BTreeMap;

#[tauri::command]
pub fn config_colors_get(workspace: String, project_id: String) -> BTreeMap<String, String> {
    colors::read_colors(&workspace, &project_id)
}

#[tauri::command]
pub fn config_colors_save(workspace: String, project_id: String, colors: BTreeMap<String, String>) -> Result<(), String> {
    colors::write_colors(&workspace, &project_id, &colors).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn config_serconf_get(workspace: String, project_id: String) -> Result<BTreeMap<String, Value>, String> {
    serconf::read_serconf(&workspace, &project_id).ok_or_else(|| "serconf.dart not found".to_string())
}

#[tauri::command]
pub fn config_serconf_save(workspace: String, project_id: String, config: BTreeMap<String, Value>) -> Result<(), String> {
    serconf::write_serconf(&workspace, &project_id, &config)
}
