use crate::deploy::locales::get_store_locales;
use serde_json::{json, Value};

/// Equivalent of `POST /api/store-locales` — a UI-triggered store locale lookup,
/// independent of the deploy flow.
#[tauri::command]
pub async fn store_locales_fetch(workspace_path: String) -> Value {
    let (ios_result, android_result) = get_store_locales(&workspace_path).await;

    let to_json = |r: Result<Vec<String>, String>| match r {
        Ok(locales) => json!({ "locales": locales, "error": "" }),
        Err(error) => json!({ "locales": [], "error": error }),
    };

    json!({ "ios": to_json(ios_result), "android": to_json(android_result) })
}
