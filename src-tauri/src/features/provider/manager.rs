use anyhow::{Result, anyhow};

use crate::shared::actor::{Actor, Addr};
use crate::features::provider::{Provider, ProviderPage};

use super::message::ProviderMessage;
use super::worker::ProviderWorker;

pub struct ProviderManager {
    worker: Addr<ProviderWorker>,
}

impl ProviderManager {
    pub fn new() -> Self {
        Self {
            worker: ProviderWorker::default().start(64),
        }
    }

    pub fn worker_addr(&self) -> Addr<ProviderWorker> {
        self.worker.clone()
    }

    pub async fn query(
        &self,
        keyword: Option<String>,
        format_types: Option<Vec<String>>,
        page: u32,
        page_size: u32,
    ) -> Result<ProviderPage> {
        self.worker
            .ask(|resp| ProviderMessage::Query {
                keyword,
                format_types,
                page,
                page_size,
                resp,
            })
            .await
            .map_err(|error| anyhow!(error))?
    }

    pub async fn get(&self, name: String) -> Result<Provider> {
        self.worker
            .ask(|resp| ProviderMessage::Get { name, resp })
            .await
            .map_err(|error| anyhow!(error))?
    }

    pub async fn create(&self, provider: Provider) -> Result<()> {
        self.worker
            .ask(|resp| ProviderMessage::Create { provider, resp })
            .await
            .map_err(|error| anyhow!(error))?
    }

    pub async fn update(&self, original_name: String, provider: Provider) -> Result<()> {
        self.worker
            .ask(|resp| ProviderMessage::Update {
                original_name,
                provider,
                resp,
            })
            .await
            .map_err(|error| anyhow!(error))?
    }

    pub async fn delete(&self, name: String) -> Result<()> {
        self.worker
            .ask(|resp| ProviderMessage::Delete { name, resp })
            .await
            .map_err(|error| anyhow!(error))?
    }

    pub async fn delete_batch(&self, names: Vec<String>) -> Result<u32> {
        self.worker
            .ask(|resp| ProviderMessage::DeleteBatch { names, resp })
            .await
            .map_err(|error| anyhow!(error))?
    }

    pub async fn fetch_models(&self, provider: Provider) -> Result<Vec<String>> {
        self.worker
            .ask(|resp| ProviderMessage::FetchModels { provider, resp })
            .await
            .map_err(|error| anyhow!(error))?
    }
}
