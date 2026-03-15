use std::sync::atomic::{AtomicBool, Ordering};

use crate::features::kv::service::KvService;
use crate::features::project::manager::ProjectManager;
use crate::features::preset::service::PresetService;
use crate::features::provider::service::ProviderService;
use crate::features::theme::service::ThemeService;

pub struct AppState {
    pub preset: PresetService,
    pub kv: KvService,
    pub provider: ProviderService,
    pub project: ProjectManager,
    pub theme: ThemeService,
    recovered_running_projects: AtomicBool,
}

impl AppState {
    pub fn new() -> Self {
        let provider = ProviderService::new();
        let project = ProjectManager::new(provider.worker_addr());
        Self {
            preset: PresetService::new(),
            kv: KvService::new(),
            provider,
            project,
            theme: ThemeService::new(),
            recovered_running_projects: AtomicBool::new(false),
        }
    }

    pub async fn ensure_recovered_running_projects(&self) {
        if self
            .recovered_running_projects
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
            && let Err(error) = self.project.resume_running_projects().await
        {
            tracing::warn!("恢复运行中项目失败: {}", error);
        }
    }
}
