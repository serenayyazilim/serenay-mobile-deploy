use super::config::AscConfig;
use super::jwt::generate_asc_token;
use reqwest::Method;
use serde_json::{json, Value};
use std::time::Duration;

const BASE_URL: &str = "https://api.appstoreconnect.apple.com";

#[derive(Debug)]
pub struct AscApiError {
    pub message: String,
    pub status: u16,
}

impl AscApiError {
    fn new(message: impl Into<String>, status: u16) -> Self {
        Self { message: message.into(), status }
    }
}

pub async fn asc_fetch(
    config: &AscConfig,
    method: Method,
    path: &str,
    body: Option<Value>,
) -> Result<Option<Value>, AscApiError> {
    let token = generate_asc_token(config).map_err(|e| AscApiError::new(e, 500))?;
    let url = if path.starts_with("http") { path.to_string() } else { format!("{BASE_URL}{path}") };

    let client = reqwest::Client::new();
    let mut req = client.request(method, &url).bearer_auth(token);
    if let Some(b) = &body {
        req = req.json(b);
    }

    let res = req.send().await.map_err(|e| AscApiError::new(e.to_string(), 500))?;
    let status = res.status().as_u16();

    if status == 204 {
        return Ok(None);
    }

    let text = res.text().await.map_err(|e| AscApiError::new(e.to_string(), 500))?;
    let data: Option<Value> = if text.is_empty() { None } else { serde_json::from_str(&text).ok() };

    if !(200..300).contains(&status) {
        let detail = data
            .as_ref()
            .and_then(|d| d.get("errors"))
            .and_then(|e| e.as_array())
            .map(|errs| {
                errs.iter()
                    .filter_map(|e| e.get("detail").or_else(|| e.get("title")).and_then(|v| v.as_str()))
                    .collect::<Vec<_>>()
                    .join("; ")
            })
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| format!("HTTP {status}"));
        return Err(AscApiError::new(detail, status));
    }

    Ok(data)
}

fn data_field(value: Option<Value>) -> Value {
    value.and_then(|v| v.get("data").cloned()).unwrap_or(Value::Null)
}

// ── Apps ──────────────────────────────────────────────

pub async fn find_app_by_bundle_id(config: &AscConfig, bundle_id: &str) -> Result<Option<Value>, AscApiError> {
    let path = format!("/v1/apps?filter[bundleId]={}&limit=1", urlencoding_encode(bundle_id));
    let data = asc_fetch(config, Method::GET, &path, None).await?;
    let first = data
        .and_then(|v| v.get("data").and_then(|d| d.as_array().and_then(|a| a.first().cloned())));
    Ok(first)
}

// ── Territories ──────────────────────────────────────────────

pub async fn list_territories(config: &AscConfig) -> Result<Vec<Value>, AscApiError> {
    let data = asc_fetch(config, Method::GET, "/v1/territories?limit=200", None).await?;
    Ok(data.and_then(|v| v.get("data").and_then(|d| d.as_array().cloned())).unwrap_or_default())
}

// ── App Events ──────────────────────────────────────────────

pub async fn list_app_events(config: &AscConfig, app_id: &str) -> Result<Value, AscApiError> {
    let path = format!(
        "/v1/apps/{app_id}/appEvents?include=localizations&limit=50&fields[appEventLocalizations]=locale,name,shortDescription,longDescription"
    );
    Ok(asc_fetch(config, Method::GET, &path, None).await?.unwrap_or(Value::Null))
}

pub async fn get_app_event(config: &AscConfig, id: &str) -> Result<Value, AscApiError> {
    let path = format!(
        "/v1/appEvents/{id}?include=localizations&fields[appEventLocalizations]=locale,name,shortDescription,longDescription"
    );
    Ok(asc_fetch(config, Method::GET, &path, None).await?.unwrap_or(Value::Null))
}

pub async fn create_app_event(config: &AscConfig, app_id: &str, attributes: Value) -> Result<Value, AscApiError> {
    let body = json!({
        "data": {
            "type": "appEvents",
            "attributes": attributes,
            "relationships": { "app": { "data": { "type": "apps", "id": app_id } } }
        }
    });
    let data = asc_fetch(config, Method::POST, "/v1/appEvents", Some(body)).await?;
    Ok(data_field(data))
}

