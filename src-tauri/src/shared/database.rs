//! 全局数据库连接与初始化入口

use std::ffi::OsStr;
use std::path::PathBuf;

use anyhow::{Context, Result};
use sqlx::SqlitePool;
use sqlx::sqlite::SqliteConnectOptions;
use tokio::sync::OnceCell;

use crate::features::kv::AppKv;
use crate::features::preset::PromptPreset;
use crate::features::project::Project;
use crate::features::provider::Provider;
use crate::features::translation::TranslationItem;

static APP_DB_POOL: OnceCell<SqlitePool> = OnceCell::const_new();

/// 获取应用数据目录
pub fn get_app_data_dir() -> PathBuf {
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if current_dir.file_name() == Some(OsStr::new("src-tauri")) {
        let project_root = current_dir.parent().unwrap_or(&current_dir);
        return project_root.join(".data");
    }
    current_dir.join(".data")
}

/// 获取应用数据库路径
pub fn get_app_db_path() -> PathBuf {
    get_app_data_dir().join("data.db")
}

/// 获取应用数据库连接池（全局单例）
pub async fn get_app_db_pool() -> Result<&'static SqlitePool> {
    APP_DB_POOL
        .get_or_try_init(|| async {
            let db_path = get_app_db_path();
            if let Some(parent_dir) = db_path.parent() {
                tokio::fs::create_dir_all(parent_dir)
                    .await
                    .with_context(|| {
                        format!("创建应用数据目录失败: {}", parent_dir.to_string_lossy())
                    })?;
            }

            let options = SqliteConnectOptions::new()
                .filename(&db_path)
                .create_if_missing(true);
            let pool = SqlitePool::connect_with(options)
                .await
                .with_context(|| format!("打开应用数据库失败: {}", db_path.to_string_lossy()))?;

            init_app_schema(&pool).await?;
            Ok(pool)
        })
        .await
}

async fn init_app_schema(pool: &SqlitePool) -> Result<()> {
    Provider::init_schema(pool).await?;
    Project::init_schema(pool).await?;
    PromptPreset::init_schema(pool).await?;
    TranslationItem::init_schema(pool).await?;
    AppKv::init_schema(pool).await?;
    Ok(())
}
