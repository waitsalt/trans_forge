use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Result, anyhow};
use tauri::async_runtime;
use tokio::sync::Mutex;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;

use crate::features::provider::message::ProviderMessage;
use crate::features::provider::worker::ProviderWorker;
use crate::shared::actor::{Actor, Addr, Context};
use crate::shared::error::AppError;
use crate::shared::common::Language;
use crate::features::project::{Project, ProjectRunStatus};
use crate::features::provider::{ApiGroupStrategy, Provider};
use crate::features::translation::{ItemStatus, TranslationItem, TranslationProgress};
use crate::shared::database::get_app_db_pool;

use super::message::{ProjectWorkerMessage, ProjectWorkerStatus};

const SNAPSHOT_PERSIST_INTERVAL: Duration = Duration::from_secs(2);

pub struct ProjectWorker {
    state: Arc<Mutex<ProjectWorkerStatus>>,
    provider_worker: Addr<ProviderWorker>,
    seq: u64,
    active: Option<(u64, CancellationToken)>,
}

impl ProjectWorker {
    pub fn new(provider_worker: Addr<ProviderWorker>) -> Self {
        Self {
            state: Arc::new(Mutex::new(ProjectWorkerStatus::default())),
            provider_worker,
            seq: 0,
            active: None,
        }
    }
}

impl Actor for ProjectWorker {
    type Message = ProjectWorkerMessage;

