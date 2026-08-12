use super::generic_adapter::GenericAdapter;
use super::sermobileboss_adapter::SermobilebossAdapter;
use super::types::{WorkspaceAdapter, WorkspaceMode};
use std::path::Path;

pub fn detect_workspace_mode(workspace: &str) -> WorkspaceMode {
    let projects_json_path = Path::new(workspace).join("sermobileboss_projects.json");
    if projects_json_path.exists() {
        WorkspaceMode::Sermobileboss
    } else {
        WorkspaceMode::Generic
    }
}

pub fn get_adapter(workspace: &str) -> Box<dyn WorkspaceAdapter> {
    match detect_workspace_mode(workspace) {
        WorkspaceMode::Sermobileboss => Box::new(SermobilebossAdapter),
        WorkspaceMode::Generic => Box::new(GenericAdapter),
    }
}
