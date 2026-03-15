use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sqlx::Row;

use crate::shared::error::AppError;
use crate::shared::database::get_app_db_pool;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppKv {
    pub name: String,
    pub value: String,
}

impl AppKv {
    pub(crate) async fn init_schema(pool: &sqlx::SqlitePool) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS app_kv (
                name TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(pool)
        .await
        .context("初始化 app_kv 表结构失败")?;

        sqlx::query(
            r#"
            CREATE TRIGGER IF NOT EXISTS trg_app_kv_updated_at
            AFTER UPDATE ON app_kv
            FOR EACH ROW
            BEGIN
                UPDATE app_kv
                SET updated_at = CURRENT_TIMESTAMP
                WHERE name = NEW.name;
            END
            "#,
        )
        .execute(pool)
        .await
        .context("初始化 app_kv 触发器失败")?;

        sqlx::query("INSERT OR IGNORE INTO app_kv (name, value) VALUES (?1, ?2)")
            .bind("default_test_text")
            .bind("This is a sample sentence for localization testing.")
            .execute(pool)
            .await
            .context("写入默认 app_kv 失败: default_test_text")?;
        sqlx::query("INSERT OR IGNORE INTO app_kv (name, value) VALUES (?1, ?2)")
            .bind("default_test_prompt")
            .bind("你是一位拥有 10 年经验的顶级中文文案大师和本地化专家。请基于原文含义进行中文改写，而不是逐句翻译。不要复述或引用原文结构。")
            .execute(pool)
            .await
            .context("写入默认 app_kv 失败: default_test_prompt")?;
        sqlx::query("INSERT OR IGNORE INTO app_kv (name, value) VALUES (?1, ?2)")
            .bind("default_source_language")
            .bind("EN")
            .execute(pool)
            .await
            .context("写入默认 app_kv 失败: default_source_language")?;
        sqlx::query("INSERT OR IGNORE INTO app_kv (name, value) VALUES (?1, ?2)")
            .bind("default_target_language")
            .bind("CN")
            .execute(pool)
            .await
            .context("写入默认 app_kv 失败: default_target_language")?;

        Ok(())
    }

    pub async fn get(name: &str) -> Result<Option<Self>> {
        let key = normalize_name(name)?;
        let pool = get_app_db_pool().await?;
        let row = sqlx::query("SELECT name, value FROM app_kv WHERE name = ?1")
            .bind(&key)
            .fetch_optional(pool)
            .await
            .with_context(|| format!("读取 app_kv 失败: {}", key))?;

        row.map(parse_app_kv_row).transpose()
    }

    pub async fn set(&self) -> Result<()> {
        let key = normalize_name(&self.name)?;
        let pool = get_app_db_pool().await?;
        sqlx::query(
            r#"
            INSERT INTO app_kv (name, value)
            VALUES (?1, ?2)
            ON CONFLICT(name) DO UPDATE SET
                value = excluded.value,
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(&key)
        .bind(&self.value)
        .execute(pool)
        .await
        .with_context(|| format!("写入 app_kv 失败: {}", key))?;
        Ok(())
    }

    pub async fn list() -> Result<Vec<Self>> {
        let pool = get_app_db_pool().await?;
        let rows = sqlx::query("SELECT name, value FROM app_kv ORDER BY name ASC")
            .fetch_all(pool)
            .await
            .context("查询 app_kv 列表失败")?;

        rows.into_iter().map(parse_app_kv_row).collect()
    }

    pub async fn delete(name: &str) -> Result<()> {
        let key = normalize_name(name)?;
        let pool = get_app_db_pool().await?;
        sqlx::query("DELETE FROM app_kv WHERE name = ?1")
            .bind(&key)
            .execute(pool)
            .await
            .with_context(|| format!("删除 app_kv 失败: {}", key))?;
        Ok(())
    }
}

fn normalize_name(name: &str) -> Result<String> {
    let key = name.trim();
    if key.is_empty() {
        return Err(AppError::RequiredField { field: "name" }.into());
    }
    Ok(key.to_string())
}

fn parse_app_kv_row(row: sqlx::sqlite::SqliteRow) -> Result<AppKv> {
    Ok(AppKv {
        name: row.try_get("name").context("读取 app_kv.name 失败")?,
        value: row.try_get("value").context("读取 app_kv.value 失败")?,
    })
}
