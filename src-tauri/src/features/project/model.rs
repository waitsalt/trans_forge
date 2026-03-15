use std::collections::HashSet;

use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

use crate::features::translation::{FileType, ItemStatus, TranslationItem, TranslationProgress};
use crate::shared::common::{Language, Page};
use crate::shared::error::AppError;

fn default_now() -> DateTime<Utc> {
    Utc::now()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ProjectRunStatus {
    #[default]
    NotStarted,
    Running,
    Paused,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub source_language: Language,
    pub target_language: Language,
    pub provider_name: String,
    pub concurrent_limit: u32,
    pub prompt: Option<String>,
    pub input_path: String,
    pub output_path: String,
    pub run_status: ProjectRunStatus,
    #[serde(default = "default_now")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "default_now")]
    pub updated_at: DateTime<Utc>,
}

impl Project {
    pub fn new(name: String) -> Self {
        Self {
            name,
            source_language: Language::JA,
            target_language: Language::ZH,
            provider_name: String::new(),
            concurrent_limit: 1,
            prompt: None,
            input_path: String::new(),
            output_path: String::new(),
            run_status: ProjectRunStatus::NotStarted,
            created_at: default_now(),
            updated_at: default_now(),
        }
    }
}

pub type ProjectPage = Page<Project>;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectRuntimeSnapshot {
    pub name: String,
    pub status: ProjectRunStatus,
    pub total: usize,
    pub processed: usize,
    pub error: usize,
}
impl Project {
    pub(crate) async fn init_schema(pool: &SqlitePool) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS projects (
                name TEXT PRIMARY KEY,
                source_language TEXT NOT NULL,
                target_language TEXT NOT NULL,
                provider_name TEXT NOT NULL,
                concurrent_limit INTEGER NOT NULL DEFAULT 1,
                prompt TEXT,
                run_status TEXT NOT NULL DEFAULT 'not_started',
                input_path TEXT NOT NULL,
                output_path TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(pool)
        .await
        .context("初始化 projects 表失败")?;

        add_column_if_missing(pool, "projects", "provider_name TEXT")
            .await
            .context("补齐 projects.provider_name 列失败")?;
        if let Err(error) = sqlx::query(
            "UPDATE projects SET provider_name = api_config_name WHERE provider_name IS NULL OR TRIM(provider_name) = ''",
        )
        .execute(pool)
        .await
            && !error.to_string().to_ascii_lowercase().contains("no such column")
        {
            return Err(error).context("迁移 projects.provider_name 数据失败");
        }

        add_column_if_missing(pool, "projects", "prompt TEXT")
            .await
            .context("补齐 projects.prompt 列失败")?;
        add_column_if_missing(
            pool,
            "projects",
            "concurrent_limit INTEGER NOT NULL DEFAULT 1",
        )
        .await
        .context("补齐 projects.concurrent_limit 列失败")?;
        add_column_if_missing(
            pool,
            "projects",
            "run_status TEXT NOT NULL DEFAULT 'not_started'",
        )
        .await
        .context("补齐 projects.run_status 列失败")?;

        sqlx::query(
            "UPDATE projects SET concurrent_limit = 1 WHERE concurrent_limit IS NULL OR concurrent_limit <= 0",
        )
        .execute(pool)
        .await
        .context("修正 projects.concurrent_limit 默认值失败")?;
        sqlx::query(
            "UPDATE projects SET run_status = 'not_started' WHERE run_status IS NULL OR TRIM(run_status) = ''",
        )
        .execute(pool)
        .await
        .context("修正 projects.run_status 默认值失败")?;

        Ok(())
    }

    pub async fn query(
        pool: &SqlitePool,
        keyword: Option<String>,
        run_statuses: Option<Vec<ProjectRunStatus>>,
        page: u32,
        page_size: u32,
    ) -> Result<ProjectPage> {
        let mut projects = Self::list_all(pool).await?;
        projects.sort_by(|a, b| a.name.cmp(&b.name));

        let keyword = keyword.unwrap_or_default().trim().to_lowercase();
        let selected_statuses: Option<HashSet<ProjectRunStatus>> =
            run_statuses.map(|values| values.into_iter().collect());
        let filtered: Vec<Project> = projects
            .into_iter()
            .filter(|project| {
                let keyword_ok = if keyword.is_empty() {
                    true
                } else {
                    project.name.to_lowercase().contains(&keyword)
                };
                let status_ok = selected_statuses
                    .as_ref()
                    .map(|statuses| statuses.contains(&project.run_status))
                    .unwrap_or(true);
                keyword_ok && status_ok
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

        Ok(ProjectPage {
            items,
            total,
            page,
            page_size,
            total_pages,
        })
    }

    pub async fn list_all(pool: &SqlitePool) -> Result<Vec<Project>> {
        let rows = sqlx::query(
            r#"
            SELECT
                name,
                source_language,
                target_language,
                provider_name,
                concurrent_limit,
                prompt,
                run_status,
                input_path,
                output_path,
                created_at,
                updated_at
            FROM projects
            ORDER BY name ASC
            "#,
        )
        .fetch_all(pool)
        .await
        .context("查询 projects 失败")?;

        let mut projects = Vec::with_capacity(rows.len());
        for row in rows {
            let source_language: String = row
                .try_get("source_language")
                .context("读取 project.source_language 失败")?;
            let target_language: String = row
                .try_get("target_language")
                .context("读取 project.target_language 失败")?;
            let run_status: String = row
                .try_get("run_status")
                .context("读取 projects.run_status 失败")?;
            let created_at_raw: String = row
                .try_get("created_at")
                .context("读取 projects.created_at 失败")?;
            let updated_at_raw: String = row
                .try_get("updated_at")
                .context("读取 projects.updated_at 失败")?;
            projects.push(Project {
                name: row.try_get("name").context("读取 project.name 失败")?,
                source_language: Language::parse_or_default(&source_language),
                target_language: Language::parse_or_default(&target_language),
                provider_name: row
                    .try_get("provider_name")
                    .context("读取 projects.provider_name 失败")?,
                concurrent_limit: (row
                    .try_get::<i64, _>("concurrent_limit")
                    .context("读取 projects.concurrent_limit 失败")?
                    .max(1)) as u32,
                prompt: row.try_get("prompt").context("读取 project.prompt 失败")?,
                run_status: Self::parse_status(&run_status),
                input_path: row
                    .try_get("input_path")
                    .context("读取 project.input_path 失败")?,
                output_path: row
                    .try_get("output_path")
                    .context("读取 project.output_path 失败")?,
                created_at: Self::parse_datetime(&created_at_raw)?,
                updated_at: Self::parse_datetime(&updated_at_raw)?,
            });
        }
        Ok(projects)
    }

    pub async fn get(pool: &SqlitePool, name: &str) -> Result<Project> {
        let row = sqlx::query(
            r#"
            SELECT
                name,
                source_language,
                target_language,
                provider_name,
                concurrent_limit,
                prompt,
                run_status,
                input_path,
                output_path,
                created_at,
                updated_at
            FROM projects
            WHERE name = ?1
            "#,
        )
        .bind(name)
        .fetch_optional(pool)
        .await
        .with_context(|| format!("读取 projects 失败: {}", name))?
        .ok_or_else(|| AppError::NotFound {
            entity: "Project",
            name: name.to_string(),
        })?;

        let source_language: String = row
            .try_get("source_language")
            .context("读取 project.source_language 失败")?;
        let target_language: String = row
            .try_get("target_language")
            .context("读取 project.target_language 失败")?;
        let run_status: String = row
            .try_get("run_status")
            .context("读取 projects.run_status 失败")?;
        let created_at_raw: String = row
            .try_get("created_at")
            .context("读取 projects.created_at 失败")?;
        let updated_at_raw: String = row
            .try_get("updated_at")
            .context("读取 projects.updated_at 失败")?;

        Ok(Project {
            name: row.try_get("name").context("读取 project.name 失败")?,
            source_language: Language::parse_or_default(&source_language),
            target_language: Language::parse_or_default(&target_language),
            provider_name: row
                .try_get("provider_name")
                .context("读取 projects.provider_name 失败")?,
            concurrent_limit: (row
                .try_get::<i64, _>("concurrent_limit")
                .context("读取 projects.concurrent_limit 失败")?
                .max(1)) as u32,
            prompt: row.try_get("prompt").context("读取 project.prompt 失败")?,
            run_status: Self::parse_status(&run_status),
            input_path: row
                .try_get("input_path")
                .context("读取 project.input_path 失败")?,
            output_path: row
                .try_get("output_path")
                .context("读取 project.output_path 失败")?,
            created_at: Self::parse_datetime(&created_at_raw)?,
            updated_at: Self::parse_datetime(&updated_at_raw)?,
        })
    }

    pub async fn create(pool: &SqlitePool, project: &Project) -> Result<()> {
        let exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(1) FROM projects WHERE name = ?1")
            .bind(&project.name)
            .fetch_one(pool)
            .await
            .with_context(|| format!("检查 Project 是否存在失败: {}", project.name))?;
        if exists > 0 {
            return Err(AppError::AlreadyExists {
                entity: "Project",
                name: project.name.clone(),
            }
            .into());
        }
        Self::upsert(pool, project).await
    }

    pub async fn update(pool: &SqlitePool, original_name: &str, project: &Project) -> Result<()> {
        let exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(1) FROM projects WHERE name = ?1")
            .bind(original_name)
            .fetch_one(pool)
            .await
            .with_context(|| format!("检查 Project 是否存在失败: {}", original_name))?;
        if exists == 0 {
            return Err(AppError::NotFound {
                entity: "Project",
                name: original_name.to_string(),
            }
            .into());
        }

        if project.name != original_name {
            let target_exists =
                sqlx::query_scalar::<_, i64>("SELECT COUNT(1) FROM projects WHERE name = ?1")
                    .bind(&project.name)
                    .fetch_one(pool)
                    .await
                    .with_context(|| format!("检查目标 Project 名称失败: {}", project.name))?;
            if target_exists > 0 {
                return Err(AppError::AlreadyExists {
                    entity: "Project",
                    name: project.name.clone(),
                }
                .into());
            }
            sqlx::query("DELETE FROM translation_items WHERE project_name = ?1")
                .bind(original_name)
                .execute(pool)
                .await
                .with_context(|| format!("删除旧 Project 关联翻译条目失败: {}", original_name))?;
            sqlx::query("DELETE FROM projects WHERE name = ?1")
                .bind(original_name)
                .execute(pool)
                .await
                .with_context(|| format!("删除旧 Project 失败: {}", original_name))?;
        }

        Self::upsert(pool, project).await
    }

    pub async fn delete(pool: &SqlitePool, name: &str) -> Result<()> {
        let mut tx = pool.begin().await.context("开启删除 Project 事务失败")?;

        sqlx::query("DELETE FROM translation_items WHERE project_name = ?1")
            .bind(name)
            .execute(&mut *tx)
            .await
            .with_context(|| format!("删除 Project 关联翻译条目失败: {}", name))?;

        let affected = sqlx::query("DELETE FROM projects WHERE name = ?1")
            .bind(name)
            .execute(&mut *tx)
            .await
            .with_context(|| format!("删除 Project 失败: {}", name))?
            .rows_affected();
        if affected == 0 {
            tx.rollback().await.context("回滚删除 Project 事务失败")?;
            return Err(AppError::NotFound {
                entity: "Project",
                name: name.to_string(),
            }
            .into());
        }

        tx.commit().await.context("提交删除 Project 事务失败")?;
        Ok(())
    }

    pub async fn delete_batch(pool: &SqlitePool, names: Vec<String>) -> Result<u32> {
        if names.is_empty() {
            return Ok(0);
        }

        let mut tx = pool
            .begin()
            .await
            .context("开启批量删除 Project 事务失败")?;
        let targets: HashSet<String> = names.into_iter().collect();
        let mut removed = 0u32;
        for name in targets {
            sqlx::query("DELETE FROM translation_items WHERE project_name = ?1")
                .bind(&name)
                .execute(&mut *tx)
                .await
                .context("批量删除 Project 关联翻译条目失败")?;

            removed += sqlx::query("DELETE FROM projects WHERE name = ?1")
                .bind(name)
                .execute(&mut *tx)
                .await
                .context("批量删除 Project 失败")?
                .rows_affected() as u32;
        }
        tx.commit().await.context("提交批量删除 Project 事务失败")?;
        Ok(removed)
    }

    pub async fn clear_items(pool: &SqlitePool, name: &str) -> Result<u32> {
        let removed = sqlx::query("DELETE FROM translation_items WHERE project_name = ?1")
            .bind(name)
            .execute(pool)
            .await
            .with_context(|| format!("清除项目翻译缓存失败: {}", name))?
            .rows_affected() as u32;
        Ok(removed)
    }

    pub async fn update_status(
        pool: &SqlitePool,
        name: &str,
        run_status: ProjectRunStatus,
    ) -> Result<()> {
        let affected = sqlx::query(
            r#"
            UPDATE projects
            SET run_status = ?2,
                updated_at = CURRENT_TIMESTAMP
            WHERE name = ?1
            "#,
        )
        .bind(name)
        .bind(format!("{:?}", run_status).to_ascii_lowercase())
        .execute(pool)
        .await
        .with_context(|| format!("更新项目运行状态失败: {}", name))?
        .rows_affected();
        if affected == 0 {
            return Err(AppError::NotFound {
                entity: "Project",
                name: name.to_string(),
            }
            .into());
        }
        Ok(())
    }

    pub async fn list_running(pool: &SqlitePool) -> Result<Vec<String>> {
        let rows =
            sqlx::query("SELECT name FROM projects WHERE run_status = 'running' ORDER BY name ASC")
                .fetch_all(pool)
                .await
                .context("查询运行中项目失败")?;
        let mut names = Vec::with_capacity(rows.len());
        for row in rows {
            names.push(row.try_get("name").context("读取 projects.name 失败")?);
        }
        Ok(names)
    }

    async fn upsert(pool: &SqlitePool, project: &Project) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO projects (
                name,
                source_language,
                target_language,
                provider_name,
                concurrent_limit,
                prompt,
                run_status,
                input_path,
                output_path,
                created_at,
                updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            ON CONFLICT(name) DO UPDATE SET
                source_language = excluded.source_language,
                target_language = excluded.target_language,
                provider_name = excluded.provider_name,
                concurrent_limit = excluded.concurrent_limit,
                prompt = excluded.prompt,
                run_status = excluded.run_status,
                input_path = excluded.input_path,
                output_path = excluded.output_path,
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(&project.name)
        .bind(format!("{:?}", project.source_language))
        .bind(format!("{:?}", project.target_language))
        .bind(&project.provider_name)
        .bind(project.concurrent_limit.max(1) as i64)
        .bind(&project.prompt)
        .bind(format!("{:?}", project.run_status).to_ascii_lowercase())
        .bind(&project.input_path)
        .bind(&project.output_path)
        .execute(pool)
        .await
        .with_context(|| format!("写入 Project 失败: {}", project.name))?;

        Ok(())
    }

    fn parse_status(raw: &str) -> ProjectRunStatus {
        match raw.trim().to_ascii_lowercase().as_str() {
            "not_started" => ProjectRunStatus::NotStarted,
            "running" => ProjectRunStatus::Running,
            "paused" => ProjectRunStatus::Paused,
            "completed" => ProjectRunStatus::Completed,
            _ => ProjectRunStatus::NotStarted,
        }
    }

    fn parse_datetime(raw: &str) -> Result<DateTime<Utc>> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(anyhow::anyhow!("解析时间失败: 空字符串"));
        }

        if let Ok(dt) = DateTime::parse_from_rfc3339(trimmed) {
            return Ok(dt.with_timezone(&Utc));
        }

        const SQLITE_TS_FMT: &str = "%Y-%m-%d %H:%M:%S";
        const SQLITE_TS_MILLIS_FMT: &str = "%Y-%m-%d %H:%M:%S%.f";

        if let Ok(naive) = NaiveDateTime::parse_from_str(trimmed, SQLITE_TS_FMT) {
            return Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc));
        }

        if let Ok(naive) = NaiveDateTime::parse_from_str(trimmed, SQLITE_TS_MILLIS_FMT) {
            return Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc));
        }

        Err(anyhow::anyhow!("解析时间失败: {}", raw))
    }
}

