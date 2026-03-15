use anyhow::Result;

use crate::features::kv::AppKv;

#[derive(Debug, Default)]
pub struct KvService;

impl KvService {
    pub fn new() -> Self {
        Self
    }

    pub async fn get(&self, name: String) -> Result<Option<AppKv>> {
        AppKv::get(&name).await
    }

    pub async fn set(&self, entry: AppKv) -> Result<()> {
        entry.set().await
    }

    pub async fn list(&self) -> Result<Vec<AppKv>> {
        AppKv::list().await
    }

    pub async fn delete(&self, name: String) -> Result<()> {
        AppKv::delete(&name).await
    }
}
