use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::features::kv::AppKv;
use crate::shared::database::get_app_db_pool;
use crate::shared::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ThemeKind {
    Light,
    Dark,
}

impl ThemeKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ThemeKind::Light => "light",
            ThemeKind::Dark => "dark",
        }
    }

    pub fn from_str(value: &str) -> Result<Self> {
        match value.trim().to_lowercase().as_str() {
            "light" => Ok(ThemeKind::Light),
            "dark" => Ok(ThemeKind::Dark),
            _ => Err(AppError::Validation("无效的主题模式".into()).into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreferenceMode {
    Light,
    #[default]
    Dark,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemePalette {
    pub rosewater: String,
    pub flamingo: String,
    pub pink: String,
    pub mauve: String,
    pub red: String,
    pub maroon: String,
    pub peach: String,
    pub yellow: String,
    pub green: String,
    pub teal: String,
    pub sky: String,
    pub sapphire: String,
    pub blue: String,
    pub lavender: String,
    pub text: String,
    pub subtext1: String,
    pub subtext0: String,
    pub overlay2: String,
    pub overlay1: String,
    pub overlay0: String,
    pub surface2: String,
    pub surface1: String,
    pub surface0: String,
    pub base: String,
    pub mantle: String,
    pub crust: String,
}

#[derive(Debug, Clone, Deserialize)]
struct LegacyThemePalette {
    bg: String,
    bg_elev: String,
    bg_soft: String,
    bg_muted: String,
    text: String,
    text_soft: String,
    text_dim: String,
    border: String,
    accent: String,
    ok: String,
    warn: String,
    danger: String,
}

impl From<LegacyThemePalette> for ThemePalette {
    fn from(old: LegacyThemePalette) -> Self {
        ThemePalette {
            rosewater: old.accent.clone(),
            flamingo: old.danger.clone(),
            pink: old.accent.clone(),
            mauve: old.accent.clone(),
            red: old.danger.clone(),
            maroon: old.danger.clone(),
            peach: old.warn.clone(),
            yellow: old.warn.clone(),
            green: old.ok.clone(),
            teal: old.ok.clone(),
            sky: old.accent.clone(),
            sapphire: old.accent.clone(),
            blue: old.accent.clone(),
            lavender: old.accent.clone(),
            text: old.text.clone(),
            subtext1: old.text_soft.clone(),
            subtext0: old.text_dim.clone(),
            overlay2: old.border.clone(),
            overlay1: old.text_dim.clone(),
            overlay0: old.text_soft.clone(),
            surface2: old.bg_muted.clone(),
            surface1: old.bg_elev.clone(),
            surface0: old.bg_soft.clone(),
            base: old.bg.clone(),
            mantle: old.bg_elev.clone(),
            crust: old.bg_muted.clone(),
        }
    }
}

impl ThemePalette {
    pub fn validate(&self) -> Result<()> {
        for (value, field) in [
            (&self.rosewater, "rosewater"),
            (&self.flamingo, "flamingo"),
            (&self.pink, "pink"),
            (&self.mauve, "mauve"),
            (&self.red, "red"),
            (&self.maroon, "maroon"),
            (&self.peach, "peach"),
            (&self.yellow, "yellow"),
            (&self.green, "green"),
            (&self.teal, "teal"),
            (&self.sky, "sky"),
            (&self.sapphire, "sapphire"),
            (&self.blue, "blue"),
            (&self.lavender, "lavender"),
            (&self.text, "text"),
            (&self.subtext1, "subtext1"),
            (&self.subtext0, "subtext0"),
            (&self.overlay2, "overlay2"),
            (&self.overlay1, "overlay1"),
            (&self.overlay0, "overlay0"),
            (&self.surface2, "surface2"),
            (&self.surface1, "surface1"),
            (&self.surface0, "surface0"),
            (&self.base, "base"),
            (&self.mantle, "mantle"),
            (&self.crust, "crust"),
        ] {
            ensure_color(value, field)?;
        }
        Ok(())
    }
}

fn ensure_color(value: &str, field: &'static str) -> Result<()> {
    let color = value.trim();
    let valid = color.starts_with('#') && (color.len() == 7 || color.len() == 4);
    if !valid {
        return Err(AppError::Validation(format!(
            "{field} 必须是 #RGB 或 #RRGGBB"
        ))
        .into());
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeInput {
    pub name: String,
    pub mode: ThemeKind,
    pub palette: ThemePalette,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub id: i64,
    pub key: String,
    pub name: String,
    pub mode: ThemeKind,
    pub palette: ThemePalette,
    pub is_builtin: bool,
}

impl Theme {
    pub(crate) async fn init_schema(pool: &SqlitePool) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS themes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                key TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                mode TEXT NOT NULL CHECK(mode IN ('light', 'dark')),
                palette TEXT NOT NULL,
                is_builtin INTEGER NOT NULL DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(pool)
        .await
        .context("初始化 themes 表失败")?;

        sqlx::query(
            r#"
            CREATE TRIGGER IF NOT EXISTS trg_themes_updated_at
            AFTER UPDATE ON themes
            FOR EACH ROW
            BEGIN
                UPDATE themes SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
            END
            "#,
        )
        .execute(pool)
        .await
        .context("初始化 themes 表触发器失败")?;

        ensure_default_themes(pool).await?;
        Ok(())
    }

    pub async fn list_all() -> Result<Vec<Self>> {
        let pool = get_app_db_pool().await?;
        let rows = sqlx::query(
            "SELECT id, key, name, mode, palette, is_builtin FROM themes ORDER BY mode DESC, id ASC",
        )
        .fetch_all(pool)
        .await
        .context("查询主题失败")?;

        rows.into_iter().map(parse_theme_row).collect()
    }

    pub async fn get_by_id(id: i64) -> Result<Self> {
        let pool = get_app_db_pool().await?;
        let row = sqlx::query("SELECT id, key, name, mode, palette, is_builtin FROM themes WHERE id = ?1")
            .bind(id)
            .fetch_optional(pool)
            .await
            .context("读取主题失败")?
            .ok_or(AppError::NotFound {
                entity: "Theme",
                name: id.to_string(),
            })?;
        parse_theme_row(row)
    }

    pub async fn count_by_mode(mode: ThemeKind) -> Result<i64> {
        let pool = get_app_db_pool().await?;
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(1) FROM themes WHERE mode = ?1")
            .bind(mode.as_str())
            .fetch_one(pool)
            .await
            .context("统计主题失败")?;
        Ok(count)
    }

    pub async fn create(input: ThemeInput, is_builtin: bool) -> Result<Self> {
        let pool = get_app_db_pool().await?;
        Self::create_with_pool(pool, input, is_builtin).await
    }

    pub(crate) async fn create_with_pool(pool: &SqlitePool, input: ThemeInput, is_builtin: bool) -> Result<Self> {
        let ThemeInput { name, mode, palette } = normalize_input(input)?;
        palette.validate()?;
        let key = Uuid::new_v4().to_string();
        let palette_json = serde_json::to_string(&palette)?;
        match sqlx::query(
            r#"
            INSERT INTO themes (key, name, mode, palette, is_builtin)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        )
        .bind(&key)
        .bind(&name)
        .bind(mode.as_str())
        .bind(&palette_json)
        .bind(is_builtin)
        .execute(pool)
        .await
        {
            Ok(_) => {}
            Err(error) => {
                if let Some(db_error) = error.as_database_error()
                    && db_error.message().contains("UNIQUE constraint failed: themes.key")
                {
                    return Err(AppError::AlreadyExists {
                        entity: "Theme",
                        name: key,
                    }
                    .into());
                }
                return Err(error).context("创建主题失败")?
            }
        }

        let row = sqlx::query("SELECT id, key, name, mode, palette, is_builtin FROM themes WHERE key = ?1")
            .bind(&key)
            .fetch_one(pool)
            .await
            .context("读取新建主题失败")?;
        parse_theme_row(row)
    }

    pub async fn update(id: i64, input: ThemeInput) -> Result<Self> {
        let ThemeInput { name, mode, palette } = normalize_input(input)?;
        palette.validate()?;
        let pool = get_app_db_pool().await?;
        let palette_json = serde_json::to_string(&palette)?;
        let affected = sqlx::query(
            r#"
            UPDATE themes
            SET name = ?1,
                mode = ?2,
                palette = ?3
            WHERE id = ?4
            "#,
        )
        .bind(&name)
        .bind(mode.as_str())
        .bind(&palette_json)
        .bind(id)
        .execute(pool)
        .await
        .context("更新主题失败")?;
        if affected.rows_affected() == 0 {
            return Err(AppError::NotFound {
                entity: "Theme",
                name: id.to_string(),
            }
            .into());
        }
        Self::get_by_id(id).await
    }

    pub async fn delete(id: i64) -> Result<Self> {
        let theme = Self::get_by_id(id).await?;
        let pool = get_app_db_pool().await?;
        sqlx::query("DELETE FROM themes WHERE id = ?1")
            .bind(id)
            .execute(pool)
            .await
            .context("删除主题失败")?;
        Ok(theme)
    }

    pub async fn delete_all() -> Result<()> {
        let pool = get_app_db_pool().await?;
        sqlx::query("DELETE FROM themes")
            .execute(pool)
            .await
            .context("清空主题失败")?;
        Ok(())
    }
}

fn normalize_input(mut input: ThemeInput) -> Result<ThemeInput> {
    input.name = input.name.trim().to_string();
    if input.name.is_empty() {
        return Err(AppError::RequiredField { field: "name" }.into());
    }
    Ok(input)
}

fn parse_theme_row(row: sqlx::sqlite::SqliteRow) -> Result<Theme> {
    let palette_json: String = row.try_get("palette").context("读取主题调色板失败")?;
    let palette = serde_json::from_str(&palette_json)
        .or_else(|_| convert_legacy_palette(&palette_json))
        .context("解析主题调色板失败")?;
    Ok(Theme {
        id: row.try_get("id").context("读取主题 id 失败")?,
        key: row.try_get("key").context("读取主题 key 失败")?,
        name: row.try_get("name").context("读取主题 name 失败")?,
        mode: ThemeKind::from_str(
            row.try_get::<String, _>("mode")
                .context("读取主题 mode 失败")?
                .as_str(),
        )?,
        palette,
        is_builtin: row
            .try_get::<i64, _>("is_builtin")
            .context("读取主题 is_builtin 失败")?
            != 0,
    })
}

fn convert_legacy_palette(json: &str) -> Result<ThemePalette> {
    let legacy: LegacyThemePalette = serde_json::from_str(json).context("解析旧版主题调色板失败")?;
    Ok(legacy.into())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemePreferences {
    pub mode: ThemePreferenceMode,
    pub light_theme_key: String,
    pub dark_theme_key: String,
}

impl ThemePreferences {
    pub fn default() -> Self {
        Self {
            mode: ThemePreferenceMode::Dark,
            light_theme_key: String::new(),
            dark_theme_key: String::new(),
        }
    }

    pub async fn load_or_default(themes: &[Theme]) -> Result<Self> {
        let mut prefs = Self::load_raw_or_default().await?;
        let changed = prefs.ensure_valid(themes)?;
        if changed {
            prefs.save().await?;
        }
        Ok(prefs)
    }

    pub async fn load_raw_or_default() -> Result<Self> {
        let entry = AppKv::get("theme_preferences").await?;
        let prefs = if let Some(entry) = entry {
            serde_json::from_str::<ThemePreferences>(&entry.value)
                .unwrap_or_else(|_| ThemePreferences::default())
        } else {
            ThemePreferences::default()
        };
        Ok(prefs)
    }

    pub async fn save(&self) -> Result<()> {
        let value = serde_json::to_string(self)?;
        AppKv {
            name: "theme_preferences".to_string(),
            value,
        }
        .set()
        .await
    }

    pub fn ensure_valid(&mut self, themes: &[Theme]) -> Result<bool> {
        let mut changed = false;
        self.mode = match self.mode {
            ThemePreferenceMode::Light => ThemePreferenceMode::Light,
            ThemePreferenceMode::System => ThemePreferenceMode::System,
            _ => ThemePreferenceMode::Dark,
        };

        if !has_theme(&self.light_theme_key, ThemeKind::Light, themes) {
            self.light_theme_key = first_theme_key(ThemeKind::Light, themes)?;
            changed = true;
        }
        if !has_theme(&self.dark_theme_key, ThemeKind::Dark, themes) {
            self.dark_theme_key = first_theme_key(ThemeKind::Dark, themes)?;
            changed = true;
        }
        Ok(changed)
    }
}

fn has_theme(key: &str, mode: ThemeKind, themes: &[Theme]) -> bool {
    themes
        .iter()
        .any(|theme| theme.key == key && theme.mode == mode)
}

fn first_theme_key(mode: ThemeKind, themes: &[Theme]) -> Result<String> {
    themes
        .iter()
        .find(|theme| theme.mode == mode)
        .map(|theme| theme.key.clone())
        .ok_or_else(|| {
            AppError::Validation(format!(
                "缺少 {} 主题，请至少保留一个",
                mode.as_str()
            ))
            .into()
        })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeState {
    pub themes: Vec<Theme>,
    pub preferences: ThemePreferences,
}

async fn ensure_default_themes(pool: &SqlitePool) -> Result<()> {
    for input in default_themes() {
        let exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(1) FROM themes WHERE name = ?1")
            .bind(&input.name)
            .fetch_one(pool)
            .await
            .context("检查默认主题失败")?;
        if exists == 0 {
            Theme::create_with_pool(pool, input, true).await?;
        }
    }
    Ok(())
}

fn default_themes() -> Vec<ThemeInput> {
    vec![
        ThemeInput {
            name: "Catppuccin Latte".to_string(),
            mode: ThemeKind::Light,
            palette: ThemePalette {
                rosewater: "#dc8a78".to_string(),
                flamingo: "#dd7878".to_string(),
                pink: "#ea76cb".to_string(),
                mauve: "#8839ef".to_string(),
                red: "#d20f39".to_string(),
                maroon: "#e64553".to_string(),
                peach: "#fe640b".to_string(),
                yellow: "#df8e1d".to_string(),
                green: "#40a02b".to_string(),
                teal: "#179299".to_string(),
                sky: "#04a5e5".to_string(),
                sapphire: "#209fb5".to_string(),
                blue: "#1e66f5".to_string(),
                lavender: "#7287fd".to_string(),
                text: "#4c4f69".to_string(),
                subtext1: "#5c5f77".to_string(),
                subtext0: "#6c6f85".to_string(),
                overlay2: "#7c7f93".to_string(),
                overlay1: "#8c8fa1".to_string(),
                overlay0: "#9ca0b0".to_string(),
                surface2: "#acb0be".to_string(),
                surface1: "#bcc0cc".to_string(),
                surface0: "#ccd0da".to_string(),
                base: "#eff1f5".to_string(),
                mantle: "#e6e9ef".to_string(),
                crust: "#dce0e8".to_string(),
            },
        },
        ThemeInput {
            name: "Catppuccin Frappe".to_string(),
            mode: ThemeKind::Dark,
            palette: ThemePalette {
                rosewater: "#f2d5cf".to_string(),
                flamingo: "#eebebe".to_string(),
                pink: "#f4b8e4".to_string(),
                mauve: "#ca9ee6".to_string(),
                red: "#e78284".to_string(),
                maroon: "#ea999c".to_string(),
                peach: "#ef9f76".to_string(),
                yellow: "#e5c890".to_string(),
                green: "#a6d189".to_string(),
                teal: "#81c8be".to_string(),
                sky: "#99d1db".to_string(),
                sapphire: "#85c1dc".to_string(),
                blue: "#8caaee".to_string(),
                lavender: "#babbf1".to_string(),
                text: "#c6d0f5".to_string(),
                subtext1: "#b5bfe2".to_string(),
                subtext0: "#a5adce".to_string(),
                overlay2: "#949cbb".to_string(),
                overlay1: "#838ba7".to_string(),
                overlay0: "#737994".to_string(),
                surface2: "#626880".to_string(),
                surface1: "#51576d".to_string(),
                surface0: "#414559".to_string(),
                base: "#303446".to_string(),
                mantle: "#292c3c".to_string(),
                crust: "#232634".to_string(),
            },
        },
        ThemeInput {
            name: "Catppuccin Macchiato".to_string(),
            mode: ThemeKind::Dark,
            palette: ThemePalette {
                rosewater: "#f4dbd6".to_string(),
                flamingo: "#f0c6c6".to_string(),
                pink: "#f5bde6".to_string(),
                mauve: "#c6a0f6".to_string(),
                red: "#ed8796".to_string(),
                maroon: "#ee99a0".to_string(),
                peach: "#f5a97f".to_string(),
                yellow: "#eed49f".to_string(),
                green: "#a6da95".to_string(),
                teal: "#8bd5ca".to_string(),
                sky: "#91d7e3".to_string(),
                sapphire: "#7dc4e4".to_string(),
                blue: "#8aadf4".to_string(),
                lavender: "#b7bdf8".to_string(),
                text: "#cad3f5".to_string(),
                subtext1: "#b8c0e0".to_string(),
                subtext0: "#a5adcb".to_string(),
                overlay2: "#939ab7".to_string(),
                overlay1: "#8087a2".to_string(),
                overlay0: "#6e738d".to_string(),
                surface2: "#5b6078".to_string(),
                surface1: "#494d64".to_string(),
                surface0: "#363a4f".to_string(),
                base: "#24273a".to_string(),
                mantle: "#1e2030".to_string(),
                crust: "#181926".to_string(),
            },
        },
        ThemeInput {
            name: "Catppuccin Mocha".to_string(),
            mode: ThemeKind::Dark,
            palette: ThemePalette {
                rosewater: "#f5e0dc".to_string(),
                flamingo: "#f2cdcd".to_string(),
                pink: "#f5c2e7".to_string(),
                mauve: "#cba6f7".to_string(),
                red: "#f38ba8".to_string(),
                maroon: "#eba0ac".to_string(),
                peach: "#fab387".to_string(),
                yellow: "#f9e2af".to_string(),
                green: "#a6e3a1".to_string(),
                teal: "#94e2d5".to_string(),
                sky: "#89dceb".to_string(),
                sapphire: "#74c7ec".to_string(),
                blue: "#89b4fa".to_string(),
                lavender: "#b4befe".to_string(),
                text: "#cdd6f4".to_string(),
                subtext1: "#bac2de".to_string(),
                subtext0: "#a6adc8".to_string(),
                overlay2: "#9399b2".to_string(),
                overlay1: "#7f849c".to_string(),
                overlay0: "#6c7086".to_string(),
                surface2: "#585b70".to_string(),
                surface1: "#45475a".to_string(),
                surface0: "#313244".to_string(),
                base: "#1e1e2e".to_string(),
                mantle: "#181825".to_string(),
                crust: "#11111b".to_string(),
            },
        },
    ]
}

pub async fn reset_to_default_themes() -> Result<Vec<Theme>> {
    Theme::delete_all().await?;
    let pool = get_app_db_pool().await?;
    for input in default_themes() {
        Theme::create_with_pool(pool, input, true).await?;
    }
    Theme::list_all().await
}