impl Project {
    pub async fn save(pool: &SqlitePool, project: &Project) -> Result<()> {
        let provider_name = project.provider_name.trim();
        if provider_name.is_empty() {
            return Err(AppError::RequiredField {
                field: "provider_name",
            }
            .into());
        }

        sqlx::query(
            r#"
            INSERT INTO projects (
                name, source_language, target_language, provider_name, concurrent_limit, prompt,
                run_status, input_path, output_path, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'not_started', ?7, ?8, ?9, ?10)
            ON CONFLICT(name) DO UPDATE SET
                source_language = excluded.source_language,
                target_language = excluded.target_language,
                provider_name = excluded.provider_name,
                concurrent_limit = excluded.concurrent_limit,
                prompt = excluded.prompt,
                input_path = excluded.input_path,
                output_path = excluded.output_path,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(&project.name)
        .bind(format!("{:?}", project.source_language))
        .bind(format!("{:?}", project.target_language))
        .bind(provider_name)
        .bind(project.concurrent_limit.max(1) as i64)
        .bind(project.prompt.clone())
        .bind(&project.input_path)
        .bind(&project.output_path)
        .bind(project.created_at.to_rfc3339())
        .bind(project.updated_at.to_rfc3339())
        .execute(pool)
        .await
        .context("保存项目失败")?;

        Ok(())
    }

    pub async fn load(pool: &SqlitePool, name: &str) -> Result<Option<Project>> {
        let row = sqlx::query(
            r#"
            SELECT
                p.name,
                p.source_language,
                p.target_language,
                p.provider_name,
                p.concurrent_limit,
                p.prompt,
                p.run_status,
                p.input_path,
                p.output_path,
                p.created_at,
                p.updated_at
            FROM projects p
            WHERE p.name = ?1
            "#,
        )
        .bind(name)
        .fetch_optional(pool)
        .await
        .with_context(|| format!("加载项目失败: {}", name))?;

        let Some(row) = row else {
            return Ok(None);
        };

        let source_lang: String = row
            .try_get("source_language")
            .context("读取 source_language 失败")?;
        let target_lang: String = row
            .try_get("target_language")
            .context("读取 target_language 失败")?;
        let provider_name: String = row
            .try_get("provider_name")
            .context("读取 provider_name 失败")?;
        let concurrent_limit: i64 = row
            .try_get("concurrent_limit")
            .context("读取 concurrent_limit 失败")?;
        let prompt: Option<String> = row.try_get("prompt").context("读取 prompt 失败")?;
        let run_status: String = row.try_get("run_status").context("读取 run_status 失败")?;
        let input_path: String = row.try_get("input_path").context("读取 input_path 失败")?;
        let output_path: String = row
            .try_get("output_path")
            .context("读取 output_path 失败")?;
        let created_at: String = row.try_get("created_at").context("读取 created_at 失败")?;
        let updated_at: String = row.try_get("updated_at").context("读取 updated_at 失败")?;
        let name: String = row.try_get("name").context("读取 name 失败")?;

        let created_at = DateTime::parse_from_rfc3339(&created_at)
            .with_context(|| format!("解析 created_at 失败: {}", created_at))?
            .with_timezone(&Utc);
        let updated_at = DateTime::parse_from_rfc3339(&updated_at)
            .with_context(|| format!("解析 updated_at 失败: {}", updated_at))?
            .with_timezone(&Utc);

        Ok(Some(Project {
            name,
            created_at,
            updated_at,
            source_language: Language::parse_or_default(&source_lang),
            target_language: Language::parse_or_default(&target_lang),
            provider_name,
            concurrent_limit: concurrent_limit.max(1) as u32,
            prompt,
            input_path,
            output_path,
            run_status: Self::parse_status(&run_status),
        }))
    }

    pub async fn save_items(
        pool: &SqlitePool,
        project_name: &str,
        items: &[TranslationItem],
    ) -> Result<()> {
        let mut tx = pool.begin().await.context("开启保存条目事务失败")?;
        sqlx::query("DELETE FROM translation_items WHERE project_name = ?1")
            .bind(project_name)
            .execute(&mut *tx)
            .await
            .with_context(|| format!("删除旧条目失败: {}", project_name))?;

        for item in items {
            sqlx::query(
                r#"
                INSERT INTO translation_items
                (id, project_name, file_type, file_path, item_index, source_text, translated_text, status, error_message)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                "#,
            )
            .bind(&item.id)
            .bind(project_name)
            .bind(format!("{:?}", item.file_type))
            .bind(&item.file_path)
            .bind(item.index)
            .bind(&item.source_text)
            .bind(&item.translated_text)
            .bind(format!("{:?}", item.status))
            .bind(&item.error_message)
            .execute(&mut *tx)
            .await
            .with_context(|| format!("写入翻译条目失败: {}", item.id))?;
        }

        tx.commit().await.context("提交保存条目事务失败")?;
        Ok(())
    }

    pub async fn upsert_items(
        pool: &SqlitePool,
        project_name: &str,
        items: &[TranslationItem],
    ) -> Result<()> {
        let mut tx = pool.begin().await.context("开启增量保存条目事务失败")?;

        for item in items {
            sqlx::query(
                r#"
                INSERT INTO translation_items
                (id, project_name, file_type, file_path, item_index, source_text, translated_text, status, error_message)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                ON CONFLICT(id) DO UPDATE SET
                    project_name = excluded.project_name,
                    file_type = excluded.file_type,
                    file_path = excluded.file_path,
                    item_index = excluded.item_index,
                    source_text = excluded.source_text,
                    translated_text = excluded.translated_text,
                    status = excluded.status,
                    error_message = excluded.error_message
                "#,
            )
            .bind(&item.id)
            .bind(project_name)
            .bind(format!("{:?}", item.file_type))
            .bind(&item.file_path)
            .bind(item.index)
            .bind(&item.source_text)
            .bind(&item.translated_text)
            .bind(format!("{:?}", item.status))
            .bind(&item.error_message)
            .execute(&mut *tx)
            .await
            .with_context(|| format!("增量写入翻译条目失败: {}", item.id))?;
        }

        tx.commit().await.context("提交增量保存条目事务失败")?;
        Ok(())
    }

    pub async fn load_progress(
        pool: &SqlitePool,
        project_name: &str,
    ) -> Result<TranslationProgress> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(*) AS total_count,
                SUM(CASE WHEN status = 'Processed' THEN 1 ELSE 0 END) AS processed_count,
                SUM(CASE WHEN status = 'Error' THEN 1 ELSE 0 END) AS error_count
            FROM translation_items
            WHERE project_name = ?1
            "#,
        )
        .bind(project_name)
        .fetch_one(pool)
        .await
        .with_context(|| format!("加载项目进度失败: {}", project_name))?;

