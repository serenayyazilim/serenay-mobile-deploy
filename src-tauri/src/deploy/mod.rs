pub mod locales;
pub mod registry;
pub mod translate;

/// Path where `scripts/deploy.rb` and `scripts/sync_versions.rb` live in the Tauri bundle.
/// Uses the project root's `scripts/` in dev mode, and the resource directory in the packaged app.
pub fn find_script_path(app: &tauri::AppHandle, script_name: &str) -> Option<std::path::PathBuf> {
    use tauri::Manager;

    if let Ok(resource_dir) = app.path().resource_dir() {
        let p = resource_dir.join("scripts").join(script_name);
        if p.exists() {
            return Some(p);
        }
    }

    // Dev fallback: src-tauri/scripts/
    let dev_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts").join(script_name);
    if dev_path.exists() {
        return Some(dev_path);
    }

    None
}
