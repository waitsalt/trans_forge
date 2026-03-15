use tauri::State;

use crate::shared::command::{to_client_error, CommandResult};
use crate::shared::state::AppState;

use super::model::{ThemeInput, ThemePreferences, ThemeState};

#[tauri::command]
pub async fn get_theme_state(state: State<'_, AppState>) -> CommandResult<ThemeState> {
    state.theme.state().await.map_err(to_client_error)
}

#[tauri::command]
pub async fn create_theme(
    state: State<'_, AppState>,
    config: ThemeInput,
) -> CommandResult<ThemeState> {
    state
        .theme
        .create_theme(config)
        .await
        .map_err(to_client_error)
}

#[tauri::command]
pub async fn update_theme(
    state: State<'_, AppState>,
    id: i64,
    config: ThemeInput,
) -> CommandResult<ThemeState> {
    state
        .theme
        .update_theme(id, config)
        .await
        .map_err(to_client_error)
}

#[tauri::command]
pub async fn delete_theme(state: State<'_, AppState>, id: i64) -> CommandResult<ThemeState> {
    state
        .theme
        .delete_theme(id)
        .await
        .map_err(to_client_error)
}

#[tauri::command]
pub async fn restore_default_themes(state: State<'_, AppState>) -> CommandResult<ThemeState> {
    state
        .theme
        .restore_defaults()
        .await
        .map_err(to_client_error)
}

#[tauri::command]
pub async fn save_theme_preferences(
    state: State<'_, AppState>,
    prefs: ThemePreferences,
) -> CommandResult<ThemePreferences> {
    state
        .theme
        .save_preferences(prefs)
        .await
        .map_err(to_client_error)
}
