mod appstoreconnect;
mod commands;
mod config;
mod deploy;
mod firebase;
mod workspace;
mod xcode_gradle;

use commands::appstoreconnect::{
    asc_config_delete, asc_config_get, asc_config_save, asc_event_create, asc_event_delete, asc_event_get,
    asc_event_submit, asc_event_update, asc_events_list, asc_localization_create, asc_localization_delete,
    asc_localization_screenshots, asc_localization_update, asc_screenshot_delete, asc_screenshot_upload,
    asc_territories_list,
};
use commands::config::{config_colors_get, config_colors_save, config_serconf_get, config_serconf_save};
use commands::deploy::{deploy_start, deploy_submit_two_factor_code};
use commands::firebase::{firebase_accounts, firebase_create_project, firebase_logout};
use commands::flutter::{flutter_build_start, flutter_devices};
use commands::project::{project_activate, project_create};
use commands::projects::{project_icon, projects_list, projects_rename, projects_version_set, projects_versions};
use commands::sentry::{sentry_check, sentry_create_project};
use commands::slack::slack_notify;
use commands::store_locales::store_locales_fetch;
use commands::sync_versions::{sync_versions_start, sync_versions_submit_two_factor_code};
use commands::workspace::{
    workspace_browse, workspace_config_get, workspace_config_save, workspace_recent_add, workspace_recent_get,
    workspace_validate,
};
use deploy::registry::DeployRegistry;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .manage(DeployRegistry::default())
        .invoke_handler(tauri::generate_handler![
            workspace_browse,
            workspace_validate,
            workspace_recent_get,
            workspace_recent_add,
            projects_list,
            projects_rename,
            projects_versions,
            project_icon,
            asc_config_get,
            asc_config_save,
            asc_config_delete,
            asc_events_list,
            asc_event_create,
            asc_event_get,
            asc_event_update,
            asc_event_delete,
            asc_event_submit,
            asc_localization_create,
            asc_localization_update,
            asc_localization_delete,
            asc_localization_screenshots,
            asc_screenshot_upload,
            asc_screenshot_delete,
            asc_territories_list,
            config_colors_get,
            config_colors_save,
            config_serconf_get,
            config_serconf_save,
            workspace_config_get,
            workspace_config_save,
            project_create,
            project_activate,
            projects_version_set,
            deploy_start,
            deploy_submit_two_factor_code,
            sync_versions_start,
            sync_versions_submit_two_factor_code,
            flutter_devices,
            flutter_build_start,
            store_locales_fetch,
            firebase_accounts,
            firebase_logout,
            firebase_create_project,
            sentry_check,
            sentry_create_project,
            slack_notify,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
