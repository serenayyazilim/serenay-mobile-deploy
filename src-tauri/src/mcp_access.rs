//! Who may call which MCP tool. Every request needs a bearer token from `mcp-tokens.json`
//! (in `~/Library/Application Support/com.serenaymobiledeploy.app/`), and the permissions listed
//! on the token decide the tools it may call. The file is read on every request, so edits apply at once.
use serde_json::{json, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Permission {
    /// Status, logs, lists, store data.
    Read,
    /// Run the app and the tests on a simulator/device.
    Run,
    /// Deploy, sync the store versions, build the bundle.
    Deploy,
    /// Change the store listings (App Store Connect events, texts, screenshots).
    Store,
    /// Change the project: version, config, colors, splash, assets, store setup.
    Project,
    /// Change the workspace and the app: git branch, workspace, updates, new Firebase/Sentry projects.
    System,
    /// Get the link of the token manager page (create, change and delete tokens in a browser).
    Admin,
}

const ALL: [(Permission, &str); 7] = [
    (Permission::Read, "read"),
    (Permission::Run, "run"),
    (Permission::Deploy, "deploy"),
    (Permission::Store, "store"),
    (Permission::Project, "project"),
    (Permission::System, "system"),
    (Permission::Admin, "admin"),
];

impl Permission {
    pub fn name(self) -> &'static str {
        ALL.iter().find(|(p, _)| *p == self).map(|(_, n)| *n).unwrap_or("system")
    }
}

/// What one token may do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Access(u8);

impl Access {
    #[cfg(test)]
    pub fn all() -> Access {
        Access(0b111_1111)
    }

    pub fn allows(self, permission: Permission) -> bool {
        self.0 & (1 << permission as u8) != 0
    }

    /// From the names in a token's `permissions`; a name that doesn't exist grants nothing.
    fn from_names(names: &[Value]) -> Access {
        Access(ALL.iter().filter(|(_, n)| names.iter().any(|v| v.as_str() == Some(n))).fold(0, |bits, (p, _)| bits | 1 << *p as u8))
    }
}

/// The permission a tool needs; `None` for a tool nobody classified (then it needs `system`).
fn classified(tool: &str) -> Option<Permission> {
    Some(match tool {
        "status" | "logs" | "list_workspaces" | "git_status" | "list_projects" | "deploy_history" | "app_version" | "list_devices"
        | "test_screenshots" | "maestro_flows" | "get_project_config" | "get_colors" | "get_splash_color" | "check_splash_image"
        | "setup_readiness" | "store_check" | "store_locales" | "asc_status" | "asc_apps" | "asc_territories" | "asc_events_list"
        | "asc_event_get" | "asc_screenshots" | "firebase_accounts" | "sentry_check" => Permission::Read,
        "run_app" | "sim_link" | "hot_reload" | "hot_restart" | "stop_app" | "run_tests" | "maestro_flow_write" | "remote_access"
        | "slack_test" => Permission::Run,
        "deploy" | "stop_deploy" | "sync_store_versions" | "build_aab" | "complete_fastlane" => Permission::Deploy,
        "asc_event_create" | "asc_event_update" | "asc_event_delete" | "asc_event_submit" | "asc_localization_create"
        | "asc_localization_update" | "asc_localization_delete" | "asc_screenshot_upload" | "asc_screenshot_delete" => Permission::Store,
        "set_version" | "rename_project" | "update_project_config" | "update_colors" | "set_splash_color" | "set_project_asset"
        | "create_project" | "setup_apply" | "set_team_id" | "set_play_key" => Permission::Project,
        "token_manager_link" => Permission::Admin,
        "open_workspace" | "git_switch" | "app_check_update" | "app_update" | "firebase_create_project" | "sentry_create_project" => Permission::System,
        _ => return None,
    })
}

pub fn required(tool: &str) -> Permission {
    classified(tool).unwrap_or(Permission::System)
}

static ENABLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

/// Whether the MCP server answers at all (the on/off switch in Settings). Kept in the file `mcp-off`
/// next to the tokens: present = off.
pub fn enabled() -> bool {
    ENABLED.load(std::sync::atomic::Ordering::Relaxed)
}

