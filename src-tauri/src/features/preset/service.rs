use anyhow::{Result, anyhow};

use crate::features::preset::{PromptPreset, PromptPresetPage};
use crate::shared::common::Language;
use crate::shared::database::get_app_db_pool;

#[derive(Debug, Default)]
pub struct PresetService;

impl PresetService {
    pub fn new() -> Self {
        Self
    }

    pub async fn query(
        &self,
        keyword: Option<String>,
        languages: Option<Vec<Language>>,
        page: u32,
        page_size: u32,
    ) -> Result<PromptPresetPage> {
        let pool = get_app_db_pool().await?;
        PromptPreset::query(pool, keyword, languages, page, page_size).await
    }

    pub async fn list(&self) -> Result<Vec<PromptPreset>> {
        let pool = get_app_db_pool().await?;
        PromptPreset::list_all(pool).await
    }

    pub async fn get(&self, name: String) -> Result<PromptPreset> {
        let pool = get_app_db_pool().await?;
        PromptPreset::get(pool, &name).await
    }

    pub async fn create(&self, mut preset: PromptPreset) -> Result<()> {
        normalize_preset(&mut preset)?;
        let pool = get_app_db_pool().await?;
        PromptPreset::create(pool, &preset).await
    }

    pub async fn update(&self, original_name: String, mut preset: PromptPreset) -> Result<()> {
        normalize_preset(&mut preset)?;
        let pool = get_app_db_pool().await?;
        PromptPreset::update(pool, &original_name, &preset).await
    }

    pub async fn delete(&self, name: String) -> Result<()> {
        let pool = get_app_db_pool().await?;
        PromptPreset::delete(pool, &name).await
    }

    pub async fn delete_batch(&self, names: Vec<String>) -> Result<u32> {
        let pool = get_app_db_pool().await?;
        PromptPreset::delete_batch(pool, names).await
    }
}

fn normalize_preset(preset: &mut PromptPreset) -> Result<()> {
    preset.name = preset.name.trim().to_string();
    if preset.name.is_empty() {
        return Err(anyhow!("提示词名称不能为空"));
    }
    let prompt = preset.prompt.trim().to_string();
    if prompt.is_empty() {
        return Err(anyhow!("提示词内容不能为空"));
    }
    preset.prompt = prompt;
    Ok(())
}
