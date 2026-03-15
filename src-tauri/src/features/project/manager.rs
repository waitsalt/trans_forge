use std::path::PathBuf;

use anyhow::{Result, anyhow};
use tokio::sync::oneshot;

use crate::features::project::message::{ProjectWorkerMessage, ProjectWorkerStatus};
use crate::features::project::worker::ProjectWorker;
use crate::shared::actor::{Actor, Addr};
use crate::shared::error::AppError;
use crate::shared::common::Language;
use crate::features::project::{
    Project, ProjectPage, ProjectRunStatus,
    ProjectRuntimeSnapshot,
};
use crate::features::provider::Provider;
use crate::features::translation::{ItemStatus, TranslationItem, TranslationProgress};
use crate::shared::database::get_app_db_pool;
use crate::features::project::io::{reader, writer};

use crate::features::provider::worker::ProviderWorker;

pub struct ProjectManager {
    worker: Addr<ProjectWorker>,
}

impl ProjectManager {
    pub fn new(provider_worker: Addr<ProviderWorker>) -> Self {
        let worker = ProjectWorker::new(provider_worker).start(32);
        Self { worker }
    }

    async fn get_status(&self) -> Result<ProjectWorkerStatus> {
        self.worker
            .ask(|resp| ProjectWorkerMessage::GetStatus { resp })
            .await
            .map_err(|error| anyhow!(error))
    }

    pub async fn query_configs(
        &self,
        keyword: Option<String>,
        page: u32,
        page_size: u32,
    ) -> Result<ProjectPage> {
        let pool = get_app_db_pool().await?;
        Project::query(pool, keyword, page, page_size).await
    }

    pub async fn get_config(&self, name: String) -> Result<Project> {
        let pool = get_app_db_pool().await?;
        Project::get(pool, &name).await
    }

    pub async fn create_config(&self, mut config: Project) -> Result<()> {
        config.name = config.name.trim().to_string();
        if config.name.is_empty() {
            anyhow::bail!("项目名称不能为空")
        }
        if config.provider_name.trim().is_empty() {
            anyhow::bail!("provider_name 不能为空")
        }
        config.concurrent_limit = config.concurrent_limit.max(1);
        config.prompt = config
            .prompt
            .as_ref()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        config.run_status = ProjectRunStatus::NotStarted;

        let pool = get_app_db_pool().await?;
        let _ = Provider::get(pool, &config.provider_name).await?;
        Project::create(pool, &config).await
    }

    pub async fn update_config(
        &self,
        original_name: String,
        mut config: Project,
    ) -> Result<()> {
        config.name = config.name.trim().to_string();
        if config.name.is_empty() {
            anyhow::bail!("项目名称不能为空")
        }
        if config.provider_name.trim().is_empty() {
            anyhow::bail!("provider_name 不能为空")
        }
        config.concurrent_limit = config.concurrent_limit.max(1);
        config.prompt = config
            .prompt
            .as_ref()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        let pool = get_app_db_pool().await?;
        if let Ok(existing) = Project::get(pool, &original_name).await {
            config.run_status = existing.run_status;
        }
        let _ = Provider::get(pool, &config.provider_name).await?;
        Project::update(pool, &original_name, &config).await
    }

    pub async fn delete_config(&self, name: String) -> Result<()> {
        let pool = get_app_db_pool().await?;
        Project::delete(pool, &name).await
    }

    pub async fn delete_configs(&self, names: Vec<String>) -> Result<u32> {
        let pool = get_app_db_pool().await?;
        Project::delete_batch(pool, names).await
    }

    pub async fn clear_project_items(&self, name: String) -> Result<u32> {
        let pool = get_app_db_pool().await?;
        Project::clear_items(pool, &name).await
    }

    pub async fn get_project_progress(&self, project_name: &str) -> Result<TranslationProgress> {
        let pool = get_app_db_pool().await?;
        Project::load_progress(pool, project_name).await
    }

    pub async fn get_project_runtime_snapshot(
        &self,
        project_name: &str,
    ) -> Result<ProjectRuntimeSnapshot> {
        let pool = get_app_db_pool().await?;
        let project_config = Project::get(pool, project_name).await?;
        let saved_progress = Project::load_progress(pool, project_name).await?;
        let status = self.get_status().await?;

        let running = status.progress.is_running
            && status
                .project
                .as_ref()
                .map(|project| project.name == project_name)
                .unwrap_or(false);

        let same_project_runtime = status
            .project
            .as_ref()
            .map(|project| project.name == project_name)
            .unwrap_or(false);
        let total = if same_project_runtime {
            status.progress.total.max(saved_progress.total)
        } else {
            saved_progress.total
        };
        let processed = if same_project_runtime {
            status.progress.processed.max(saved_progress.processed)
        } else {
            saved_progress.processed
        };
        let error = if same_project_runtime {
            status.progress.error.max(saved_progress.error)
        } else {
            saved_progress.error
        };

        let run_status = if running {
            ProjectRunStatus::Running
        } else {
            project_config.run_status
        };

        Ok(ProjectRuntimeSnapshot {
            name: project_name.to_string(),
            status: run_status,
            total,
            processed,
            error,
        })
    }

