use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::Result;
use reqwest::Client;
use serde_json::Value;
use tauri::async_runtime;

use crate::shared::actor::{Actor, Context};
use crate::shared::error::AppError;
use crate::features::provider::{ApiFormat, Provider};
use crate::shared::database::get_app_db_pool;
use crate::features::project::io::translator::Translator;

use super::message::ProviderMessage;

pub struct ProviderWorker {
    translator: Arc<Translator>,
    active_translations: Arc<AtomicUsize>,
}

impl Default for ProviderWorker {
    fn default() -> Self {
        Self {
            translator: Arc::new(Translator::new()),
            active_translations: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl Actor for ProviderWorker {
    type Message = ProviderMessage;

    async fn handle(&mut self, msg: Self::Message, _ctx: &Context<Self>) -> Result<()> {
        match msg {
            ProviderMessage::Query {
                keyword,
                format_types,
                page,
                page_size,
                resp,
            } => {
                let result = async {
                    let pool = get_app_db_pool().await?;
                    Provider::query(pool, keyword, format_types, page, page_size).await
                }
                .await;
                let _ = resp.send(result);
            }
            ProviderMessage::Get { name, resp } => {
                let result = async {
                    let pool = get_app_db_pool().await?;
                    Provider::get(pool, &name).await
                }
                .await;
                let _ = resp.send(result);
            }
            ProviderMessage::Create { provider, resp } => {
                let result = async {
                    let mut provider = provider;
                    provider.validate()?;
                    provider.name = provider.name.trim().to_string();
                    if provider.name.is_empty() {
                        anyhow::bail!("Provider 名称不能为空")
                    }
                    let pool = get_app_db_pool().await?;
                    Provider::create(pool, &provider).await
                }
                .await;
                let _ = resp.send(result);
            }
            ProviderMessage::Update {
                original_name,
                provider,
                resp,
            } => {
                let result = async {
                    let mut provider = provider;
                    provider.validate()?;
                    provider.name = provider.name.trim().to_string();
                    if provider.name.is_empty() {
                        anyhow::bail!("Provider 名称不能为空")
                    }
                    let pool = get_app_db_pool().await?;
                    Provider::update(pool, &original_name, &provider).await
                }
                .await;
                let _ = resp.send(result);
            }
            ProviderMessage::Delete { name, resp } => {
                let result = async {
                    let pool = get_app_db_pool().await?;
                    Provider::delete(pool, &name).await
                }
                .await;
                let _ = resp.send(result);
            }
            ProviderMessage::DeleteBatch { names, resp } => {
                let result = async {
                    let pool = get_app_db_pool().await?;
                    Provider::delete_batch(pool, names).await
                }
                .await;
                let _ = resp.send(result);
            }
            ProviderMessage::FetchModels { provider, resp } => {
                let _ = resp.send(fetch_models(provider).await);
            }
            ProviderMessage::TranslateItem {
                mut item,
                provider,
                prompt,
                api_key,
                source_language,
                target_language,
                resp,
            } => {
                let translator = self.translator.clone();
                let active_translations = self.active_translations.clone();
                if active_translations.fetch_add(1, Ordering::SeqCst) == 0 {
                    translator.start_translation();
                }
                async_runtime::spawn(async move {
                    let result = translator
                        .translate_item(
                            &mut item,
                            &provider,
                            prompt.as_ref(),
                            &api_key,
                            &source_language,
                            &target_language,
                        )
                        .await
                        .map(|_| item);
                    let _ = resp.send(result);

                    if active_translations.fetch_sub(1, Ordering::SeqCst) == 1 {
                        translator.stop_translation();
                    }
                });
            }
        }

        Ok(())
    }
}

async fn fetch_models(config: Provider) -> Result<Vec<String>> {
    let api_key = config
        .normalized_api_keys()
        .into_iter()
        .map(|entry| entry.key)
        .find(|key| !key.trim().is_empty())
        .ok_or(AppError::NoAvailableApiKey)?;
    let base_url = config.api_url.trim_end_matches('/').to_string();
    if base_url.is_empty() {
        return Err(AppError::RequiredField { field: "api_url" }.into());
    }

    let client = Client::new();
    let response = match &config.format {
        ApiFormat::OpenAi => {
            request_openai_models(&client, &base_url, &api_key, config.timeout).await?
        }
        ApiFormat::Google => {
            request_google_models(&client, &base_url, &api_key, config.timeout).await?
        }
        ApiFormat::Anthropic { anthropic_version } => {
            request_anthropic_models(
                &client,
                &base_url,
                &api_key,
                config.timeout,
                anthropic_version.as_deref(),
            )
            .await?
        }
    };

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(AppError::ModelListFetchFailed(format!("{}: {}", status, body)).into());
    }

    let payload: Value = response
        .json()
        .await
        .map_err(|error| AppError::ModelListParseFailed(error.to_string()))?;
    let mut models = extract_model_names(&config.format, &payload);
    models.sort_unstable();
    models.dedup();
    Ok(models)
}

async fn request_openai_models(
    client: &Client,
    base_url: &str,
    api_key: &str,
    timeout_seconds: u32,
) -> Result<reqwest::Response> {
    client
        .get(format!("{}/models", base_url))
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .timeout(std::time::Duration::from_secs(u64::from(timeout_seconds)))
        .send()
        .await
        .map_err(|error| AppError::ModelListFetchFailed(error.to_string()).into())
}

async fn request_google_models(
    client: &Client,
    base_url: &str,
    api_key: &str,
    timeout_seconds: u32,
) -> Result<reqwest::Response> {
    let mut response = client
        .get(format!("{}/models", base_url))
        .header("x-goog-api-key", api_key)
        .header("Content-Type", "application/json")
        .timeout(std::time::Duration::from_secs(u64::from(timeout_seconds)))
        .send()
        .await
        .map_err(|error| AppError::ModelListFetchFailed(error.to_string()))?;

    if !response.status().is_success() {
        response = client
            .get(format!("{}/models?key={}", base_url, api_key))
            .header("Content-Type", "application/json")
            .timeout(std::time::Duration::from_secs(u64::from(timeout_seconds)))
            .send()
            .await
            .map_err(|error| AppError::ModelListFetchFailed(error.to_string()))?;
    }

    Ok(response)
}

async fn request_anthropic_models(
    client: &Client,
    base_url: &str,
    api_key: &str,
    timeout_seconds: u32,
    anthropic_version: Option<&str>,
) -> Result<reqwest::Response> {
    let mut request = client
        .get(format!("{}/v1/models", base_url))
        .header("x-api-key", api_key)
        .header("Content-Type", "application/json")
        .timeout(std::time::Duration::from_secs(u64::from(timeout_seconds)));

    if let Some(version) = anthropic_version.filter(|value| !value.trim().is_empty()) {
        request = request.header("anthropic-version", version);
    }

    request
        .send()
        .await
        .map_err(|error| AppError::ModelListFetchFailed(error.to_string()).into())
}

fn extract_model_names(api_format: &ApiFormat, payload: &Value) -> Vec<String> {
    let from_data = payload
        .get("data")
        .and_then(Value::as_array)
        .into_iter()
        .flat_map(|items| items.iter());
    let from_models = payload
        .get("models")
        .and_then(Value::as_array)
        .into_iter()
        .flat_map(|items| items.iter());

    from_data
        .chain(from_models)
        .filter_map(|item| {
            item.get("id")
                .and_then(Value::as_str)
                .or_else(|| item.get("name").and_then(Value::as_str))
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .map(|name| match api_format {
                    ApiFormat::Google => name.trim_start_matches("models/").to_string(),
                    _ => name.to_string(),
                })
        })
        .collect()
}
