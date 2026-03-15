use anyhow::Result;
use tokio::sync::oneshot;

use crate::features::project::Project;
use crate::features::provider::Provider;
use crate::features::translation::{TranslationItem, TranslationProgress};
use crate::shared::common::Language;

#[derive(Debug, Clone, Default)]
pub struct ProjectWorkerStatus {
    pub project: Option<Project>,
    pub items: Vec<TranslationItem>,
    pub progress: TranslationProgress,
}

pub enum ProjectWorkerMessage {
    GetStatus {
        resp: oneshot::Sender<ProjectWorkerStatus>,
    },
    SetProject {
        project: Option<Project>,
        resp: oneshot::Sender<Result<()>>,
    },
    SetItems {
        items: Vec<TranslationItem>,
        resp: oneshot::Sender<Result<()>>,
    },
    Start {
        provider: Provider,
        prompt: Option<String>,
        source_language: Language,
        target_language: Language,
        resp: oneshot::Sender<Result<TranslationProgress>>,
    },
    Stop {
        resp: oneshot::Sender<Result<()>>,
    },
    Finished {
        run_id: u64,
    },
}
