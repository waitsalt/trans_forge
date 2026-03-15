use tauri::State;

use crate::shared::command::{CommandResult, to_client_error};
use crate::features::project::{
    Project, ProjectPage, ProjectRunStatus, ProjectRuntimeSnapshot,
};
use crate::features::translation::TranslationProgress;
use crate::shared::state::AppState;

#[tauri::command]
pub async fn query_projects(
    state: State<'_, AppState>,
    keyword: Option<String>,
    run_statuses: Option<Vec<ProjectRunStatus>>,
    page: u32,
    page_size: u32,
) -> CommandResult<ProjectPage> {
    state.ensure_recovered_running_projects().await;
    state
        .project
        .query_configs(keyword, run_statuses, page, page_size)
        .await
        .map_err(to_client_error)
}

#[tauri::command]
pub async fn list_projects(state: State<'_, AppState>) -> CommandResult<Vec<Project>> {
    state.ensure_recovered_running_projects().await;
    let pool = crate::shared::database::get_app_db_pool()
        .await
        .map_err(to_client_error)?;
    Project::list_all(pool).await.map_err(to_client_error)
}

#[tauri::command]
pub async fn get_project(state: State<'_, AppState>, name: String) -> CommandResult<Project> {
    state.ensure_recovered_running_projects().await;
    state.project.get_config(name).await.map_err(to_client_error)
}

#[tauri::command]
pub async fn get_project_progress(
    state: State<'_, AppState>,
    name: String,
) -> CommandResult<TranslationProgress> {
    state.ensure_recovered_running_projects().await;
    state
        .project
        .get_project_progress(&name)
        .await
        .map_err(to_client_error)
}

#[tauri::command]
pub async fn get_project_runtime_snapshot(
    state: State<'_, AppState>,
    name: String,
) -> CommandResult<ProjectRuntimeSnapshot> {
    state.ensure_recovered_running_projects().await;
    state
        .project
        .get_project_runtime_snapshot(&name)
        .await
        .map_err(to_client_error)
}

#[tauri::command]
pub async fn get_all_project_runtime_snapshots(
    state: State<'_, AppState>,
) -> CommandResult<Vec<ProjectRuntimeSnapshot>> {
    state.ensure_recovered_running_projects().await;
    state
        .project
        .get_all_project_runtime_snapshots()
        .await
        .map_err(to_client_error)
}

#[tauri::command]
pub async fn create_project_profile(
    state: State<'_, AppState>,
    config: Project,
) -> CommandResult<()> {
    state.ensure_recovered_running_projects().await;
    state
        .project
        .create_config(config)
        .await
        .map_err(to_client_error)
}

#[tauri::command]
pub async fn update_project_profile(
    state: State<'_, AppState>,
    original_name: String,
    config: Project,
) -> CommandResult<()> {
    state.ensure_recovered_running_projects().await;
    state
        .project
        .update_config(original_name, config)
        .await
        .map_err(to_client_error)
}

#[tauri::command]
pub async fn delete_project(state: State<'_, AppState>, name: String) -> CommandResult<()> {
    state.ensure_recovered_running_projects().await;
    state.project.delete_config(name).await.map_err(to_client_error)
}

#[tauri::command]
pub async fn delete_projects(state: State<'_, AppState>, names: Vec<String>) -> CommandResult<u32> {
    state.ensure_recovered_running_projects().await;
    state.project.delete_configs(names).await.map_err(to_client_error)
}

#[tauri::command]
pub async fn create_project(
    state: State<'_, AppState>,
    name: String,
    input_path: String,
    output_path: String,
    source_language: String,
    target_language: String,
    provider_name: String,
) -> CommandResult<Project> {
    state.ensure_recovered_running_projects().await;
    state
        .project
        .create_project(
            name,
            input_path,
            output_path,
            source_language,
            target_language,
            provider_name,
        )
        .await
        .map_err(to_client_error)
}

#[tauri::command]
pub async fn load_project(state: State<'_, AppState>, project_path: String) -> CommandResult<Project> {
    state.ensure_recovered_running_projects().await;
    state
        .project
        .load_project(project_path)
        .await
        .map_err(to_client_error)
}

#[tauri::command]
pub async fn clear_project_items(
    state: State<'_, AppState>,
    name: String,
) -> CommandResult<u32> {
    state.ensure_recovered_running_projects().await;
    state
        .project
        .clear_project_items(name)
        .await
        .map_err(to_client_error)
}
