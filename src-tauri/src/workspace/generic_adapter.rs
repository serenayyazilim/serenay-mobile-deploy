use super::types::{WorkspaceAdapter, WorkspaceMode, WorkspaceProject};
use regex::Regex;
use std::path::Path;

pub const GENERIC_PROJECT_ID: &str = "default";

fn read_pubspec_name(workspace: &str) -> String {
    let pubspec_path = Path::new(workspace).join("pubspec.yaml");
    let Ok(content) = std::fs::read_to_string(&pubspec_path) else {
        return "Flutter Project".to_string();
    };
    let re = Regex::new(r"(?m)^name:\s*(.+)$").unwrap();
    re.captures(&content)
        .map(|c| c[1].trim().to_string())
        .unwrap_or_else(|| "Flutter Project".to_string())
}

fn read_android_application_id(workspace: &str) -> Option<String> {
    let re = Regex::new(r#"applicationId\s*[=]?\s*"([^"]+)""#).unwrap();
    for file in ["android/app/build.gradle.kts", "android/app/build.gradle"] {
        let gradle_path = Path::new(workspace).join(file);
        if let Ok(content) = std::fs::read_to_string(&gradle_path) {
            if let Some(c) = re.captures(&content) {
                return Some(c[1].to_string());
            }
        }
    }

    let manifest_path = Path::new(workspace).join("android/app/src/main/AndroidManifest.xml");
    if let Ok(content) = std::fs::read_to_string(&manifest_path) {
        let manifest_re = Regex::new(r#"package="([^"]+)""#).unwrap();
        if let Some(c) = manifest_re.captures(&content) {
            return Some(c[1].to_string());
        }
    }

    None
}

fn read_ios_bundle_id(workspace: &str) -> Option<String> {
    let pbxproj_path = Path::new(workspace).join("ios/Runner.xcodeproj/project.pbxproj");
    let content = std::fs::read_to_string(&pbxproj_path).ok()?;
    let re = Regex::new(r"PRODUCT_BUNDLE_IDENTIFIER = ([^;]+);").unwrap();
    re.captures(&content).map(|c| c[1].trim().to_string())
}

fn resolve_bundle_id(workspace: &str) -> String {
    read_android_application_id(workspace)
        .or_else(|| read_ios_bundle_id(workspace))
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
            bundle_id: resolve_bundle_id(workspace),
            app_name: read_pubspec_name(workspace),
        }]
    }

    fn rename_project(&self, _workspace: &str, _project_id: &str, _app_name: &str) -> bool {
        // Single project = the workspace itself; renaming would require manually
        // editing pubspec.yaml, which is not supported through this tool.
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
