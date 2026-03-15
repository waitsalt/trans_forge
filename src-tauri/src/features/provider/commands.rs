use tauri::State;

use crate::shared::command::{CommandResult, to_client_error};
use crate::features::provider::{Provider, ProviderPage};
use crate::shared::state::AppState;

#[tauri::command]
pub async fn query_providers(
    state: State<'_, AppState>,
    keyword: Option<String>,
    format_types: Option<Vec<String>>,
    page: u32,
    page_size: u32,
) -> CommandResult<ProviderPage> {
    state
        .provider
        .query(keyword, format_types, page, page_size)
        .await
        .map_err(to_client_error)
}

#[tauri::command]
pub async fn list_providers(state: State<'_, AppState>) -> CommandResult<Vec<Provider>> {
    state.provider.list().await.map_err(to_client_error)
}

#[tauri::command]
pub async fn get_provider(state: State<'_, AppState>, name: String) -> CommandResult<Provider> {
    state.provider.get(name).await.map_err(to_client_error)
}

#[tauri::command]
pub async fn create_provider(state: State<'_, AppState>, config: Provider) -> CommandResult<()> {
    state.provider.create(config).await.map_err(to_client_error)
}

#[tauri::command]
pub async fn update_provider(
    state: State<'_, AppState>,
    original_name: String,
    config: Provider,
) -> CommandResult<()> {
    state
        .provider
        .update(original_name, config)
        .await
        .map_err(to_client_error)
}

#[tauri::command]
pub async fn delete_provider(state: State<'_, AppState>, name: String) -> CommandResult<()> {
    state.provider.delete(name).await.map_err(to_client_error)
}

#[tauri::command]
pub async fn delete_providers(
    state: State<'_, AppState>,
    names: Vec<String>,
) -> CommandResult<u32> {
    state.provider.delete_batch(names).await.map_err(to_client_error)
}

#[tauri::command]
pub async fn fetch_models(
    state: State<'_, AppState>,
    config: Provider,
) -> CommandResult<Vec<String>> {
    state.provider.fetch_models(config).await.map_err(to_client_error)
}

#[tauri::command]
pub async fn test_provider(state: State<'_, AppState>, name: String) -> CommandResult<String> {
    state.provider.test_provider(name).await.map_err(to_client_error)
}
