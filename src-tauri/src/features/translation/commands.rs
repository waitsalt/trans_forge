use tauri::State;

use crate::shared::command::{CommandResult, to_client_error};
use crate::features::provider::Provider;
use crate::features::translation::{TranslationItem, TranslationProgress};
use crate::shared::common::Language;
use crate::shared::state::AppState;

#[tauri::command]
pub async fn read_input_files(
    state: State<'_, AppState>,
    input_path: String,
) -> CommandResult<Vec<TranslationItem>> {
    state.ensure_recovered_running_projects().await;
    state
        .project
        .read_input_files(input_path)
        .await
        .map_err(to_client_error)
}

#[tauri::command]
pub async fn load_project_items(
    state: State<'_, AppState>,
    name: String,
) -> CommandResult<Vec<TranslationItem>> {
    state.ensure_recovered_running_projects().await;
    state
        .project
        .load_project_items(&name)
        .await
        .map_err(to_client_error)
}

#[tauri::command]
pub async fn set_items(
    state: State<'_, AppState>,
    items: Vec<TranslationItem>,
) -> CommandResult<()> {
    state.ensure_recovered_running_projects().await;
    state.project.set_items(items).await.map_err(to_client_error)
}

#[tauri::command]
pub async fn start_translation(
    state: State<'_, AppState>,
    config: Provider,
    source_language: String,
    target_language: String,
) -> CommandResult<TranslationProgress> {
    state.ensure_recovered_running_projects().await;
    state
        .project
        .start_translation(config, source_language, target_language)
        .await
        .map_err(to_client_error)
}

#[tauri::command]
pub async fn stop_translation(state: State<'_, AppState>) -> CommandResult<()> {
    state.ensure_recovered_running_projects().await;
    state.project.stop_translation().await.map_err(to_client_error)
}

#[tauri::command]
pub async fn export_files(
    state: State<'_, AppState>,
    output_path: String,
) -> CommandResult<usize> {
    state.ensure_recovered_running_projects().await;
    state
        .project
        .export_files(output_path)
        .await
        .map_err(to_client_error)
}

#[tauri::command]
pub async fn get_items(state: State<'_, AppState>) -> CommandResult<Vec<TranslationItem>> {
    state.ensure_recovered_running_projects().await;
    Ok(state.project.get_items().await)
}

#[tauri::command]
pub async fn get_progress(state: State<'_, AppState>) -> CommandResult<TranslationProgress> {
    state.ensure_recovered_running_projects().await;
    Ok(state.project.get_progress().await)
}

#[tauri::command]
pub async fn get_supported_languages() -> Vec<(&'static str, &'static str)> {
    Language::supported_items()
}
