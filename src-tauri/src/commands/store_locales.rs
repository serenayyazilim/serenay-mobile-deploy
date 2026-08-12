use crate::deploy::locales::run_fastlane_fetch_locales;
use serde_json::{json, Value};
use std::path::Path;

/// `POST /api/store-locales` karşılığı — deploy akışından bağımsız, UI-tetiklemeli
/// mağaza dili sorgusu.
#[tauri::command]
pub async fn store_locales_fetch(workspace_path: String) -> Value {
    let root = Path::new(&workspace_path);
    let ios_dir = root.join("ios");
    let android_dir = root.join("android");
    let (ios_result, android_result) =
        tokio::join!(run_fastlane_fetch_locales(&ios_dir), run_fastlane_fetch_locales(&android_dir));

    let to_json = |r: Result<Vec<String>, String>| match r {
        Ok(locales) => json!({ "locales": locales, "error": "" }),
        Err(error) => json!({ "locales": [], "error": error }),
    };

    json!({ "ios": to_json(ios_result), "android": to_json(android_result) })
}
