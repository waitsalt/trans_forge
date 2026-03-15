use tauri::State;

use crate::shared::command::{CommandResult, to_client_error};
use crate::features::kv::AppKv;
use crate::shared::state::AppState;

#[tauri::command]
pub async fn get_app_kv(
    state: State<'_, AppState>,
    name: String,
) -> CommandResult<Option<AppKv>> {
    state.kv.get(name).await.map_err(to_client_error)
}

#[tauri::command]
pub async fn set_app_kv(state: State<'_, AppState>, entry: AppKv) -> CommandResult<()> {
    state.kv.set(entry).await.map_err(to_client_error)
}

#[tauri::command]
pub async fn list_app_kv(state: State<'_, AppState>) -> CommandResult<Vec<AppKv>> {
    state.kv.list().await.map_err(to_client_error)
}

#[tauri::command]
pub async fn delete_app_kv(state: State<'_, AppState>, name: String) -> CommandResult<()> {
    state.kv.delete(name).await.map_err(to_client_error)
}
