use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkspaceMode {
    Sermobileboss,
    Generic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceProject {
    pub id: String,
    #[serde(rename = "bundleId")]
    pub bundle_id: String,
    #[serde(rename = "appName")]
    pub app_name: String,
}

/// `lib/workspace/types.ts`'teki `WorkspaceAdapter` interface'inin Rust karşılığı.
pub trait WorkspaceAdapter {
    fn mode(&self) -> WorkspaceMode;

    fn list_projects(&self, workspace: &str) -> Vec<WorkspaceProject>;
    fn rename_project(&self, workspace: &str, project_id: &str, app_name: &str) -> bool;

    fn get_active_project_id(&self, workspace: &str) -> Option<String>;

    /// Bir projenin asset/config dosyalarının kök klasörü.
    fn get_project_dir(&self, workspace: &str, project_id: &str) -> String;

    fn supports_multiple_projects(&self) -> bool;
    fn supports_tenant_config(&self) -> bool;
}
