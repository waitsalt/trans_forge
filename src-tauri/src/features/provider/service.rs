use anyhow::Result;

use crate::features::kv::AppKv;
use crate::features::provider::manager::ProviderManager;
use crate::features::provider::message::ProviderMessage;
use crate::features::provider::{Provider, ProviderPage};
use crate::features::translation::{FileType, TranslationItem};
use crate::shared::actor::Addr;
use crate::shared::error::AppError;
use super::worker::ProviderWorker;

pub struct ProviderService {
    manager: ProviderManager,
}

impl ProviderService {
    pub fn new() -> Self {
        Self {
            manager: ProviderManager::new(),
        }
    }

    pub async fn query(
        &self,
        keyword: Option<String>,
        format_types: Option<Vec<String>>,
        page: u32,
        page_size: u32,
    ) -> Result<ProviderPage> {
        self.manager
            .query(keyword, format_types, page, page_size)
            .await
    }

    pub async fn list(&self) -> Result<Vec<Provider>> {
        let pool = crate::shared::database::get_app_db_pool().await?;
        Provider::list_all(pool).await
    }

    pub async fn get(&self, name: String) -> Result<Provider> {
        self.manager.get(name).await
    }

    pub async fn create(&self, config: Provider) -> Result<()> {
        self.manager.create(config).await
    }

    pub async fn update(&self, original_name: String, config: Provider) -> Result<()> {
        self.manager.update(original_name, config).await
    }

    pub async fn delete(&self, name: String) -> Result<()> {
        self.manager.delete(name).await
    }

    pub async fn delete_batch(&self, names: Vec<String>) -> Result<u32> {
        self.manager.delete_batch(names).await
    }

    pub async fn fetch_models(&self, config: Provider) -> Result<Vec<String>> {
        self.manager.fetch_models(config).await
    }

    pub async fn test_provider(&self, name: String) -> Result<String> {
        let provider = self.manager.get(name).await?;
        let api_key = provider
            .normalized_api_keys()
            .into_iter()
            .map(|entry| entry.key)
            .find(|key| !key.trim().is_empty())
            .ok_or(AppError::NoAvailableApiKey)?;

        let test_text = AppKv::get("default_test_text")
            .await?
            .map(|entry| entry.value)
            .unwrap_or_else(|| "This is a sample sentence for localization testing.".to_string());
        let test_prompt = AppKv::get("default_test_prompt")
            .await?
            .map(|entry| entry.value)
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let source_language = AppKv::get("default_source_language")
            .await?
            .map(|entry| entry.value)
            .unwrap_or_else(|| "EN".to_string());
        let target_language = AppKv::get("default_target_language")
            .await?
            .map(|entry| entry.value)
            .unwrap_or_else(|| "ZH".to_string());

        let item = TranslationItem::new(
            FileType::Txt,
            "__provider_test__".to_string(),
            0,
            test_text,
        );
        let translated = self
            .manager
            .worker_addr()
            .ask(|resp| ProviderMessage::TranslateItem {
                item,
                provider,
                prompt: test_prompt,
                api_key,
                source_language,
                target_language,
                resp,
            })
            .await??;

        Ok(translated.translated_text)
    }

    pub(crate) fn worker_addr(&self) -> Addr<ProviderWorker> {
        self.manager.worker_addr()
    }
}
