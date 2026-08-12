use crate::appstoreconnect::client::{self, AscApiError};
use crate::appstoreconnect::config::{delete_asc_config, read_asc_config, write_asc_config, AscConfig};
use reqwest::Method;
use serde::Serialize;
use serde_json::{json, Value};

const NOT_CONFIGURED: &str = "App Store Connect API credentials not set";

fn to_err(e: AscApiError) -> String {
    e.message
}

fn require_config(workspace: &str) -> Result<AscConfig, String> {
    read_asc_config(workspace).ok_or_else(|| NOT_CONFIGURED.to_string())
}

#[derive(Debug, Serialize)]
pub struct AscConfigStatus {
    pub configured: bool,
    #[serde(rename = "issuerId", skip_serializing_if = "Option::is_none")]
    pub issuer_id: Option<String>,
    #[serde(rename = "keyId", skip_serializing_if = "Option::is_none")]
    pub key_id: Option<String>,
}

#[tauri::command]
pub fn asc_config_get(workspace: String) -> AscConfigStatus {
    match read_asc_config(&workspace) {
        Some(config) => AscConfigStatus { configured: true, issuer_id: Some(config.issuer_id), key_id: Some(config.key_id) },
        None => AscConfigStatus { configured: false, issuer_id: None, key_id: None },
    }
}

#[tauri::command]
pub async fn asc_config_save(workspace: String, issuer_id: String, key_id: String, private_key: String) -> Result<(), String> {
    let config = AscConfig {
        issuer_id: issuer_id.trim().to_string(),
        key_id: key_id.trim().to_string(),
        private_key: private_key.trim().to_string(),
    };

    client::asc_fetch(&config, Method::GET, "/v1/apps?limit=1", None)
        .await
        .map_err(|e| format!("Failed to verify API key: {}", e.message))?;

    write_asc_config(&workspace, &config).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn asc_config_delete(workspace: String) {
    delete_asc_config(&workspace);
}

#[tauri::command]
pub async fn asc_events_list(workspace: String, bundle_id: String) -> Result<Value, String> {
    let config = require_config(&workspace)?;
    let app = client::find_app_by_bundle_id(&config, &bundle_id)
        .await
        .map_err(to_err)?
        .ok_or_else(|| format!("This bundle ID was not found in App Store Connect: {bundle_id}"))?;
    let app_id = app.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();

    let events = client::list_app_events(&config, &app_id).await.map_err(to_err)?;
    Ok(json!({
        "appId": app_id,
        "events": events.get("data").cloned().unwrap_or(json!([])),
        "included": events.get("included").cloned().unwrap_or(json!([])),
    }))
}

#[tauri::command]
pub async fn asc_event_create(
    workspace: String,
    bundle_id: String,
    attributes: Value,
    primary_localization: Option<Value>,
) -> Result<Value, String> {
    let config = require_config(&workspace)?;
    let app = client::find_app_by_bundle_id(&config, &bundle_id)
        .await
        .map_err(to_err)?
        .ok_or_else(|| format!("This bundle ID was not found in App Store Connect: {bundle_id}"))?;
    let app_id = app.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();

    let event = client::create_app_event(&config, &app_id, attributes).await.map_err(to_err)?;
    let event_id = event.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();

    if let Some(loc) = primary_localization {
        if loc.get("locale").and_then(|v| v.as_str()).is_some() {
            client::create_app_event_localization(&config, &event_id, loc).await.map_err(to_err)?;
        }
    }

    Ok(json!({ "event": event, "appId": app_id }))
}

#[tauri::command]
pub async fn asc_event_get(workspace: String, id: String) -> Result<Value, String> {
    let config = require_config(&workspace)?;
    let data = client::get_app_event(&config, &id).await.map_err(to_err)?;
    Ok(json!({
        "event": data.get("data").cloned().unwrap_or(Value::Null),
        "included": data.get("included").cloned().unwrap_or(json!([])),
    }))
}

#[tauri::command]
pub async fn asc_event_update(workspace: String, id: String, attributes: Value) -> Result<Value, String> {
    let config = require_config(&workspace)?;
    client::update_app_event(&config, &id, attributes).await.map_err(to_err)
}

#[tauri::command]
pub async fn asc_event_delete(workspace: String, id: String) -> Result<(), String> {
    let config = require_config(&workspace)?;
    client::delete_app_event(&config, &id).await.map_err(to_err)
}

#[tauri::command]
pub async fn asc_event_submit(workspace: String, id: String, app_id: String) -> Result<Value, String> {
    let config = require_config(&workspace)?;
    client::submit_app_event_for_review(&config, &app_id, &id, "IOS").await.map_err(to_err)
}

#[tauri::command]
pub async fn asc_localization_create(workspace: String, app_event_id: String, attributes: Value) -> Result<Value, String> {
    let config = require_config(&workspace)?;
    client::create_app_event_localization(&config, &app_event_id, attributes).await.map_err(to_err)
}

#[tauri::command]
pub async fn asc_localization_update(workspace: String, id: String, attributes: Value) -> Result<Value, String> {
    let config = require_config(&workspace)?;
    client::update_app_event_localization(&config, &id, attributes).await.map_err(to_err)
}

#[tauri::command]
pub async fn asc_localization_delete(workspace: String, id: String) -> Result<(), String> {
    let config = require_config(&workspace)?;
    client::delete_app_event_localization(&config, &id).await.map_err(to_err)
}

#[tauri::command]
pub async fn asc_localization_screenshots(workspace: String, id: String) -> Result<Vec<Value>, String> {
    let config = require_config(&workspace)?;
    client::list_localization_screenshots(&config, &id).await.map_err(to_err)
}

#[tauri::command]
pub async fn asc_screenshot_upload(
    workspace: String,
    localization_id: String,
    asset_type: String,
    file_path: String,
) -> Result<Value, String> {
    let config = require_config(&workspace)?;
    let bytes = std::fs::read(&file_path).map_err(|e| e.to_string())?;
    let file_name = std::path::Path::new(&file_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "screenshot.png".to_string());

    client::upload_app_event_screenshot(&config, &localization_id, &asset_type, &file_name, &bytes)
        .await
        .map_err(to_err)
}

#[tauri::command]
pub async fn asc_screenshot_delete(workspace: String, id: String) -> Result<(), String> {
    let config = require_config(&workspace)?;
    client::delete_app_event_screenshot(&config, &id).await.map_err(to_err)
}

#[tauri::command]
pub async fn asc_territories_list(workspace: String) -> Result<Vec<String>, String> {
    let config = require_config(&workspace)?;
    let territories = client::list_territories(&config).await.map_err(to_err)?;
    Ok(territories.into_iter().filter_map(|t| t.get("id").and_then(|v| v.as_str()).map(String::from)).collect())
}
