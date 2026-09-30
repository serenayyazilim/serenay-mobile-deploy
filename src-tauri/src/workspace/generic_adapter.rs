use crate::setup::{detect_kind, platform_dirs, project_name, read_android_package, read_ios_bundle_id};
use super::types::{WorkspaceAdapter, WorkspaceMode, WorkspaceProject};
use std::path::Path;

pub const GENERIC_PROJECT_ID: &str = "default";

fn resolve_bundle_id(root: &Path) -> String {
    let (ios_dir, android_dir) = detect_kind(root).map(|kind| platform_dirs(root, kind)).unwrap_or((None, None));
    android_dir
        .as_deref()
        .and_then(read_android_package)
        .or_else(|| ios_dir.as_deref().and_then(read_ios_bundle_id))
        .unwrap_or_else(|| "unknown".to_string())
}

pub struct GenericAdapter;

impl WorkspaceAdapter for GenericAdapter {
    fn mode(&self) -> WorkspaceMode {
        WorkspaceMode::Generic
    }

    fn list_projects(&self, workspace: &str) -> Vec<WorkspaceProject> {
        vec![WorkspaceProject {
            id: GENERIC_PROJECT_ID.to_string(),
            bundle_id: resolve_bundle_id(Path::new(workspace)),
            app_name: detect_kind(Path::new(workspace))
                .map(|kind| project_name(Path::new(workspace), kind))
                .unwrap_or_else(|| "Mobile Project".to_string()),
            kind: detect_kind(Path::new(workspace)),
        }]
    }

    fn rename_project(&self, _workspace: &str, _project_id: &str, _app_name: &str) -> bool {
        // Single project = the workspace itself; renaming would mean editing the
        // project's own files (pubspec.yaml, package.json, ...), which this tool doesn't do.
        false
    }

    fn get_active_project_id(&self, _workspace: &str) -> Option<String> {
        Some(GENERIC_PROJECT_ID.to_string())
    }

    fn get_project_dir(&self, workspace: &str, _project_id: &str) -> String {
        workspace.to_string()
    }

    fn supports_multiple_projects(&self) -> bool {
        false
    }

    fn supports_tenant_config(&self) -> bool {
        false
    }
}