    pub async fn get_all_project_runtime_snapshots(
        &self,
    ) -> Result<Vec<ProjectRuntimeSnapshot>> {
        let pool = get_app_db_pool().await?;
        let projects = Project::list_all(pool).await?;
        let status = self.get_status().await?;
        let running_project_name = status.project.as_ref().map(|project| project.name.clone());

        let mut snapshots = Vec::with_capacity(projects.len());
        for project in projects {
            let saved_progress = Project::load_progress(pool, &project.name).await?;
            let same_project_runtime = running_project_name
                .as_ref()
                .map(|name| name == &project.name)
                .unwrap_or(false);
            let running = same_project_runtime && status.progress.is_running;
            let total = if same_project_runtime {
                status.progress.total.max(saved_progress.total)
            } else {
                saved_progress.total
            };
            let processed = if same_project_runtime {
                status.progress.processed.max(saved_progress.processed)
            } else {
                saved_progress.processed
            };
            let error = if same_project_runtime {
                status.progress.error.max(saved_progress.error)
            } else {
                saved_progress.error
            };
            let run_status = if running {
                ProjectRunStatus::Running
            } else {
                project.run_status
            };
            snapshots.push(ProjectRuntimeSnapshot {
                name: project.name,
                status: run_status,
                total,
                processed,
                error,
            });
        }
        Ok(snapshots)
    }

    pub async fn create_project(
        &self,
        name: String,
        input_path: String,
        output_path: String,
        source_language: String,
        target_language: String,
        provider_name: String,
    ) -> Result<Project> {
        let source_lang = Language::parse(&source_language)?;
        let target_lang = Language::parse(&target_language)?;
        let normalized_provider_name = normalize_required_field(&provider_name, "provider_name")?;

        let pool = get_app_db_pool().await?;
        let _ = Provider::get(pool, &normalized_provider_name).await?;

        let existing_project = Project::get(pool, &name).await.ok();
        let mut project = Project::new(name);
        project.source_language = source_lang;
        project.target_language = target_lang;
        project.provider_name = normalized_provider_name;
        project.input_path = input_path;
        project.output_path = output_path;
        project.concurrent_limit = existing_project
            .as_ref()
            .map(|existing| existing.concurrent_limit)
            .unwrap_or(1)
            .max(1);
        project.prompt = existing_project.and_then(|existing| existing.prompt);
        Project::save(pool, &project).await?;

        self.worker
            .ask(|resp| ProjectWorkerMessage::SetProject {
                project: Some(project.clone()),
                resp,
            })
            .await
            .map_err(|error| anyhow!(error))??;

        Ok(project)
    }

    pub async fn load_project(&self, project_path: String) -> Result<Project> {
        let path = PathBuf::from(&project_path);
        let project_name = path
            .file_stem()
            .and_then(|name| name.to_str())
            .filter(|name| !name.trim().is_empty())
            .or_else(|| {
                let normalized = project_path.trim();
                (!normalized.is_empty()).then_some(normalized)
            })
            .ok_or(AppError::InvalidProjectFileName)?;

        let pool = get_app_db_pool().await?;
        let project = Project::load(pool, project_name)
            .await?
            .ok_or(AppError::NotFound {
                entity: "项目",
                name: project_name.to_string(),
            })?;
        self.worker
            .ask(|resp| ProjectWorkerMessage::SetProject {
                project: Some(project.clone()),
                resp,
            })
            .await
            .map_err(|error| anyhow!(error))??;
        Ok(project)
    }

    pub async fn read_input_files(&self, input_path: String) -> Result<Vec<TranslationItem>> {
        let translation_items = reader::read_files_from_path(&input_path);

        self.worker
            .ask(|resp| ProjectWorkerMessage::SetItems {
                items: translation_items.clone(),
                resp,
            })
            .await
            .map_err(|error| anyhow!(error))??;

        if let Some(project) = self.get_status().await?.project {
            let pool = get_app_db_pool().await?;
            Project::save_items(pool, &project.name, &translation_items).await?;
        }

        Ok(translation_items)
    }

