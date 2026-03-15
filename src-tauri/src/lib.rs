//! 主库入口

mod features;
mod shared;

use features::kv::commands::*;
use features::preset::commands::*;
use features::project::commands::*;
use features::provider::commands::*;
use features::theme::commands::*;
use features::translation::commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    crate::shared::util::log_init();
    tracing::info!("Tran001 Actor Runtime 已启动");

    tauri::Builder::default()
        .manage(crate::shared::state::AppState::new())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            query_providers,
            list_providers,
            get_provider,
            create_provider,
            update_provider,
            delete_provider,
            delete_providers,
            query_projects,
            list_projects,
            get_project,
            get_project_progress,
            get_project_runtime_snapshot,
            get_all_project_runtime_snapshots,
            create_project_profile,
            update_project_profile,
            delete_project,
            delete_projects,
            query_prompt_presets,
            list_prompt_presets,
            get_prompt_preset,
            create_prompt_preset,
            update_prompt_preset,
            delete_prompt_preset,
            delete_prompt_presets,
            clear_project_items,
            fetch_models,
            create_project,
            load_project,
            read_input_files,
            load_project_items,
            get_items,
            set_items,
            start_translation,
            stop_translation,
            get_progress,
            export_files,
            get_supported_languages,
            get_app_kv,
            set_app_kv,
            list_app_kv,
            delete_app_kv,
            test_provider,
            get_theme_state,
            create_theme,
            update_theme,
            delete_theme,
            restore_default_themes,
            save_theme_preferences,
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|error| {
            tracing::error!("运行 tauri 应用时发生错误: {}", error);
        });
}
