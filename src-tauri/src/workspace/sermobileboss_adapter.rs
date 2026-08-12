use super::sermobileboss_config::read_sermobileboss_config;
use super::types::{WorkspaceAdapter, WorkspaceMode, WorkspaceProject};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn projects_json_path(workspace: &str) -> PathBuf {
    Path::new(workspace).join("sermobileboss_projects.json")
}

fn active_project_file_path(workspace: &str) -> PathBuf {
    Path::new(workspace).join("sermobileboss.txt")
}

pub struct SermobilebossAdapter;

impl WorkspaceAdapter for SermobilebossAdapter {
    fn mode(&self) -> WorkspaceMode {
        WorkspaceMode::Sermobileboss
    }

    fn list_projects(&self, workspace: &str) -> Vec<WorkspaceProject> {
        let json_path = projects_json_path(workspace);
        let Ok(content) = std::fs::read_to_string(&json_path) else {
            return vec![];
        };
        let Ok(projects_data): Result<BTreeMap<String, String>, _> = serde_json::from_str(&content)
        else {
            return vec![];
        };

        let bundle_id_prefix = read_sermobileboss_config(workspace).map(|c| c.bundle_id_prefix);

        let mut projects: Vec<WorkspaceProject> = projects_data
            .into_iter()
            .map(|(key, app_name)| {
                let bundle_id = if key.contains('.') {
                    key.clone()
                } else if let Some(prefix) = &bundle_id_prefix {
                    format!("{prefix}.{key}")
                } else {
                    "? (Workspace Ayarları eksik)".to_string()
                };
                WorkspaceProject {
                    id: key,
                    bundle_id,
                    app_name,
                }
            })
            .collect();

        // Not: orijinal TS `localeCompare(..., "tr")` kullanıyor; burada Türkçe
        // koleksiyon kurallarına (ı/i, ç, ş, ö, ü, ğ sıralaması) tam uyum için
        // basit case-insensitive karşılaştırma yeterli değil, ama Faz 0 kapsamında
        // (salt-okunur olmayan bir sıralama detayı) kabul edilebilir bir yaklaşım.
        projects.sort_by(|a, b| a.app_name.to_lowercase().cmp(&b.app_name.to_lowercase()));
        projects
    }

    fn rename_project(&self, workspace: &str, project_id: &str, app_name: &str) -> bool {
        let json_path = projects_json_path(workspace);
        let Ok(content) = std::fs::read_to_string(&json_path) else {
            return false;
        };
        let Ok(mut projects_data): Result<serde_json::Map<String, Value>, _> =
            serde_json::from_str(&content)
        else {
            return false;
        };

        if !projects_data.contains_key(project_id) {
            return false;
        }

        projects_data.insert(project_id.to_string(), Value::String(app_name.to_string()));
        let Ok(json) = serde_json::to_string_pretty(&projects_data) else {
            return false;
        };
        std::fs::write(json_path, json).is_ok()
    }

    fn get_active_project_id(&self, workspace: &str) -> Option<String> {
        let file = active_project_file_path(workspace);
        std::fs::read_to_string(file).ok().map(|s| s.trim().to_string())
    }

    fn get_project_dir(&self, workspace: &str, project_id: &str) -> String {
        Path::new(workspace)
            .join("lib/conf/sermobplus-projects")
            .join(project_id)
            .to_string_lossy()
            .to_string()
    }

    fn supports_multiple_projects(&self) -> bool {
        true
    }

    fn supports_tenant_config(&self) -> bool {
        true
    }
}
