use std::collections::HashSet;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

use crate::shared::common::{Language, Page};
use crate::shared::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptPreset {
    pub name: String,
    pub language: Language,
    pub prompt: String,
}

pub type PromptPresetPage = Page<PromptPreset>;

impl PromptPreset {
    pub(crate) async fn init_schema(pool: &SqlitePool) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS prompt_presets (
                name TEXT PRIMARY KEY,
                language TEXT NOT NULL,
                prompt TEXT NOT NULL,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(pool)
        .await
        .context("初始化 prompt_presets 表失败")?;

        Ok(())
    }

    pub async fn query(
        pool: &SqlitePool,
        keyword: Option<String>,
        languages: Option<Vec<Language>>,
        page: u32,
        page_size: u32,
    ) -> Result<PromptPresetPage> {
        let mut presets = Self::list_all(pool).await?;
        presets.sort_by(|a, b| a.name.cmp(&b.name));

        let keyword = keyword.unwrap_or_default().trim().to_lowercase();
        let selected_languages: Option<HashSet<Language>> =
            languages.map(|values| values.into_iter().collect());
        let filtered: Vec<PromptPreset> = presets
            .into_iter()
            .filter(|preset| {
                let keyword_ok = if keyword.is_empty() {
                    true
                } else {
                    preset.name.to_lowercase().contains(&keyword)
                        || format!("{:?}", preset.language)
                            .to_lowercase()
                            .contains(&keyword)
                        || preset.prompt.to_lowercase().contains(&keyword)
                };
                let language_ok = selected_languages
                    .as_ref()
                    .map(|items| items.contains(&preset.language))
                    .unwrap_or(true);
                keyword_ok && language_ok
            })
            .collect();

        let total = filtered.len() as u32;
        let page_size = if page_size == 0 { 10 } else { page_size };
        let total_pages = if total == 0 { 0 } else { total.div_ceil(page_size) };
        let page = if total_pages == 0 {
            0
        } else {
            page.min(total_pages.saturating_sub(1))
        };
        let start = (page as usize).saturating_mul(page_size as usize);
        let end = (start + page_size as usize).min(filtered.len());
        let items = if start >= filtered.len() {
            Vec::new()
        } else {
            filtered[start..end].to_vec()
        };

        Ok(PromptPresetPage {
            items,
            total,
            page,
            page_size,
            total_pages,
        })
    }

    pub async fn list_all(pool: &SqlitePool) -> Result<Vec<PromptPreset>> {
        let rows = sqlx::query(
            r#"
            SELECT
                name,
                language,
                prompt
            FROM prompt_presets
            ORDER BY name ASC
            "#,
        )
        .fetch_all(pool)
        .await
        .context("查询 prompt_presets 失败")?;

        let mut presets = Vec::with_capacity(rows.len());
        for row in rows {
            let language: String = row
                .try_get("language")
                .context("读取 prompt_presets.language 失败")?;
            let prompt: String = row
                .try_get("prompt")
                .context("读取 prompt_presets.prompt 失败")?;
            presets.push(PromptPreset {
                name: row
                    .try_get("name")
                    .context("读取 prompt_presets.name 失败")?,
                language: Language::parse_or_default(&language),
                prompt,
            });
        }
        Ok(presets)
    }

    pub async fn get(pool: &SqlitePool, name: &str) -> Result<PromptPreset> {
        let row = sqlx::query(
            r#"
            SELECT
                name,
                language,
                prompt
            FROM prompt_presets
            WHERE name = ?1
            "#,
        )
        .bind(name)
        .fetch_optional(pool)
        .await
        .with_context(|| format!("读取 prompt_presets 失败: {}", name))?
        .ok_or_else(|| AppError::NotFound {
            entity: "PromptPreset",
            name: name.to_string(),
        })?;

        let language: String = row
            .try_get("language")
            .context("读取 prompt_presets.language 失败")?;
        let prompt: String = row
            .try_get("prompt")
            .context("读取 prompt_presets.prompt 失败")?;
        Ok(PromptPreset {
            name: row
                .try_get("name")
                .context("读取 prompt_presets.name 失败")?,
            language: Language::parse_or_default(&language),
            prompt,
        })
    }

    pub async fn create(pool: &SqlitePool, preset: &PromptPreset) -> Result<()> {
        let exists =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(1) FROM prompt_presets WHERE name = ?1")
                .bind(&preset.name)
                .fetch_one(pool)
                .await
                .with_context(|| format!("检查 PromptPreset 是否存在失败: {}", preset.name))?;
        if exists > 0 {
            return Err(AppError::AlreadyExists {
                entity: "PromptPreset",
                name: preset.name.clone(),
            }
            .into());
        }
        Self::upsert(pool, preset).await
    }

    pub async fn update(
        pool: &SqlitePool,
        original_name: &str,
        preset: &PromptPreset,
    ) -> Result<()> {
        let exists =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(1) FROM prompt_presets WHERE name = ?1")
                .bind(original_name)
                .fetch_one(pool)
                .await
                .with_context(|| format!("检查 PromptPreset 是否存在失败: {}", original_name))?;
        if exists == 0 {
            return Err(AppError::NotFound {
                entity: "PromptPreset",
                name: original_name.to_string(),
            }
            .into());
        }

        if preset.name != original_name {
            let target_exists =
                sqlx::query_scalar::<_, i64>("SELECT COUNT(1) FROM prompt_presets WHERE name = ?1")
                    .bind(&preset.name)
                    .fetch_one(pool)
                    .await
                    .with_context(|| format!("检查目标 PromptPreset 名称失败: {}", preset.name))?;
            if target_exists > 0 {
                return Err(AppError::AlreadyExists {
                    entity: "PromptPreset",
                    name: preset.name.clone(),
                }
                .into());
            }
            sqlx::query("DELETE FROM prompt_presets WHERE name = ?1")
                .bind(original_name)
                .execute(pool)
                .await
                .with_context(|| format!("删除旧 PromptPreset 失败: {}", original_name))?;
        }

        Self::upsert(pool, preset).await
    }

    pub async fn delete(pool: &SqlitePool, name: &str) -> Result<()> {
        let affected = sqlx::query("DELETE FROM prompt_presets WHERE name = ?1")
            .bind(name)
            .execute(pool)
            .await
            .with_context(|| format!("删除 PromptPreset 失败: {}", name))?
            .rows_affected();
        if affected == 0 {
            return Err(AppError::NotFound {
                entity: "PromptPreset",
                name: name.to_string(),
            }
            .into());
        }
        Ok(())
    }

    pub async fn delete_batch(pool: &SqlitePool, names: Vec<String>) -> Result<u32> {
        if names.is_empty() {
            return Ok(0);
        }

        let targets: HashSet<String> = names.into_iter().collect();
        let mut removed = 0u32;
        for name in targets {
            removed += sqlx::query("DELETE FROM prompt_presets WHERE name = ?1")
                .bind(name)
                .execute(pool)
                .await
                .context("批量删除 PromptPreset 失败")?
                .rows_affected() as u32;
        }
        Ok(removed)
    }

    async fn upsert(pool: &SqlitePool, preset: &PromptPreset) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO prompt_presets (name, language, prompt, updated_at)
            VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)
            ON CONFLICT(name) DO UPDATE SET
                language = excluded.language,
                prompt = excluded.prompt,
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(&preset.name)
        .bind(format!("{:?}", preset.language))
        .bind(&preset.prompt)
        .execute(pool)
        .await
        .with_context(|| format!("写入 PromptPreset 失败: {}", preset.name))?;

        Ok(())
    }
}