    pub async fn load_project_items(&self, project_name: &str) -> Result<Vec<TranslationItem>> {
        let pool = get_app_db_pool().await?;
        let items = Project::load_items(pool, project_name).await?;
        self.worker
            .ask(|resp| ProjectWorkerMessage::SetItems {
                items: items.clone(),
                resp,
            })
            .await
            .map_err(|error| anyhow!(error))??;
        Ok(items)
    }

    pub async fn get_items(&self) -> Vec<TranslationItem> {
        self.get_status()
            .await
            .map(|status| status.items)
            .unwrap_or_default()
    }

    pub async fn set_items(&self, items: Vec<TranslationItem>) -> Result<()> {
        self.worker
            .ask(|resp| ProjectWorkerMessage::SetItems { items, resp })
            .await
            .map_err(|error| anyhow!(error))?
    }

    pub async fn start_translation(
        &self,
        provider: Provider,
        source_language: String,
        target_language: String,
    ) -> Result<TranslationProgress> {
        provider.validate()?;
        let source_language = Language::parse(&source_language)?;
        let target_language = Language::parse(&target_language)?;
        let status = self.get_status().await?;
        if status.items.is_empty() {
            return Err(AppError::NoTranslatableItems.into());
        }
        let prompt = status
            .project
            .as_ref()
            .and_then(|project| project.prompt.clone());

        let (completion_tx, _completion_rx) = oneshot::channel();
        self.worker
            .send(ProjectWorkerMessage::Start {
                provider,
                prompt,
                source_language,
                target_language,
                resp: completion_tx,
            })
            .await
            .map_err(|error| anyhow!(error))?;

        if let Some(project) = status.project {
            let pool = get_app_db_pool().await?;
            Project::update_status(pool, &project.name, ProjectRunStatus::Running).await?;
        }

        Ok(TranslationProgress {
            total: status.items.len(),
            processed: 0,
            error: 0,
            is_running: true,
            current_item: None,
        })
    }

    pub async fn stop_translation(&self) -> Result<()> {
        let status = self.get_status().await?;
        self.worker
            .ask(|resp| ProjectWorkerMessage::Stop { resp })
            .await
            .map_err(|error| anyhow!(error))??;
        if let Some(project) = status.project {
            let pool = get_app_db_pool().await?;
            let progress = Project::load_progress(pool, &project.name).await?;
            let next =
                if progress.total > 0 && progress.processed + progress.error >= progress.total {
                    ProjectRunStatus::Completed
                } else {
                    ProjectRunStatus::Paused
                };
            Project::update_status(pool, &project.name, next).await?;
        }
        Ok(())
    }

    pub async fn get_progress(&self) -> TranslationProgress {
        self.get_status()
            .await
            .map(|status| status.progress)
            .unwrap_or_default()
    }

    pub async fn export_files(&self, output_path: String) -> Result<usize> {
        let items = self.get_status().await?.items;
        let completed_items: Vec<TranslationItem> = items
            .iter()
            .filter(|item| item.status == ItemStatus::Processed)
            .cloned()
            .collect();
        writer::write_files(&completed_items, &output_path)
    }

    pub async fn resume_running_projects(&self) -> Result<()> {
        let pool = get_app_db_pool().await?;
        let running = Project::list_running(pool).await?;
        if running.is_empty() {
            return Ok(());
        }

        let project_name = running[0].clone();
        let config = Project::get(pool, &project_name).await?;
        let provider = Provider::get(pool, &config.provider_name).await?;
        let project = Project::load(pool, &project_name)
            .await?
            .ok_or(AppError::NotFound {
                entity: "Project",
                name: project_name.clone(),
            })?;
        let items = Project::load_items(pool, &project_name).await?;

        self.worker
            .ask(|resp| ProjectWorkerMessage::SetProject {
                project: Some(project),
                resp,
            })
            .await
            .map_err(|error| anyhow!(error))??;
        self.worker
            .ask(|resp| ProjectWorkerMessage::SetItems { items, resp })
            .await
            .map_err(|error| anyhow!(error))??;
        self.start_translation(
            provider,
            format!("{:?}", config.source_language),
            format!("{:?}", config.target_language),
        )
        .await?;

        for name in running.into_iter().skip(1) {
            let _ = Project::update_status(pool, &name, ProjectRunStatus::Paused).await;
        }
        Ok(())
    }
}

fn normalize_required_field(value: &str, field_name: &'static str) -> Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::RequiredField { field: field_name }.into());
    }
    Ok(trimmed.to_string())
}
