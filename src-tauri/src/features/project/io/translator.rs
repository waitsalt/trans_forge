//! 支持多种 API 格式的 AI 翻译器

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::time::Instant;

use anyhow::Result;
use reqwest::Client;
use serde_json::json;
use tokio::sync::Mutex;
use tokio::time;

use crate::shared::error::AppError;
use crate::features::provider::{ApiFormat, Provider};
use crate::features::translation::{ItemStatus, TranslationItem};

pub struct Translator {
    client: Client,
    is_running: Arc<AtomicBool>,
    rate_limiter: Arc<Mutex<RateLimiterState>>,
}

impl Translator {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            is_running: Arc::new(AtomicBool::new(false)),
            rate_limiter: Arc::new(Mutex::new(RateLimiterState::default())),
        }
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    pub fn start_translation(&self) {
        self.is_running.store(true, Ordering::SeqCst);
    }

    pub fn stop_translation(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }

    pub async fn translate_item(
        &self,
        item: &mut TranslationItem,
        config: &Provider,
        prompt: Option<&String>,
        api_key: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<String> {
        if !self.is_running() {
            return Err(AppError::TranslationStopped.into());
        }

        self.apply_rate_limit(config).await?;

        let result = match &config.format {
            ApiFormat::OpenAi => {
                self.translate_openai(item, config, prompt, api_key, source_lang, target_lang)
                    .await
            }
            ApiFormat::Google => {
                self.translate_google(item, config, prompt, api_key, source_lang, target_lang)
                    .await
            }
            ApiFormat::Anthropic { .. } => {
                self.translate_anthropic(item, config, prompt, api_key, source_lang, target_lang)
                    .await
            }
        };

        match result {
            Ok(text) => {
                item.status = ItemStatus::Processed;
                item.translated_text = text.clone();
                item.error_message = None;
                Ok(text)
            }
            Err(e) => {
                item.status = ItemStatus::Error;
                item.error_message = Some(e.to_string());
                Err(e)
            }
        }
    }

    async fn apply_rate_limit(&self, config: &Provider) -> Result<()> {
        loop {
            let wait_duration = {
                let mut rate_limiter = self.rate_limiter.lock().await;
                rate_limiter.try_acquire(config)
            };
            match wait_duration {
                Some(wait) => time::sleep(wait).await,
                None => return Ok(()),
            }
        }
    }

    async fn translate_openai(
        &self,
        item: &TranslationItem,
        config: &Provider,
        prompt: Option<&String>,
        api_key: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<String> {
        let system_prompt = build_prompt(prompt, source_lang, target_lang);

        let response = self
            .client
            .post(format!(
                "{}/chat/completions",
                config.api_url.trim_end_matches('/')
            ))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .timeout(std::time::Duration::from_secs(u64::from(config.timeout)))
            .json(&json!({
                "model": config.model,
                "messages": [
                    {
                        "role": "system",
                        "content": system_prompt
                    },
                    {
                        "role": "user",
                        "content": &item.source_text
                    }
                ],
                "temperature": config.temperature
            }))
            .send()
            .await
            .map_err(|e| AppError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::ApiError(format!("{}: {}", status, body)).into());
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AppError::ResponseParseFailed(e.to_string()))?;

        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or(AppError::InvalidResponseFormat)?;

        Ok(content.to_string())
    }

    async fn translate_google(
        &self,
        item: &TranslationItem,
        config: &Provider,
        prompt: Option<&String>,
        api_key: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<String> {
        let user_prompt = build_prompt(prompt, source_lang, target_lang);

        let response = self
            .client
            .post(format!(
                "{}/models/{}:generateContent",
                config.api_url.trim_end_matches('/'),
                config.model
            ))
            .header("x-goog-api-key", api_key)
            .header("Content-Type", "application/json")
            .timeout(std::time::Duration::from_secs(u64::from(config.timeout)))
            .json(&json!({
                "contents": [{
                    "parts": [{
                        "text": format!(
                            "{}\n\n{}",
                            user_prompt, item.source_text
                        )
                    }]
                }],
                "generationConfig": {
                    "temperature": config.temperature,
                    "maxOutputTokens": 4096
                }
            }))
            .send()
            .await
            .map_err(|e| AppError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::ApiError(format!("{}: {}", status, body)).into());
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AppError::ResponseParseFailed(e.to_string()))?;

        let content = json["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .ok_or(AppError::InvalidResponseFormat)?;

        Ok(content.to_string())
    }

    async fn translate_anthropic(
        &self,
        item: &TranslationItem,
        config: &Provider,
        prompt: Option<&String>,
        api_key: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<String> {
        let anthropic_version = match &config.format {
            ApiFormat::Anthropic { anthropic_version } => anthropic_version,
            _ => return Err(AppError::InvalidApiFormat.into()),
        };

        let user_prompt = build_prompt(prompt, source_lang, target_lang);

        let mut request = self
            .client
            .post(format!(
                "{}/v1/messages",
                config.api_url.trim_end_matches('/')
            ))
            .header("x-api-key", api_key)
            .header("Content-Type", "application/json")
            .timeout(std::time::Duration::from_secs(u64::from(config.timeout)))
            .json(&json!({
                "model": config.model,
                "messages": [
                    {
                        "role": "user",
                        "content": format!(
                            "{}\n\n{}",
                            user_prompt, item.source_text
                        )
                    }
                ],
                "temperature": config.temperature
            }));
        if let Some(version) = anthropic_version
            .as_deref()
            .filter(|v| !v.trim().is_empty())
        {
            request = request.header("anthropic-version", version);
        }

        let response = request
            .send()
            .await
            .map_err(|e| AppError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::ApiError(format!("{}: {}", status, body)).into());
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AppError::ResponseParseFailed(e.to_string()))?;

        let content = json["content"][0]["text"]
            .as_str()
            .ok_or(AppError::InvalidResponseFormat)?;

        Ok(content.to_string())
    }
}

impl Default for Translator {
    fn default() -> Self {
        Self::new()
    }
}

fn build_prompt(prompt: Option<&String>, source_lang: &str, target_lang: &str) -> String {
    if let Some(p) = prompt {
        p.replace("{source}", source_lang)
            .replace("{target}", target_lang)
    } else {
        format!(
            "You are a professional translator. Translate the following text from {} to {}. Only output the translated text, no explanations.",
            source_lang, target_lang
        )
    }
}

#[derive(Clone, Debug)]
struct TokenBucket {
    capacity: f64,
    refill_rate_per_second: f64,
    available_tokens: f64,
    last_refill_at: Instant,
}

impl TokenBucket {
    fn new(capacity: f64, refill_rate_per_second: f64, now: Instant) -> Self {
        Self {
            capacity,
            refill_rate_per_second,
            available_tokens: capacity,
            last_refill_at: now,
        }
    }

    fn refill(&mut self, now: Instant) {
        let elapsed_seconds = now
            .saturating_duration_since(self.last_refill_at)
            .as_secs_f64();
        if elapsed_seconds <= 0.0 {
            return;
        }

        self.available_tokens = (self.available_tokens
            + elapsed_seconds * self.refill_rate_per_second)
            .min(self.capacity);
        self.last_refill_at = now;
    }

    fn can_consume(&self, token_count: f64) -> bool {
        self.available_tokens + f64::EPSILON >= token_count
    }

    fn consume(&mut self, token_count: f64) {
        self.available_tokens = (self.available_tokens - token_count).max(0.0);
    }

    fn wait_duration(&self, token_count: f64) -> Duration {
        if self.refill_rate_per_second <= 0.0 {
            return Duration::from_secs(1);
        }
        let missing_tokens = (token_count - self.available_tokens).max(0.0);
        if missing_tokens <= 0.0 {
            Duration::from_millis(0)
        } else {
            Duration::from_secs_f64(missing_tokens / self.refill_rate_per_second)
        }
    }
}

#[derive(Debug, Default)]
struct RateLimiterState {
    signature: String,
    second_bucket: Option<TokenBucket>,
    minute_bucket: Option<TokenBucket>,
    hour_bucket: Option<TokenBucket>,
    day_bucket: Option<TokenBucket>,
}

impl RateLimiterState {
    fn try_acquire(&mut self, config: &Provider) -> Option<Duration> {
        let now = Instant::now();
        self.reconfigure_if_needed(config, now);

        let mut wait_duration = Duration::from_millis(0);
        let mut blocked = false;

        blocked |= Self::check_bucket_wait(&mut self.second_bucket, now, &mut wait_duration);
        blocked |= Self::check_bucket_wait(&mut self.minute_bucket, now, &mut wait_duration);
        blocked |= Self::check_bucket_wait(&mut self.hour_bucket, now, &mut wait_duration);
        blocked |= Self::check_bucket_wait(&mut self.day_bucket, now, &mut wait_duration);

        if blocked {
            return Some(wait_duration.max(Duration::from_millis(5)));
        }

        Self::consume_bucket(&mut self.second_bucket);
        Self::consume_bucket(&mut self.minute_bucket);
        Self::consume_bucket(&mut self.hour_bucket);
        Self::consume_bucket(&mut self.day_bucket);
        None
    }

    fn check_bucket_wait(
        bucket: &mut Option<TokenBucket>,
        now: Instant,
        wait_duration: &mut Duration,
    ) -> bool {
        let Some(current_bucket) = bucket.as_mut() else {
            return false;
        };

        current_bucket.refill(now);
        if current_bucket.can_consume(1.0) {
            return false;
        }

        let needed = current_bucket.wait_duration(1.0);
        if needed > *wait_duration {
            *wait_duration = needed;
        }
        true
    }

    fn consume_bucket(bucket: &mut Option<TokenBucket>) {
        if let Some(current_bucket) = bucket.as_mut() {
            current_bucket.consume(1.0);
        }
    }

    fn reconfigure_if_needed(&mut self, config: &Provider, now: Instant) {
        let signature = format!(
            "{}|{}|{}|{}",
            config.requests_per_second,
            config.requests_per_minute,
            config.requests_per_hour,
            config.requests_per_day
        );
        if signature == self.signature {
            return;
        }

        self.signature = signature;
        self.second_bucket = Self::build_window_bucket(config.requests_per_second, 1.0, now);
        self.minute_bucket = Self::build_window_bucket(config.requests_per_minute, 60.0, now);
        self.hour_bucket = Self::build_window_bucket(config.requests_per_hour, 3_600.0, now);
        self.day_bucket = Self::build_window_bucket(config.requests_per_day, 86_400.0, now);
    }

    fn build_window_bucket(limit: u32, window_seconds: f64, now: Instant) -> Option<TokenBucket> {
        if limit == 0 {
            return None;
        }

        let capacity = limit as f64;
        let refill_rate_per_second = capacity / window_seconds;
        Some(TokenBucket::new(capacity, refill_rate_per_second, now))
    }
}