    async fn handle(&mut self, msg: Self::Message, ctx: &Context<Self>) -> Result<()> {
        match msg {
            ProjectWorkerMessage::GetStatus { resp } => {
                let status = self.state.lock().await.clone();
                let _ = resp.send(status);
            }
            ProjectWorkerMessage::SetProject { project, resp } => {
                self.state.lock().await.project = project;
                let _ = resp.send(Ok(()));
            }
            ProjectWorkerMessage::SetItems { items, resp } => {
                self.state.lock().await.items = items;
                let _ = resp.send(Ok(()));
            }
            ProjectWorkerMessage::Start {
                provider,
                prompt,
                source_language,
                target_language,
                resp,
            } => {
                if self.active.is_some() {
                    let _ = resp.send(Err(AppError::TranslationTaskRunning.into()));
                    return Ok(());
                }

                self.seq += 1;
                let run_id = self.seq;
                let cancel = CancellationToken::new();
                self.active = Some((run_id, cancel.clone()));

                let state = Arc::clone(&self.state);
                let provider_worker = self.provider_worker.clone();
                let app_addr = ctx.addr().clone();

                async_runtime::spawn(async move {
                    let result = run_translation(
                        state,
                        provider_worker,
                        provider,
                        prompt,
                        source_language,
                        target_language,
                        cancel,
                    )
                    .await;
                    let _ = resp.send(result);
                    let _ = app_addr
                        .send(ProjectWorkerMessage::Finished { run_id })
                        .await;
                });
            }
            ProjectWorkerMessage::Stop { resp } => {
                if let Some((_, cancel)) = &self.active {
                    cancel.cancel();
                }

                let mut guard = self.state.lock().await;
                guard.progress.is_running = false;
                let _ = resp.send(Ok(()));
            }
            ProjectWorkerMessage::Finished { run_id } => {
                if self.active.as_ref().map(|(id, _)| *id) == Some(run_id) {
                    self.active = None;
                }
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
struct ApiKeyState {
    key: String,
    base_weight: f64,
    unavailable: bool,
    failures: u32,
    latency_ema_ms: Option<f64>,
}

struct ApiKeyPool {
    strategy: ApiGroupStrategy,
    max_retries_per_key: u32,
    entries: Vec<ApiKeyState>,
    cursor: usize,
    latency_influence: f64,
    rng_state: u64,
}

impl ApiKeyPool {
    fn new(config: &Provider) -> Result<Self> {
        let normalized = config.normalized_api_keys();
        if normalized.is_empty() {
            return Err(AppError::NoAvailableApiKey.into());
        }

        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E3779B97F4A7C15);

        Ok(Self {
            strategy: config.group_strategy.clone(),
            max_retries_per_key: config.max_retries_per_key,
            entries: normalized
                .into_iter()
                .map(|entry| ApiKeyState {
                    key: entry.key,
                    base_weight: f64::from(entry.weight.max(0.0001)),
                    unavailable: false,
                    failures: 0,
                    latency_ema_ms: None,
                })
                .collect(),
            cursor: 0,
            latency_influence: 1.0,
            rng_state: seed,
        })
    }

    fn has_available(&self) -> bool {
        self.entries.iter().any(|entry| !entry.unavailable)
    }

    fn next_index(&mut self) -> Option<usize> {
        if !self.has_available() {
            return None;
        }

        match self.strategy {
            ApiGroupStrategy::Sequential => self.next_sequential_index(),
            ApiGroupStrategy::Random => self.next_random_index(),
            ApiGroupStrategy::Available => self.next_available_index(),
            ApiGroupStrategy::Weighted => self.next_weighted_index(),
        }
    }

    fn key_at(&self, index: usize) -> &str {
        &self.entries[index].key
    }

    fn report_success(&mut self, index: usize, latency_ms: f64) {
        let entry = &mut self.entries[index];
        entry.failures = 0;
        entry.latency_ema_ms = Some(match entry.latency_ema_ms {
            Some(prev) => prev * 0.7 + latency_ms * 0.3,
            None => latency_ms,
        });
    }

    fn report_failure(&mut self, index: usize) {
        let entry = &mut self.entries[index];
        entry.failures = entry.failures.saturating_add(1);
        if entry.failures > self.max_retries_per_key {
            entry.unavailable = true;
        }
    }

    fn next_sequential_index(&mut self) -> Option<usize> {
        let len = self.entries.len();
        for _ in 0..len {
            let idx = self.cursor % len;
            self.cursor = self.cursor.wrapping_add(1);
            if !self.entries[idx].unavailable {
                return Some(idx);
            }
        }
        None
    }

    fn next_random_index(&mut self) -> Option<usize> {
        let available: Vec<usize> = self
            .entries
            .iter()
            .enumerate()
            .filter_map(|(idx, entry)| (!entry.unavailable).then_some(idx))
            .collect();
        if available.is_empty() {
            return None;
        }
        let pick = (self.next_rand_f64() * available.len() as f64).floor() as usize;
        available
            .get(pick)
            .copied()
            .or_else(|| available.last().copied())
    }

    fn next_available_index(&mut self) -> Option<usize> {
        self.entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| !entry.unavailable)
            .min_by(|(_, a), (_, b)| {
                a.failures
                    .cmp(&b.failures)
                    .then_with(|| a.key.len().cmp(&b.key.len()))
            })
            .map(|(idx, _)| idx)
    }

    fn next_weighted_index(&mut self) -> Option<usize> {
        let avg_latency = {
            let latencies: Vec<f64> = self
                .entries
                .iter()
                .filter_map(|entry| {
                    (!entry.unavailable)
                        .then_some(entry.latency_ema_ms)
                        .flatten()
                })
                .collect();
            if latencies.is_empty() {
                None
            } else {
                Some(latencies.iter().sum::<f64>() / latencies.len() as f64)
            }
        };

        let mut candidates: Vec<(usize, f64)> = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| !entry.unavailable)
            .map(|(idx, entry)| {
                let speed_factor = match (entry.latency_ema_ms, avg_latency) {
                    (Some(lat), Some(avg)) if lat > 0.0 => (avg / lat).clamp(0.5, 1.5),
                    _ => 1.0,
                };
                let score =
                    entry.base_weight * (1.0 + self.latency_influence * (speed_factor - 1.0));
                (idx, score.max(0.0001))
            })
            .collect();

        if candidates.is_empty() {
            return None;
        }

        self.latency_influence = (self.latency_influence * 0.97).max(0.0);

        let total: f64 = candidates.iter().map(|(_, score)| *score).sum();
        let mut target = self.next_rand_f64() * total;
        for (idx, score) in candidates.drain(..) {
            if target <= score {
                return Some(idx);
            }
            target -= score;
        }
        None
    }

    fn next_rand_f64(&mut self) -> f64 {
        let mut x = if self.rng_state == 0 {
            0x9E3779B97F4A7C15
        } else {
            self.rng_state
        };
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.rng_state = x;
        let val = x.wrapping_mul(0x2545F4914F6CDD1D);
        (val as f64) / (u64::MAX as f64)
    }
}

async fn run_translation(
    state: Arc<Mutex<ProjectWorkerStatus>>,
    provider_worker: Addr<ProviderWorker>,
    provider: Provider,
    prompt: Option<String>,
    source_language: Language,
    target_language: Language,
    cancel: CancellationToken,
) -> Result<TranslationProgress> {
    let source_lang = source_language.to_string();
    let target_lang = target_language.to_string();

    let (project_name, configured_concurrent_limit, mut items) = {
        let guard = state.lock().await;
        (
            guard.project.as_ref().map(|project| project.name.clone()),
            guard
                .project
                .as_ref()
                .map(|project| project.concurrent_limit.max(1))
                .unwrap_or(1),
            guard.items.clone(),
        )
    };

    if items.is_empty() {
        return Err(AppError::NoTranslatableItems.into());
    }

    let total = items.len();
    let concurrent_limit = (configured_concurrent_limit as usize)
        .max(1)
        .min(total.max(1));
    let key_pool = Arc::new(Mutex::new(ApiKeyPool::new(&provider)?));
    let runtime = Arc::new(TranslationRuntime {
        provider_worker: provider_worker.clone(),
        key_pool: key_pool.clone(),
        provider: provider.clone(),
        prompt: prompt.clone(),
        source_lang: source_lang.clone(),
        target_lang: target_lang.clone(),
        cancel: cancel.clone(),
    });

    let mut progress = TranslationProgress {
        total,
        processed: 0,
        error: 0,
        is_running: true,
        current_item: None,
    };
    {
        let mut guard = state.lock().await;
        guard.progress = progress.clone();
    }

    persist_items_snapshot(&state, &project_name, &items).await;
    let mut last_persist_at = Instant::now();
    let mut processed = 0;
    let mut errors = 0;
    let mut join_set: JoinSet<(usize, TranslationItem, bool)> = JoinSet::new();
    let mut next_index = 0usize;
    let mut in_flight = 0usize;

    while in_flight < concurrent_limit && next_index < total {
        let item_index = next_index;
        let current_item = items[item_index].clone();
        let preview = current_item
            .source_text
            .chars()
            .take(50)
            .collect::<String>();
        progress.current_item = Some(preview);
        {
            let mut guard = state.lock().await;
            guard.progress = progress.clone();
        }

        join_set.spawn(translate_item_with_key_pool(
            runtime.clone(),
            item_index,
            current_item,
        ));
        next_index += 1;
        in_flight += 1;
    }

    while in_flight > 0 {
        let Some(join_result) = join_set.join_next().await else {
            break;
        };
        in_flight = in_flight.saturating_sub(1);
        let (item_index, translated_item, success) = match join_result {
            Ok(value) => value,
            Err(error) => return Err(anyhow!("并发任务执行失败: {}", error)),
        };

        items[item_index] = translated_item;
        if success {
            processed += 1;
        } else {
            errors += 1;
        }

        progress.processed = processed;
        progress.error = errors;
        {
            let mut guard = state.lock().await;
            guard.progress = progress.clone();
        }

        if last_persist_at.elapsed() >= SNAPSHOT_PERSIST_INTERVAL {
            persist_items_snapshot(&state, &project_name, &items).await;
            last_persist_at = Instant::now();
        }

        if cancel.is_cancelled() {
            break;
        }

        let has_available_key = {
            let key_guard = key_pool.lock().await;
            key_guard.has_available()
        };
        if !has_available_key {
            tracing::error!("API key 全部不可用，提前结束任务");
            break;
        }

        if next_index < total {
            let item_index = next_index;
            let current_item = items[item_index].clone();
            let preview = current_item
                .source_text
                .chars()
                .take(50)
                .collect::<String>();
            progress.current_item = Some(preview);
            {
                let mut guard = state.lock().await;
                guard.progress = progress.clone();
            }

            join_set.spawn(translate_item_with_key_pool(
                runtime.clone(),
                item_index,
                current_item,
            ));
            next_index += 1;
            in_flight += 1;
        }
    }

    persist_items_snapshot(&state, &project_name, &items).await;

    let final_progress = TranslationProgress {
        total,
        processed,
        error: errors,
        is_running: false,
        current_item: None,
    };
    {
        let mut guard = state.lock().await;
        guard.progress = final_progress.clone();
    }

    if let Some(project_name) = project_name
        && let Ok(pool) = get_app_db_pool().await
    {
        let next_status = if final_progress.total > 0
            && final_progress.processed + final_progress.error >= final_progress.total
        {
            ProjectRunStatus::Completed
        } else if cancel.is_cancelled() {
            ProjectRunStatus::Paused
        } else {
            ProjectRunStatus::NotStarted
        };
        if let Err(error) = Project::update_status(pool, &project_name, next_status)
            .await
        {
            tracing::warn!("更新项目运行状态失败: {}", error);
        }
    }

    Ok(final_progress)
}

struct TranslationRuntime {
    provider_worker: Addr<ProviderWorker>,
    key_pool: Arc<Mutex<ApiKeyPool>>,
    provider: Provider,
    prompt: Option<String>,
    source_lang: String,
    target_lang: String,
    cancel: CancellationToken,
}

async fn translate_item_with_key_pool(
    runtime: Arc<TranslationRuntime>,
    item_index: usize,
    item: TranslationItem,
) -> (usize, TranslationItem, bool) {
    let mut working_item = item;
    let mut last_error: Option<String> = None;

    loop {
        if runtime.cancel.is_cancelled() {
            working_item.status = ItemStatus::Error;
            working_item.error_message = Some("任务已取消".to_string());
            return (item_index, working_item, false);
        }

        let next_key = {
            let mut guard = runtime.key_pool.lock().await;
            guard
                .next_index()
                .map(|key_idx| (key_idx, guard.key_at(key_idx).to_string()))
        };
        let Some((key_idx, key)) = next_key else {
            working_item.status = ItemStatus::Error;
            working_item.error_message =
                Some(last_error.unwrap_or_else(|| AppError::NoAvailableApiKey.to_string()));
            return (item_index, working_item, false);
        };

        let started_at = Instant::now();
        let outcome = runtime
            .provider_worker
            .ask(|resp| ProviderMessage::TranslateItem {
                item: working_item.clone(),
                provider: runtime.provider.clone(),
                prompt: runtime.prompt.clone(),
                api_key: key.clone(),
                source_language: runtime.source_lang.clone(),
                target_language: runtime.target_lang.clone(),
                resp,
            })
            .await;

        match outcome {
            Ok(Ok(translated_item)) => {
                {
                    let mut guard = runtime.key_pool.lock().await;
                    guard.report_success(key_idx, started_at.elapsed().as_millis() as f64);
                }
                return (item_index, translated_item, true);
            }
            Ok(Err(error)) => {
                {
                    let mut guard = runtime.key_pool.lock().await;
                    guard.report_failure(key_idx);
                }
                last_error = Some(error.to_string());
                working_item.status = ItemStatus::Error;
                working_item.error_message = last_error.clone();
            }
            Err(error) => {
                {
                    let mut guard = runtime.key_pool.lock().await;
                    guard.report_failure(key_idx);
                }
                last_error = Some(error.to_string());
                working_item.status = ItemStatus::Error;
                working_item.error_message = last_error.clone();
            }
        }
    }
}

async fn persist_items_snapshot(
    state: &Arc<Mutex<ProjectWorkerStatus>>,
    project_name: &Option<String>,
    items: &[TranslationItem],
) {
    {
        let mut guard = state.lock().await;
        guard.items = items.to_vec();
    }

    if let Some(project_name) = project_name {
        match get_app_db_pool().await {
            Ok(pool) => {
                if let Err(error) = Project::upsert_items(pool, project_name, items).await {
                    tracing::warn!("同步翻译条目快照到数据库失败: {}", error);
                }
            }
            Err(error) => {
                tracing::warn!("打开应用数据库失败，无法持久化翻译进度: {}", error);
            }
        }
    }
}
