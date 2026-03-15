use std::collections::HashSet;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

use crate::shared::common::Page;
use crate::shared::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyEntry {
    pub key: String,
    pub weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ApiFormat {
    #[default]
    OpenAi,
    Google,
    Anthropic {
        anthropic_version: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ApiGroupStrategy {
    #[default]
    Sequential,
    Random,
    Available,
    Weighted,
}

impl std::fmt::Display for ApiFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiFormat::OpenAi => write!(f, "OpenAI"),
            ApiFormat::Google => write!(f, "Google"),
            ApiFormat::Anthropic { .. } => write!(f, "Anthropic"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub name: String,
    pub format: ApiFormat,
    pub api_url: String,
    pub api_keys: Vec<ApiKeyEntry>,
    pub group_strategy: ApiGroupStrategy,
    pub max_retries_per_key: u32,
    pub model: String,
    pub temperature: f32,
    pub timeout: u32,
    pub requests_per_second: u32,
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub requests_per_day: u32,
}

pub type ProviderPage = Page<Provider>;

impl Default for Provider {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            format: ApiFormat::default(),
            api_url: "https://api.openai.com/v1".to_string(),
            api_keys: Vec::new(),
            group_strategy: ApiGroupStrategy::Sequential,
            max_retries_per_key: 2,
            model: String::new(),
            temperature: 0.3,
            timeout: 120,
            requests_per_second: 0,
            requests_per_minute: 60,
            requests_per_hour: 0,
            requests_per_day: 0,
        }
    }
}

impl Provider {
    pub(crate) async fn init_schema(pool: &SqlitePool) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS providers (
                name TEXT PRIMARY KEY,
                format_type TEXT NOT NULL,
                anthropic_version TEXT,
                api_url TEXT NOT NULL,
                api_keys_json TEXT NOT NULL,
                group_strategy TEXT NOT NULL,
                max_retries_per_key INTEGER NOT NULL,
                model TEXT NOT NULL,
                temperature REAL NOT NULL,
                timeout INTEGER NOT NULL,
                requests_per_second INTEGER NOT NULL DEFAULT 0,
                requests_per_minute INTEGER NOT NULL DEFAULT 60,
                requests_per_hour INTEGER NOT NULL DEFAULT 0,
                requests_per_day INTEGER NOT NULL DEFAULT 0,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(pool)
        .await
        .context("初始化 providers 失败")?;

        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        if self.api_url.trim().is_empty() {
            return Err(AppError::RequiredField { field: "api_url" }.into());
        }
        if self.timeout == 0 {
            return Err(AppError::Validation("timeout 必须大于 0".to_string()).into());
        }
        if self.model.trim().is_empty() {
            return Err(AppError::RequiredField { field: "model" }.into());
        }

        let mut valid_keys = 0usize;
        for entry in &self.api_keys {
            if entry.key.trim().is_empty() {
                continue;
            }
            if entry.weight <= 0.0 {
                return Err(AppError::Validation("api_keys.weight 必须大于 0".to_string()).into());
            }
            valid_keys += 1;
        }
        if valid_keys == 0 {
            return Err(AppError::Validation("api_keys 至少提供一个有效 key".to_string()).into());
        }
        Ok(())
    }

    pub fn normalized_api_keys(&self) -> Vec<ApiKeyEntry> {
        self.api_keys
            .iter()
            .filter_map(|entry| {
                let key = entry.key.trim();
                if key.is_empty() {
                    None
                } else {
                    Some(ApiKeyEntry {
                        key: key.to_string(),
                        weight: if entry.weight > 0.0 {
                            entry.weight
                        } else {
                            1.0
                        },
                    })
                }
            })
            .collect()
    }
}
impl Provider {
    pub async fn query(
        pool: &SqlitePool,
        keyword: Option<String>,
        page: u32,
        page_size: u32,
    ) -> Result<ProviderPage> {
        let mut providers = Self::list_all(pool).await?;
        providers.sort_by(|a, b| a.name.cmp(&b.name));

        let keyword = keyword.unwrap_or_default().trim().to_lowercase();
        let filtered: Vec<Provider> = providers
            .into_iter()
            .filter(|provider| {
                if keyword.is_empty() {
                    true
                } else {
                    provider.name.to_lowercase().contains(&keyword)
                }
            })
            .collect();

        let total = filtered.len() as u32;
        let page_size = if page_size == 0 { 10 } else { page_size };
        let total_pages = if total == 0 {
            0
        } else {
            total.div_ceil(page_size)
        };
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

        Ok(ProviderPage {
            items,
            total,
            page,
            page_size,
            total_pages,
        })
    }

    pub async fn list_all(pool: &SqlitePool) -> Result<Vec<Provider>> {
        let rows = sqlx::query(
            r#"
            SELECT
                name,
                format_type,
                anthropic_version,
                api_url,
                api_keys_json,
                group_strategy,
                max_retries_per_key,
                model,
                temperature,
                timeout,
                requests_per_second,
                requests_per_minute,
                requests_per_hour,
                requests_per_day
            FROM providers
            ORDER BY name ASC
            "#,
        )
        .fetch_all(pool)
        .await
        .context("查询 providers 失败")?;

        let mut providers = Vec::with_capacity(rows.len());
        for row in rows {
            providers.push(Self::parse_row(&row)?);
        }
        Ok(providers)
    }

    pub async fn get(pool: &SqlitePool, name: &str) -> Result<Provider> {
        let row = sqlx::query(
            r#"
            SELECT
                name,
                format_type,
                anthropic_version,
                api_url,
                api_keys_json,
                group_strategy,
                max_retries_per_key,
                model,
                temperature,
                timeout,
                requests_per_second,
                requests_per_minute,
                requests_per_hour,
                requests_per_day
            FROM providers
            WHERE name = ?1
            "#,
        )
        .bind(name)
        .fetch_optional(pool)
        .await
        .with_context(|| format!("读取 provider 失败: {}", name))?
        .ok_or_else(|| AppError::NotFound {
            entity: "Provider",
            name: name.to_string(),
        })?;

        Self::parse_row(&row)
    }

    pub async fn create(pool: &SqlitePool, provider: &Provider) -> Result<()> {
        Self::ensure_not_exists(pool, &provider.name).await?;
        Self::upsert(pool, provider).await
    }

    pub async fn update(pool: &SqlitePool, original_name: &str, provider: &Provider) -> Result<()> {
        let exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(1) FROM providers WHERE name = ?1")
            .bind(original_name)
            .fetch_one(pool)
            .await
            .with_context(|| format!("检查 Provider 是否存在失败: {}", original_name))?;
        if exists == 0 {
            return Err(AppError::NotFound {
                entity: "Provider",
                name: original_name.to_string(),
            }
            .into());
        }

        if provider.name != original_name {
            Self::ensure_not_exists(pool, &provider.name).await?;
            sqlx::query("DELETE FROM providers WHERE name = ?1")
                .bind(original_name)
                .execute(pool)
                .await
                .with_context(|| format!("删除旧 Provider 失败: {}", original_name))?;
        }

        Self::upsert(pool, provider).await
    }

    pub async fn delete(pool: &SqlitePool, name: &str) -> Result<()> {
        let affected = sqlx::query("DELETE FROM providers WHERE name = ?1")
            .bind(name)
            .execute(pool)
            .await
            .with_context(|| format!("删除 Provider 失败: {}", name))?
            .rows_affected();
        if affected == 0 {
            return Err(AppError::NotFound {
                entity: "Provider",
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
            removed += sqlx::query("DELETE FROM providers WHERE name = ?1")
                .bind(name)
                .execute(pool)
                .await
                .context("批量删除 Provider 失败")?
                .rows_affected() as u32;
        }
        Ok(removed)
    }

    async fn ensure_not_exists(pool: &SqlitePool, name: &str) -> Result<()> {
        let exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(1) FROM providers WHERE name = ?1")
            .bind(name)
            .fetch_one(pool)
            .await
            .with_context(|| format!("检查 Provider 是否存在失败: {}", name))?;
        if exists > 0 {
            return Err(AppError::AlreadyExists {
                entity: "Provider",
                name: name.to_string(),
            }
            .into());
        }
        Ok(())
    }

    async fn upsert(pool: &SqlitePool, provider: &Provider) -> Result<()> {
        let format_type = match &provider.format {
            ApiFormat::OpenAi => "openai",
            ApiFormat::Google => "google",
            ApiFormat::Anthropic { .. } => "anthropic",
        };
        let anthropic_version = match &provider.format {
            ApiFormat::Anthropic { anthropic_version } => anthropic_version.clone(),
            _ => None,
        };
        let api_keys_json =
            serde_json::to_string(&provider.api_keys).context("序列化 api_keys 失败")?;
        let group_strategy = match provider.group_strategy {
            ApiGroupStrategy::Sequential => "sequential",
            ApiGroupStrategy::Random => "random",
            ApiGroupStrategy::Available => "available",
            ApiGroupStrategy::Weighted => "weighted",
        };

        sqlx::query(
            r#"
            INSERT INTO providers (
                name,
                format_type,
                anthropic_version,
                api_url,
                api_keys_json,
                group_strategy,
                max_retries_per_key,
                model,
                temperature,
                timeout,
                requests_per_second,
                requests_per_minute,
                requests_per_hour,
                requests_per_day,
                updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, CURRENT_TIMESTAMP)
            ON CONFLICT(name) DO UPDATE SET
                format_type = excluded.format_type,
                anthropic_version = excluded.anthropic_version,
                api_url = excluded.api_url,
                api_keys_json = excluded.api_keys_json,
                group_strategy = excluded.group_strategy,
                max_retries_per_key = excluded.max_retries_per_key,
                model = excluded.model,
                temperature = excluded.temperature,
                timeout = excluded.timeout,
                requests_per_second = excluded.requests_per_second,
                requests_per_minute = excluded.requests_per_minute,
                requests_per_hour = excluded.requests_per_hour,
                requests_per_day = excluded.requests_per_day,
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(&provider.name)
        .bind(format_type)
        .bind(anthropic_version)
        .bind(&provider.api_url)
        .bind(api_keys_json)
        .bind(group_strategy)
        .bind(provider.max_retries_per_key as i64)
        .bind(&provider.model)
        .bind(provider.temperature as f64)
        .bind(provider.timeout as i64)
        .bind(provider.requests_per_second as i64)
        .bind(provider.requests_per_minute as i64)
        .bind(provider.requests_per_hour as i64)
        .bind(provider.requests_per_day as i64)
        .execute(pool)
        .await
        .with_context(|| format!("写入 Provider 失败: {}", provider.name))?;

        Ok(())
    }

    fn parse_row(row: &sqlx::sqlite::SqliteRow) -> Result<Provider> {
        let format_type: String = row
            .try_get("format_type")
            .context("读取 providers.format_type 失败")?;
        let anthropic_version: Option<String> = row
            .try_get("anthropic_version")
            .context("读取 providers.anthropic_version 失败")?;
        let api_keys_json: String = row
            .try_get("api_keys_json")
            .context("读取 providers.api_keys_json 失败")?;
        let api_keys = serde_json::from_str(&api_keys_json).unwrap_or_default();

        let format = match format_type.trim().to_ascii_lowercase().as_str() {
            "openai" => ApiFormat::OpenAi,
            "google" => ApiFormat::Google,
            "anthropic" => ApiFormat::Anthropic { anthropic_version },
            _ => ApiFormat::OpenAi,
        };

        let group_strategy_raw: String = row
            .try_get("group_strategy")
            .context("读取 providers.group_strategy 失败")?;
        let group_strategy = match group_strategy_raw.trim().to_ascii_lowercase().as_str() {
            "random" => ApiGroupStrategy::Random,
            "available" => ApiGroupStrategy::Available,
            "weighted" => ApiGroupStrategy::Weighted,
            _ => ApiGroupStrategy::Sequential,
        };

        Ok(Provider {
            name: row.try_get("name").context("读取 providers.name 失败")?,
            format,
            api_url: row
                .try_get("api_url")
                .context("读取 providers.api_url 失败")?,
            api_keys,
            group_strategy,
            max_retries_per_key: row
                .try_get::<i64, _>("max_retries_per_key")
                .context("读取 providers.max_retries_per_key 失败")?
                .max(0) as u32,
            model: row.try_get("model").context("读取 providers.model 失败")?,
            temperature: row
                .try_get::<f64, _>("temperature")
                .context("读取 providers.temperature 失败")? as f32,
            timeout: row
                .try_get::<i64, _>("timeout")
                .context("读取 providers.timeout 失败")?
                .max(1) as u32,
            requests_per_second: row
                .try_get::<i64, _>("requests_per_second")
                .context("读取 providers.requests_per_second 失败")?
                .max(0) as u32,
            requests_per_minute: row
                .try_get::<i64, _>("requests_per_minute")
                .context("读取 providers.requests_per_minute 失败")?
                .max(0) as u32,
            requests_per_hour: row
                .try_get::<i64, _>("requests_per_hour")
                .context("读取 providers.requests_per_hour 失败")?
                .max(0) as u32,
            requests_per_day: row
                .try_get::<i64, _>("requests_per_day")
                .context("读取 providers.requests_per_day 失败")?
                .max(0) as u32,
        })
    }

}