pub fn load_enabled() {
    let off = dir().is_some_and(|d| d.join("mcp-off").exists());
    ENABLED.store(!off, std::sync::atomic::Ordering::Relaxed);
}

pub fn set_enabled(on: bool) {
    ENABLED.store(on, std::sync::atomic::Ordering::Relaxed);
    if let Some(dir) = dir() {
        let _ = std::fs::create_dir_all(&dir);
        if on {
            let _ = std::fs::remove_file(dir.join("mcp-off"));
        } else {
            let _ = std::fs::write(dir.join("mcp-off"), "");
        }
    }
}

/// The main password of the browser token manager: `MCP_MAIN_PASSWORD` in `.env` next to the tokens
/// (read on every use, so editing the file changes it at once). Made with a random value the first
/// time; Settings, MCP shows it.
pub fn main_password() -> String {
    let Some(dir) = dir() else { return String::new() };
    let file = dir.join(".env");
    let text = std::fs::read_to_string(&file).unwrap_or_default();
    let found = text.lines().find_map(|l| l.trim().strip_prefix("MCP_MAIN_PASSWORD=")).map(|v| v.trim().trim_matches(['"', '\'']).to_string()).filter(|v| !v.is_empty());
    if let Some(found) = found {
        return found;
    }
    let password = format!("{}", uuid::Uuid::new_v4().simple());
    let mut text: String = text.lines().filter(|l| !l.trim().starts_with("MCP_MAIN_PASSWORD=")).map(|l| format!("{l}\n")).collect();
    text.push_str(&format!("MCP_MAIN_PASSWORD={password}\n"));
    let _ = std::fs::create_dir_all(&dir);
    if std::fs::write(&file, text).is_ok() {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600));
    }
    password
}

/// Compares without stopping at the first different byte.
pub fn password_matches(given: &str) -> bool {
    let want = main_password();
    given.len() == want.len() && given.bytes().zip(want.bytes()).fold(0u8, |d, (a, b)| d | (a ^ b)) == 0
}

fn dir() -> Option<std::path::PathBuf> {
    Some(dirs::config_dir()?.join("com.serenaymobiledeploy.app"))
}

/// The raw entries of `mcp-tokens.json`. The first time, the old single `mcp-token` becomes a
/// token named "default" with every permission (or a new one is made).
fn read_entries() -> Vec<Value> {
    let Some(dir) = dir() else { return vec![] };
    let file = dir.join("mcp-tokens.json");
    let text = match std::fs::read_to_string(&file) {
        Ok(text) => text,
        Err(_) => {
            let token = std::fs::read_to_string(dir.join("mcp-token"))
                .ok()
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .unwrap_or_else(new_token);
            let names: Vec<&str> = ALL.iter().map(|(_, n)| *n).collect();
            let entries = vec![json!({ "name": "default", "token": token, "permissions": names })];
            write_entries(&entries);
            return entries;
        }
    };
    serde_json::from_str(&text).unwrap_or_default()
}

fn write_entries(entries: &[Value]) {
    let Some(dir) = dir() else { return };
    let _ = std::fs::create_dir_all(&dir);
    let file = dir.join("mcp-tokens.json");
    if std::fs::write(&file, serde_json::to_string_pretty(entries).unwrap_or_default()).is_ok() {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600));
    }
}

fn new_token() -> String {
    format!("{}{}", uuid::Uuid::new_v4().simple(), uuid::Uuid::new_v4().simple())
}

fn load() -> Vec<(String, Access)> {
    parse_entries(&read_entries())
}

fn parse(text: &str) -> Vec<(String, Access)> {
    parse_entries(&serde_json::from_str::<Vec<Value>>(text).unwrap_or_default())
}

fn parse_entries(entries: &[Value]) -> Vec<(String, Access)> {
    entries
        .iter()
        .filter_map(|e| Some((e.get("token")?.as_str()?.to_lowercase(), Access::from_names(e.get("permissions")?.as_array()?))))
        .filter(|(token, _)| !token.is_empty())
        .collect()
}