        let total: i64 = row
            .try_get("total_count")
            .context("读取 total_count 失败")?;
        let processed: i64 = row
            .try_get::<Option<i64>, _>("processed_count")
            .context("读取 processed_count 失败")?
            .unwrap_or(0);
        let error: i64 = row
            .try_get::<Option<i64>, _>("error_count")
            .context("读取 error_count 失败")?
            .unwrap_or(0);

        Ok(TranslationProgress {
            total: total.max(0) as usize,
            processed: processed.max(0) as usize,
            error: error.max(0) as usize,
            is_running: false,
            current_item: None,
        })
    }

    pub async fn load_items(pool: &SqlitePool, project_name: &str) -> Result<Vec<TranslationItem>> {
        let rows = sqlx::query(
            r#"
            SELECT
                id,
                file_type,
                file_path,
                item_index,
                source_text,
                translated_text,
                status,
                error_message
            FROM translation_items
            WHERE project_name = ?1
            ORDER BY file_path ASC, item_index ASC
            "#,
        )
        .bind(project_name)
        .fetch_all(pool)
        .await
        .with_context(|| format!("加载翻译条目失败: {}", project_name))?;

        let mut items = Vec::with_capacity(rows.len());
        for row in rows {
            let file_type_raw: String = row.try_get("file_type").context("读取 file_type 失败")?;
            let status_raw: String = row.try_get("status").context("读取 status 失败")?;

            items.push(TranslationItem {
                id: row.try_get("id").context("读取 id 失败")?,
                file_type: Self::parse_file_type(&file_type_raw),
                file_path: row.try_get("file_path").context("读取 file_path 失败")?,
                index: row.try_get("item_index").context("读取 item_index 失败")?,
                source_text: row
                    .try_get("source_text")
                    .context("读取 source_text 失败")?,
                translated_text: row
                    .try_get("translated_text")
                    .context("读取 translated_text 失败")?,
                status: Self::parse_item_status(&status_raw),
                error_message: row
                    .try_get("error_message")
                    .context("读取 error_message 失败")?,
            });
        }

        Ok(items)
    }

    fn parse_file_type(raw: &str) -> FileType {
        match raw.trim().to_ascii_lowercase().as_str() {
            "txt" => FileType::Txt,
            "md" => FileType::Md,
            "srt" => FileType::Srt,
            "ass" => FileType::Ass,
            "epub" => FileType::Epub,
            "xlsx" => FileType::Xlsx,
            "json" => FileType::Json,
            _ => FileType::Unknown,
        }
    }

    fn parse_item_status(raw: &str) -> ItemStatus {
        match raw.trim().to_ascii_lowercase().as_str() {
            "none" => ItemStatus::None,
            "processing" => ItemStatus::Processing,
            "processed" => ItemStatus::Processed,
            "error" => ItemStatus::Error,
            "excluded" => ItemStatus::Excluded,
            _ => ItemStatus::None,
        }
    }
}

async fn add_column_if_missing(
    pool: &SqlitePool,
    table: &str,
    column_definition: &str,
) -> Result<()> {
    let sql = format!("ALTER TABLE {} ADD COLUMN {}", table, column_definition);
    match sqlx::query(&sql).execute(pool).await {
        Ok(_) => Ok(()),
        Err(error) => {
            let message = error.to_string().to_ascii_lowercase();
            if message.contains("duplicate column name") {
                Ok(())
            } else {
                Err(error).with_context(|| format!("执行 SQL 失败: {}", sql))
            }
        }
    }
}
