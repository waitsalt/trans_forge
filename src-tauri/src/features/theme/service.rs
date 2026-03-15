use anyhow::Result;

use super::model::{
    reset_to_default_themes, Theme, ThemeInput, ThemeKind, ThemePreferences, ThemePreferenceMode, ThemeState,
};
use crate::shared::error::AppError;

#[derive(Debug, Default)]
pub struct ThemeService;

impl ThemeService {
    pub fn new() -> Self {
        Self
    }

    pub async fn state(&self) -> Result<ThemeState> {
        let themes = Theme::list_all().await?;
        let preferences = ThemePreferences::load_or_default(&themes).await?;
        Ok(ThemeState { themes, preferences })
    }

    pub async fn create_theme(&self, input: ThemeInput) -> Result<ThemeState> {
        Theme::create(input, false).await?;
        self.state().await
    }

    pub async fn update_theme(&self, id: i64, input: ThemeInput) -> Result<ThemeState> {
        let existing = Theme::get_by_id(id).await?;
        if existing.mode != input.mode {
            self.ensure_mode_count_after_removal(existing.mode.clone()).await?;
        }

        Theme::update(id, input).await?;
        self.sync_preferences_after_update(Some(&existing)).await?;
        self.state().await
    }

    pub async fn delete_theme(&self, id: i64) -> Result<ThemeState> {
        let theme = Theme::get_by_id(id).await?;
        self.ensure_mode_count_after_removal(theme.mode.clone()).await?;
        Theme::delete(id).await?;
        self.sync_preferences_after_update(Some(&theme)).await?;
        self.state().await
    }

    pub async fn restore_defaults(&self) -> Result<ThemeState> {
        reset_to_default_themes().await?;
        ThemePreferences::default().save().await?;
        self.state().await
    }

    pub async fn save_preferences(&self, mut prefs: ThemePreferences) -> Result<ThemePreferences> {
        let themes = Theme::list_all().await?;
        prefs.mode = match prefs.mode {
            ThemePreferenceMode::Light => ThemePreferenceMode::Light,
            ThemePreferenceMode::System => ThemePreferenceMode::System,
            _ => ThemePreferenceMode::Dark,
        };

        if !theme_exists(&themes, &prefs.light_theme_key, ThemeKind::Light) {
            return Err(AppError::Validation("请选择有效的浅色主题".into()).into());
        }
        if !theme_exists(&themes, &prefs.dark_theme_key, ThemeKind::Dark) {
            return Err(AppError::Validation("请选择有效的深色主题".into()).into());
        }

        prefs.ensure_valid(&themes)?;
        prefs.save().await?;
        Ok(prefs)
    }

    async fn ensure_mode_count_after_removal(&self, mode: ThemeKind) -> Result<()> {
        let count = Theme::count_by_mode(mode.clone()).await?;
        if count <= 1 {
            return Err(AppError::Validation(format!(
                "{} 模式至少需要保留一个主题",
                mode.as_str()
            ))
            .into());
        }
        Ok(())
    }

    async fn sync_preferences_after_update(&self, previous: Option<&Theme>) -> Result<()> {
        if previous.is_none() {
            return Ok(());
        }
        let themes = Theme::list_all().await?;
        let mut prefs = ThemePreferences::load_raw_or_default().await?;
        let mut changed = false;

        if let Some(before) = previous {
            let updated = themes.iter().find(|theme| theme.id == before.id);
            if before.mode == ThemeKind::Light && prefs.light_theme_key == before.key {
                if let Some(after) = updated.filter(|theme| theme.mode == ThemeKind::Light) {
                    if prefs.light_theme_key != after.key {
                        prefs.light_theme_key = after.key.clone();
                        changed = true;
                    }
                } else if let Some(fallback) =
                    find_first_theme_key(&themes, ThemeKind::Light)
                        .filter(|fallback| prefs.light_theme_key != *fallback)
                {
                    prefs.light_theme_key = fallback;
                    changed = true;
                }
            }

            if before.mode == ThemeKind::Dark && prefs.dark_theme_key == before.key {
                if let Some(after) = updated.filter(|theme| theme.mode == ThemeKind::Dark) {
                    if prefs.dark_theme_key != after.key {
                        prefs.dark_theme_key = after.key.clone();
                        changed = true;
                    }
                } else if let Some(fallback) =
                    find_first_theme_key(&themes, ThemeKind::Dark)
                        .filter(|fallback| prefs.dark_theme_key != *fallback)
                {
                    prefs.dark_theme_key = fallback;
                    changed = true;
                }
            }
        }

        let ensured = prefs.ensure_valid(&themes)?;
        if changed || ensured {
            prefs.save().await?;
        }
        Ok(())
    }
}

fn theme_exists(themes: &[Theme], key: &str, mode: ThemeKind) -> bool {
    themes
        .iter()
        .any(|theme| theme.key == key && theme.mode == mode)
}

fn find_first_theme_key(themes: &[Theme], mode: ThemeKind) -> Option<String> {
    themes
        .iter()
        .filter(|theme| theme.mode == mode)
        .map(|theme| theme.key.clone())
        .next()
}