/// Every token with its name and permissions, for the token manager (app screen and browser page).
pub fn list_tokens() -> Value {
    let entries = read_entries();
    json!({ "permissions": ALL.iter().map(|(_, n)| *n).collect::<Vec<_>>(), "tokens": entries })
}

fn has_admin(entries: &[Value]) -> bool {
    parse_entries(entries).iter().any(|(_, a)| a.allows(Permission::Admin))
}

/// One change from the token manager: `create` (name, permissions), `update` (name, permissions),
/// `regenerate` (name: a new secret, the old one stops working) or `delete` (name). Returns the new list.
/// A change may not take away the last token with `admin` (that would lock the browser page out
/// for good) nor the last token of all.
pub fn apply(action: &str, name: &str, permissions: Option<&[Value]>) -> Result<Value, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("A token needs a name.".to_string());
    }
    let mut entries = read_entries();
    let had_admin = has_admin(&entries);
    let pos = entries.iter().position(|e| e.get("name").and_then(Value::as_str) == Some(name));
    let names: Vec<Value> = ALL
        .iter()
        .filter(|(_, n)| permissions.is_some_and(|p| p.iter().any(|v| v.as_str() == Some(n))))
        .map(|(_, n)| json!(n))
        .collect();
    match (action, pos) {
        ("create", Some(_)) => return Err(format!("A token named {name} exists already.")),
        ("create", None) => entries.push(json!({ "name": name, "token": new_token(), "permissions": names })),
        ("update", Some(i)) => entries[i]["permissions"] = json!(names),
        ("regenerate", Some(i)) => entries[i]["token"] = json!(new_token()),
        ("delete", Some(i)) => {
            entries.remove(i);
        }
        ("create" | "update" | "regenerate" | "delete", None) => return Err(format!("No token named {name}.")),
        _ => return Err(format!("Unknown action: {action}")),
    }
    if entries.is_empty() {
        return Err("The last token cannot be deleted; nobody could use the server then.".to_string());
    }
    if had_admin && !has_admin(&entries) {
        return Err("The last token with the admin permission cannot lose it or be deleted.".to_string());
    }
    write_entries(&entries);
    Ok(list_tokens())
}

/// What a bearer token may do, or `None` when the token is unknown (or no token came).
pub fn access_for(token: &str) -> Option<Access> {
    let token = token.to_lowercase();
    load().into_iter().find(|(t, _)| *t == token).map(|(_, access)| access)
}

/// Whether any token is set up (otherwise nobody can use the server).
pub fn has_tokens() -> bool {
    !load().is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_tool_is_classified() {
        // A tool left out of `classified` silently needs `system`; make that a conscious choice.
        for (name, ..) in crate::mcp::TOOLS {
            assert!(classified(name).is_some(), "{name} has no permission in mcp_access::classified");
        }
        assert_eq!(required("deploy"), Permission::Deploy);
        assert_eq!(required("some_new_tool"), Permission::System);
    }

    #[test]
    fn admin_is_a_permission_of_its_own() {
        let tokens = parse(r#"[{"name":"a","token":"x","permissions":["system"]},{"name":"b","token":"y","permissions":["admin"]}]"#);
        assert!(!tokens[0].1.allows(Permission::Admin) && tokens[1].1.allows(Permission::Admin));
        assert_eq!(required("token_manager_link"), Permission::Admin);
    }

    #[test]
    fn a_token_has_only_the_permissions_it_lists() {
        let tokens = parse(r#"[{"name":"a","token":"AbC","permissions":["read","run","deplooy"]},{"name":"b","token":"","permissions":["read"]},{"name":"c","token":"x"}]"#);
        assert_eq!(tokens.len(), 1);
        let (token, access) = &tokens[0];
        assert_eq!(token, "abc");
        let access = *access;
        assert!(access.allows(Permission::Read) && access.allows(Permission::Run));
        assert!(!access.allows(Permission::Deploy) && !access.allows(Permission::System)); // a typo grants nothing
    }
}
