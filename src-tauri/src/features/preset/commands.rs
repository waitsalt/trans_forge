use crate::features::preset::{PromptPreset, PromptPresetPage};
use tauri::State;

use crate::shared::command::{CommandResult, to_client_error};
use crate::shared::state::AppState;

#[tauri::command]
pub async fn query_prompt_presets(
    state: State<'_, AppState>,
    keyword: Option<String>,
    page: u32,
    page_size: u32,
) -> CommandResult<PromptPresetPage> {
    state
        .preset
        .query(keyword, page, page_size)
        .await
        .map_err(to_client_error)
}

#[tauri::command]
pub async fn list_prompt_presets(state: State<'_, AppState>) -> CommandResult<Vec<PromptPreset>> {
    state.preset.list().await.map_err(to_client_error)
}

#[tauri::command]
pub async fn get_prompt_preset(
    state: State<'_, AppState>,
    name: String,
) -> CommandResult<PromptPreset> {
    state.preset.get(name).await.map_err(to_client_error)
}

#[tauri::command]
pub async fn create_prompt_preset(
    state: State<'_, AppState>,
    preset: PromptPreset,
) -> CommandResult<()> {
    state.preset.create(preset).await.map_err(to_client_error)
}

#[tauri::command]
pub async fn update_prompt_preset(
    state: State<'_, AppState>,
    original_name: String,
    preset: PromptPreset,
) -> CommandResult<()> {
    state
        .preset
        .update(original_name, preset)
        .await
        .map_err(to_client_error)
}

#[tauri::command]
pub async fn delete_prompt_preset(
    state: State<'_, AppState>,
    name: String,
) -> CommandResult<()> {
    state.preset.delete(name).await.map_err(to_client_error)
}

#[tauri::command]
pub async fn delete_prompt_presets(
    state: State<'_, AppState>,
    names: Vec<String>,
) -> CommandResult<u32> {
    state.preset.delete_batch(names).await.map_err(to_client_error)
}
