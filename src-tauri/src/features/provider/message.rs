use anyhow::Result;
use tokio::sync::oneshot;

use crate::features::provider::{Provider, ProviderPage};
use crate::features::translation::TranslationItem;

pub enum ProviderMessage {
    Query {
        keyword: Option<String>,
        format_types: Option<Vec<String>>,
        page: u32,
        page_size: u32,
        resp: oneshot::Sender<Result<ProviderPage>>,
    },
    Get {
        name: String,
        resp: oneshot::Sender<Result<Provider>>,
    },
    Create {
        provider: Provider,
        resp: oneshot::Sender<Result<()>>,
    },
    Update {
        original_name: String,
        provider: Provider,
        resp: oneshot::Sender<Result<()>>,
    },
    Delete {
        name: String,
        resp: oneshot::Sender<Result<()>>,
    },
    DeleteBatch {
        names: Vec<String>,
        resp: oneshot::Sender<Result<u32>>,
    },
    FetchModels {
        provider: Provider,
        resp: oneshot::Sender<Result<Vec<String>>>,
    },
    TranslateItem {
        item: TranslationItem,
        provider: Provider,
        prompt: Option<String>,
        api_key: String,
        source_language: String,
        target_language: String,
        resp: oneshot::Sender<Result<TranslationItem>>,
    },
}
