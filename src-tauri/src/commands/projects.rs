use crate::workspace::detect::{detect_workspace_mode, get_adapter};
use crate::workspace::generic_adapter::GENERIC_PROJECT_ID;
use crate::workspace::types::{WorkspaceMode, WorkspaceProject};
use crate::xcode_gradle;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use regex::Regex;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[tauri::command]
pub fn projects_list(workspace: String) -> Result<Vec<WorkspaceProject>, String> {
    Ok(get_adapter(&workspace).list_projects(&workspace))
}

#[tauri::command]
pub fn projects_rename(workspace: String, project_id: String, app_name: String) -> Result<(), String> {
    let adapter = get_adapter(&workspace);
    let renamed = adapter.rename_project(&workspace, &project_id, &app_name);
    if !renamed {
        return Err(if adapter.supports_multiple_projects() {
            "Project not found".to_string()
        } else {
            "Renaming projects is not supported in this workspace".to_string()
        });
    }
    Ok(())
}

fn read_pubspec_version(workspace: &str) -> Option<String> {
    let pubspec_path = Path::new(workspace).join("pubspec.yaml");
    let content = std::fs::read_to_string(pubspec_path).ok()?;
    let re = Regex::new(r"(?m)^version:\s*(.+)$").unwrap();
    re.captures(&content).map(|c| c[1].trim().to_string())
}

#[derive(Debug, Serialize)]
pub struct VersionsResult {
    pub versions: HashMap<String, String>,
}

#[tauri::command]
pub fn projects_versions(workspace: String) -> VersionsResult {
    if detect_workspace_mode(&workspace) == WorkspaceMode::Generic {
        let version = read_pubspec_version(&workspace).unwrap_or_else(|| "1.0.0+1".to_string());
        let mut versions = HashMap::new();
        versions.insert(GENERIC_PROJECT_ID.to_string(), version);
        return VersionsResult { versions };
    }

    let projects_path = Path::new(&workspace).join("lib/conf/sermobplus-projects");
    let mut versions = HashMap::new();

    let Ok(entries) = std::fs::read_dir(&projects_path) else {
        return VersionsResult { versions };
    };

    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else { continue };
        if !file_type.is_dir() {
            continue;
        }
        let project_id = entry.file_name().to_string_lossy().to_string();
        let version_file = entry.path().join("version.json");
        if let Ok(content) = std::fs::read_to_string(&version_file) {
            let version = serde_json::from_str::<serde_json::Value>(&content)
                .ok()
                .and_then(|v| v.get("version").and_then(|s| s.as_str()).map(String::from))
                .unwrap_or_else(|| "1.0.0+1".to_string());
            versions.insert(project_id, version);
        }
    }

    VersionsResult { versions }
}

/// Equivalent of `PUT /api/projects/versions` — writes the version, and also
/// syncs pubspec.yaml + Generated.xcconfig + pbxproj + Info.plist if it's the active project.
#[tauri::command]
pub fn projects_version_set(workspace: String, project_id: String, version: String) -> Result<(), String> {
    let root = Path::new(&workspace);

    if detect_workspace_mode(&workspace) == WorkspaceMode::Generic {
        let pubspec_path = root.join("pubspec.yaml");
        let mut content = std::fs::read_to_string(&pubspec_path).map_err(|_| "pubspec.yaml not found".to_string())?;
        content = Regex::new(r"(?m)^version:\s*.+$").unwrap().replace(&content, format!("version: {version}")).to_string();
        std::fs::write(&pubspec_path, content).map_err(|e| e.to_string())?;

        let (semver, build_num) = xcode_gradle::split_version(&version);
        xcode_gradle::update_generated_xcconfig(&root.join("ios"), &semver, &build_num);
        return Ok(());
    }

    let project_folder = root.join("lib/conf/sermobplus-projects").join(&project_id);
    if !project_folder.exists() {
        return Err("Project folder not found".to_string());
    }

    let version_file = project_folder.join("version.json");
    let mut existing: serde_json::Map<String, Value> = std::fs::read_to_string(&version_file)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default();
    existing.insert("version".to_string(), Value::String(version.clone()));
    std::fs::write(&version_file, serde_json::to_string_pretty(&existing).unwrap()).map_err(|e| e.to_string())?;

    let active_project = std::fs::read_to_string(root.join("sermobileboss.txt")).ok().map(|s| s.trim().to_string());
    if active_project.as_deref() == Some(project_id.as_str()) {
        let pubspec_path = root.join("pubspec.yaml");
        if let Ok(mut content) = std::fs::read_to_string(&pubspec_path) {
            content = Regex::new(r"(?m)^version:\s*.+$").unwrap().replace(&content, format!("version: {version}")).to_string();
            let _ = std::fs::write(&pubspec_path, content);
        }

        let (semver, build_num) = xcode_gradle::split_version(&version);
        let ios_folder = root.join("ios");
        xcode_gradle::update_generated_xcconfig(&ios_folder, &semver, &build_num);

        xcode_gradle::patch_pbxproj_version_only(&ios_folder, &version);
        xcode_gradle::patch_info_plist_version_only(&ios_folder, &version);
    }

    Ok(())
}

fn sanitize_project_id(project_id: &str) -> String {
    project_id
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-' || *c == '.')
        .collect()
}

fn read_icon_as_data_url(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    Some(format!("data:image/png;base64,{}", STANDARD.encode(bytes)))
}

/// App icon `<img>` source for components/project-grid/**; equivalent of the old
/// `/api/project-icon` route. `icon_type`: logo | splash | icon512 | icon1024 | (none = default).
#[tauri::command]
pub fn project_icon(workspace: String, project_id: String, icon_type: Option<String>) -> Option<String> {
    let workspace_root = Path::new(&workspace);

    if detect_workspace_mode(&workspace) == WorkspaceMode::Generic {
        let image_path = if icon_type.as_deref() == Some("splash") {
            workspace_root.join("assets/splash/splash.png")
        } else {
            workspace_root.join("assets/icon/icon.png")
        };
        return read_icon_as_data_url(&image_path);
    }

    let safe_id = sanitize_project_id(&project_id);
    let project_dir = workspace_root.join("lib/conf/sermobplus-projects").join(&safe_id);

    let image_path: Option<PathBuf> = match icon_type.as_deref() {
        Some("logo") => {
            let p = workspace_root.join("assets/images").join(format!("logo-{safe_id}.png"));
            p.exists().then_some(p)
        }
        Some("splash") => {
            let launch_dir = project_dir.join("Launch");
            ["splash.png", "3x.png", "2x.png", "1x.png"]
                .into_iter()
                .map(|name| launch_dir.join(name))
                .find(|p| p.exists())
        }
        Some("icon512") => {
            let p = project_dir.join("Graphic/512x512 icon.png");
            p.exists().then_some(p)
        }
        Some("icon1024") => {
            let p = project_dir.join("Graphic/1024x1024 icon.png");
            p.exists().then_some(p)
        }
        _ => {
            let graphic_dir = project_dir.join("Graphic");
            ["512x512 icon.png", "1024x1024 icon.png", "icon.png"]
                .into_iter()
                .map(|name| graphic_dir.join(name))
                .find(|p| p.exists())
                .or_else(|| {
                    std::fs::read_dir(&graphic_dir).ok().and_then(|entries| {
                        entries
                            .flatten()
                            .map(|e| e.path())
                            .find(|p| {
                                let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();
                                name.ends_with("icon.png") || name.contains("512") || name.ends_with(".png")
                            })
                    })
                })
        }
    };

    read_icon_as_data_url(&image_path?)
}