pub async fn update_app_event(config: &AscConfig, id: &str, attributes: Value) -> Result<Value, AscApiError> {
    let body = json!({ "data": { "type": "appEvents", "id": id, "attributes": attributes } });
    let data = asc_fetch(config, Method::PATCH, &format!("/v1/appEvents/{id}"), Some(body)).await?;
    Ok(data_field(data))
}

pub async fn delete_app_event(config: &AscConfig, id: &str) -> Result<(), AscApiError> {
    asc_fetch(config, Method::DELETE, &format!("/v1/appEvents/{id}"), None).await?;
    Ok(())
}

// ── App Event Localizations ──────────────────────────────────────────────

pub async fn create_app_event_localization(
    config: &AscConfig,
    app_event_id: &str,
    attributes: Value,
) -> Result<Value, AscApiError> {
    let body = json!({
        "data": {
            "type": "appEventLocalizations",
            "attributes": attributes,
            "relationships": { "appEvent": { "data": { "type": "appEvents", "id": app_event_id } } }
        }
    });
    let data = asc_fetch(config, Method::POST, "/v1/appEventLocalizations", Some(body)).await?;
    Ok(data_field(data))
}

pub async fn update_app_event_localization(config: &AscConfig, id: &str, attributes: Value) -> Result<Value, AscApiError> {
    let body = json!({ "data": { "type": "appEventLocalizations", "id": id, "attributes": attributes } });
    let data = asc_fetch(config, Method::PATCH, &format!("/v1/appEventLocalizations/{id}"), Some(body)).await?;
    Ok(data_field(data))
}

pub async fn delete_app_event_localization(config: &AscConfig, id: &str) -> Result<(), AscApiError> {
    asc_fetch(config, Method::DELETE, &format!("/v1/appEventLocalizations/{id}"), None).await?;
    Ok(())
}

pub async fn list_localization_screenshots(config: &AscConfig, localization_id: &str) -> Result<Vec<Value>, AscApiError> {
    let path = format!("/v1/appEventLocalizations/{localization_id}/appEventScreenshots");
    let data = asc_fetch(config, Method::GET, &path, None).await?;
    Ok(data.and_then(|v| v.get("data").and_then(|d| d.as_array().cloned())).unwrap_or_default())
}

// ── App Event Screenshots (asset upload) ──────────────────────────────────────────────

async fn reserve_asset(
    config: &AscConfig,
    localization_id: &str,
    file_name: &str,
    file_size: u64,
    asset_type: &str,
) -> Result<Value, AscApiError> {
    let body = json!({
        "data": {
            "type": "appEventScreenshots",
            "attributes": { "fileName": file_name, "fileSize": file_size, "appEventAssetType": asset_type },
            "relationships": {
                "appEventLocalization": { "data": { "type": "appEventLocalizations", "id": localization_id } }
            }
        }
    });
    let data = asc_fetch(config, Method::POST, "/v1/appEventScreenshots", Some(body)).await?;
    Ok(data_field(data))
}

async fn upload_asset_parts(bytes: &[u8], upload_operations: &[Value]) -> Result<(), AscApiError> {
    let client = reqwest::Client::new();
    for op in upload_operations {
        let offset = op.get("offset").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        let length = op.get("length").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        let method = op.get("method").and_then(|v| v.as_str()).unwrap_or("PUT");
        let url = op.get("url").and_then(|v| v.as_str()).unwrap_or("");
        let chunk = bytes[offset..(offset + length).min(bytes.len())].to_vec();

        let mut req = client.request(method.parse().unwrap_or(Method::PUT), url);
        if let Some(headers) = op.get("requestHeaders").and_then(|v| v.as_array()) {
            for h in headers {
                let name = h.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let value = h.get("value").and_then(|v| v.as_str()).unwrap_or("");
                if !name.is_empty() {
                    req = req.header(name, value);
                }
            }
        }

        let res = req.body(chunk).send().await.map_err(|e| AscApiError::new(e.to_string(), 500))?;
        if !res.status().is_success() {
            let status = res.status().as_u16();
            return Err(AscApiError::new(format!("Failed to upload chunk (offset {offset})"), status));
        }
    }
    Ok(())
}

