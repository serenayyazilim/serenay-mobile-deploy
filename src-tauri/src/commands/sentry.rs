use regex::Regex;
use serde::Serialize;
use serde_json::{json, Value};

fn read_sentry_config() -> (Option<String>, Option<String>) {
    let mut token = std::env::var("SENTRY_AUTH_TOKEN").ok();
    let mut org = std::env::var("SENTRY_ORG").ok();

    if token.is_none() || org.is_none() {
        if let Some(home) = dirs::home_dir() {
            let rc_path = home.join(".sentryclirc");
            if let Ok(content) = std::fs::read_to_string(&rc_path) {
                if token.is_none() {
                    if let Some(c) = Regex::new(r"token\s*=\s*(.+)").unwrap().captures(&content) {
                        token = Some(c[1].trim().to_string());
                    }
                }
                if org.is_none() {
                    if let Some(c) = Regex::new(r"org\s*=\s*(.+)").unwrap().captures(&content) {
                        org = Some(c[1].trim().to_string());
                    }
                }
            }
        }
    }

    (token, org)
}

#[derive(Debug, Serialize)]
pub struct SentryCheckResult {
    pub authenticated: bool,
    pub org: Option<String>,
    pub message: Option<String>,
}

#[tauri::command]
pub async fn sentry_check() -> SentryCheckResult {
    let (token, mut org) = read_sentry_config();
    let Some(token) = token else {
        return SentryCheckResult {
            authenticated: false,
            org: None,
            message: Some("Sentry auth token bulunamadı. ~/.sentryclirc veya SENTRY_AUTH_TOKEN ayarlayın.".to_string()),
        };
    };

    let client = reqwest::Client::new();

    if org.is_none() {
        if let Ok(res) = client.get("https://sentry.io/api/0/organizations/?member=1").bearer_auth(&token).send().await {
            if res.status().is_success() {
                if let Ok(orgs) = res.json::<Value>().await {
                    if let Some(first) = orgs.as_array().and_then(|a| a.first()) {
                        org = first.get("slug").and_then(|v| v.as_str()).map(String::from);
                    }
                }
            }
        }
    }

    if let Some(org_slug) = &org {
        let url = format!("https://sentry.io/api/0/organizations/{org_slug}/projects/");
        if let Ok(res) = client.get(&url).bearer_auth(&token).send().await {
            if res.status().as_u16() == 401 {
                return SentryCheckResult {
                    authenticated: false,
                    org: None,
                    message: Some("Sentry token geçersiz veya süresi dolmuş.".to_string()),
                };
            }
        }
    }

    SentryCheckResult { authenticated: true, org, message: None }
}

async fn sentry_api(method: reqwest::Method, endpoint: &str, token: &str, body: Option<Value>) -> Result<(bool, Value), String> {
    let client = reqwest::Client::new();
    let mut req = client.request(method, format!("https://sentry.io/api/0{endpoint}")).bearer_auth(token);
    if let Some(b) = &body {
        req = req.json(b);
    }
    let res = req.send().await.map_err(|e| e.to_string())?;
    let ok = res.status().is_success();
    let data: Value = res.json().await.unwrap_or(Value::Null);
    Ok((ok, data))
}

#[derive(Debug, Serialize)]
pub struct SentryCreateResult {
    pub success: bool,
    pub message: String,
    pub dsn: String,
}

#[tauri::command]
pub async fn sentry_create_project(project_id: String, org_slug: String) -> Result<SentryCreateResult, String> {
    let (token, _) = read_sentry_config();
    let Some(token) = token else {
        return Ok(SentryCreateResult {
            success: false,
            message: "Sentry auth token bulunamadı. ~/.sentryclirc veya SENTRY_AUTH_TOKEN ayarlayın.".to_string(),
            dsn: String::new(),
        });
    };

    let (teams_ok, teams_data) = sentry_api(reqwest::Method::GET, &format!("/organizations/{org_slug}/teams/"), &token, None).await?;
    let team_slug = if teams_ok {
        teams_data.as_array().and_then(|a| a.first()).and_then(|t| t.get("slug")).and_then(|v| v.as_str()).map(String::from)
    } else {
        None
    };

    let Some(team_slug) = team_slug else {
        return Ok(SentryCreateResult {
            success: false,
            message: "Organizasyonda team bulunamadı. Sentry dashboard'dan en az bir team oluşturun.".to_string(),
            dsn: String::new(),
        });
    };

    let (create_ok, create_data) = sentry_api(
        reqwest::Method::POST,
        &format!("/teams/{org_slug}/{team_slug}/projects/"),
        &token,
        Some(json!({ "name": project_id, "slug": project_id, "platform": "flutter" })),
    )
    .await?;

    if !create_ok {
        let detail = create_data
            .get("detail")
            .or_else(|| create_data.get("slug"))
            .map(|v| v.to_string())
            .unwrap_or_else(|| create_data.to_string());
        return Ok(SentryCreateResult { success: false, message: format!("Sentry projesi oluşturulamadı: {detail}"), dsn: String::new() });
    }

    let (keys_ok, keys_data) = sentry_api(reqwest::Method::GET, &format!("/projects/{org_slug}/{project_id}/keys/"), &token, None).await?;
    let dsn = if keys_ok {
        keys_data
            .as_array()
            .and_then(|a| a.first())
            .and_then(|k| k.get("dsn"))
            .and_then(|d| d.get("public").or_else(|| d.get("secret")))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string()
    } else {
        String::new()
    };

    if dsn.is_empty() {
        return Ok(SentryCreateResult {
            success: true,
            message: "Sentry projesi oluşturuldu ancak DSN alınamadı. Sentry dashboard'dan kopyalayın.".to_string(),
            dsn: String::new(),
        });
    }

    Ok(SentryCreateResult { success: true, message: "Sentry projesi oluşturuldu ve DSN alındı".to_string(), dsn })
}
