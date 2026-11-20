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
}

const ALL: [(Permission, &str); 6] = [
    (Permission::Read, "read"),
    (Permission::Run, "run"),
    (Permission::Deploy, "deploy"),
    (Permission::Store, "store"),
    (Permission::Project, "project"),
    (Permission::System, "system"),
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
        Access(0b11_1111)
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
        "open_workspace" | "git_switch" | "app_check_update" | "app_update" | "firebase_create_project" | "sentry_create_project" => Permission::System,
        _ => return None,
    })
}

pub fn required(tool: &str) -> Permission {
    classified(tool).unwrap_or(Permission::System)
}

fn dir() -> Option<std::path::PathBuf> {
    Some(dirs::config_dir()?.join("com.serenaymobiledeploy.app"))
}

/// The (token, access) pairs of `mcp-tokens.json`. The first time, the old single `mcp-token`
/// becomes a token named "default" with every permission (or a new one is made).
fn load() -> Vec<(String, Access)> {
    let Some(dir) = dir() else { return vec![] };
    let file = dir.join("mcp-tokens.json");
    let text = match std::fs::read_to_string(&file) {
        Ok(text) => text,
        Err(_) => {
            let token = std::fs::read_to_string(dir.join("mcp-token"))
                .ok()
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .unwrap_or_else(|| format!("{}{}", uuid::Uuid::new_v4().simple(), uuid::Uuid::new_v4().simple()));
            let names: Vec<&str> = ALL.iter().map(|(_, n)| *n).collect();
            let text = serde_json::to_string_pretty(&json!([{ "name": "default", "token": token, "permissions": names }])).unwrap_or_default();
            let _ = std::fs::create_dir_all(&dir);
            if std::fs::write(&file, &text).is_ok() {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600));
            }
            text
        }
    };
    parse(&text)
}

fn parse(text: &str) -> Vec<(String, Access)> {
    let entries: Vec<Value> = serde_json::from_str(text).unwrap_or_default();
    entries
        .iter()
        .filter_map(|e| Some((e.get("token")?.as_str()?.to_lowercase(), Access::from_names(e.get("permissions")?.as_array()?))))
        .filter(|(token, _)| !token.is_empty())
        .collect()
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