async fn commit_screenshot(config: &AscConfig, id: &str, checksum: &str) -> Result<Value, AscApiError> {
    let body = json!({
        "data": {
            "type": "appEventScreenshots",
            "id": id,
            "attributes": { "uploaded": true, "sourceFileChecksum": checksum }
        }
    });
    let data = asc_fetch(config, Method::PATCH, &format!("/v1/appEventScreenshots/{id}"), Some(body)).await?;
    Ok(data_field(data))
}

async fn poll_screenshot(config: &AscConfig, id: &str) -> Result<Value, AscApiError> {
    let timeout = Duration::from_secs(60);
    let start = std::time::Instant::now();
    loop {
        let data = asc_fetch(config, Method::GET, &format!("/v1/appEventScreenshots/{id}"), None).await?;
        let entry = data_field(data);
        let state = entry
            .get("attributes")
            .and_then(|a| a.get("assetDeliveryState"))
            .and_then(|s| s.get("state"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if state == "COMPLETE" || state == "FAILED" {
            return Ok(entry);
        }
        if start.elapsed() >= timeout {
            return Err(AscApiError::new("Image processing timed out", 408));
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

pub async fn upload_app_event_screenshot(
    config: &AscConfig,
    localization_id: &str,
    asset_type: &str,
    file_name: &str,
    bytes: &[u8],
) -> Result<Value, AscApiError> {
    let reservation = reserve_asset(config, localization_id, file_name, bytes.len() as u64, asset_type).await?;
    let upload_operations = reservation
        .get("attributes")
        .and_then(|a| a.get("uploadOperations"))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    upload_asset_parts(bytes, &upload_operations).await?;

    let checksum = format!("{:x}", md5::compute(bytes));
    let id = reservation.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    commit_screenshot(config, &id, &checksum).await?;
    poll_screenshot(config, &id).await
}

pub async fn delete_app_event_screenshot(config: &AscConfig, id: &str) -> Result<(), AscApiError> {
    asc_fetch(config, Method::DELETE, &format!("/v1/appEventScreenshots/{id}"), None).await?;
    Ok(())
}

// ── Review Submission ──────────────────────────────────────────────

async fn find_open_review_submission(config: &AscConfig, app_id: &str, platform: &str) -> Result<Option<Value>, AscApiError> {
    let path = format!("/v1/apps/{app_id}/reviewSubmissions?filter[state]=READY_FOR_REVIEW&filter[platform]={platform}&limit=1");
    let data = asc_fetch(config, Method::GET, &path, None).await?;
    Ok(data.and_then(|v| v.get("data").and_then(|d| d.as_array().and_then(|a| a.first().cloned()))))
}

async fn create_review_submission(config: &AscConfig, app_id: &str, platform: &str) -> Result<Value, AscApiError> {
    let body = json!({
        "data": {
            "type": "reviewSubmissions",
            "attributes": { "platform": platform },
            "relationships": { "app": { "data": { "type": "apps", "id": app_id } } }
        }
    });
    let data = asc_fetch(config, Method::POST, "/v1/reviewSubmissions", Some(body)).await?;
    Ok(data_field(data))
}

pub async fn submit_app_event_for_review(
    config: &AscConfig,
    app_id: &str,
    app_event_id: &str,
    platform: &str,
) -> Result<Value, AscApiError> {
    let submission = match find_open_review_submission(config, app_id, platform).await? {
        Some(s) => s,
        None => create_review_submission(config, app_id, platform).await?,
    };
    let submission_id = submission.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();

    let body = json!({
        "data": {
            "type": "reviewSubmissionItems",
            "relationships": {
                "reviewSubmission": { "data": { "type": "reviewSubmissions", "id": submission_id } },
                "appEvent": { "data": { "type": "appEvents", "id": app_event_id } }
            }
        }
    });
    let data = asc_fetch(config, Method::POST, "/v1/reviewSubmissionItems", Some(body)).await?;
    Ok(data_field(data))
}

fn urlencoding_encode(s: &str) -> String {
    let mut out = String::new();
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(byte as char),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}
