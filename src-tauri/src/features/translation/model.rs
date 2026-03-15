use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ItemStatus {
    #[default]
    None,
    Processing,
    Processed,
    Error,
    Excluded,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FileType {
    Txt,
    Md,
    Srt,
    Ass,
    Epub,
    Xlsx,
    Json,
    #[default]
    Unknown,
}

impl FileType {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "txt" => FileType::Txt,
            "md" => FileType::Md,
            "srt" => FileType::Srt,
            "ass" => FileType::Ass,
            "epub" => FileType::Epub,
            "xlsx" | "xls" => FileType::Xlsx,
            "json" => FileType::Json,
            _ => FileType::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationItem {
    pub id: String,
    pub file_type: FileType,
    pub file_path: String,
    pub index: i32,
    pub source_text: String,
    pub translated_text: String,
    pub status: ItemStatus,
    pub error_message: Option<String>,
}

impl TranslationItem {
    pub(crate) async fn init_schema(pool: &SqlitePool) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS translation_items (
                id TEXT PRIMARY KEY,
                project_name TEXT NOT NULL,
                file_type TEXT NOT NULL,
                file_path TEXT NOT NULL,
                item_index INTEGER NOT NULL,
                source_text TEXT NOT NULL,
                translated_text TEXT NOT NULL DEFAULT '',
                status TEXT NOT NULL DEFAULT 'NONE',
                error_message TEXT,
                FOREIGN KEY (project_name) REFERENCES projects(name)
            )
            "#,
        )
        .execute(pool)
        .await
        .context("初始化 translation_items 表失败")?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_items_project ON translation_items(project_name)")
            .execute(pool)
            .await
            .context("初始化 idx_items_project 失败")?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_items_status ON translation_items(status)")
            .execute(pool)
            .await
            .context("初始化 idx_items_status 失败")?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_items_file ON translation_items(file_path)")
            .execute(pool)
            .await
            .context("初始化 idx_items_file 失败")?;

        Ok(())
    }

    pub fn new(file_type: FileType, file_path: String, index: i32, source_text: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            file_type,
            file_path,
            index,
            source_text,
            translated_text: String::new(),
            status: ItemStatus::None,
            error_message: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TranslationProgress {
    pub total: usize,
    pub processed: usize,
    pub error: usize,
    pub is_running: bool,
    pub current_item: Option<String>,
}
