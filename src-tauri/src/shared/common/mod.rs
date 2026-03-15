use serde::{Deserialize, Serialize};

use crate::shared::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: u32,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "UPPERCASE")]
pub enum Language {
    ZH,
    EN,
    #[default]
    JA,
    KO,
    RU,
    DE,
    FR,
    IT,
    ES,
    PT,
    AR,
    TH,
    VI,
    ID,
    MS,
    UK,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::ZH => write!(f, "Chinese"),
            Language::EN => write!(f, "English"),
            Language::JA => write!(f, "Japanese"),
            Language::KO => write!(f, "Korean"),
            Language::RU => write!(f, "Russian"),
            Language::DE => write!(f, "German"),
            Language::FR => write!(f, "French"),
            Language::IT => write!(f, "Italian"),
            Language::ES => write!(f, "Spanish"),
            Language::PT => write!(f, "Portuguese"),
            Language::AR => write!(f, "Arabic"),
            Language::TH => write!(f, "Thai"),
            Language::VI => write!(f, "Vietnamese"),
            Language::ID => write!(f, "Indonesian"),
            Language::MS => write!(f, "Malay"),
            Language::UK => write!(f, "Ukrainian"),
        }
    }
}

impl Language {
    pub fn parse(code: &str) -> AppResult<Self> {
        match code.to_uppercase().as_str() {
            "ZH" => Ok(Self::ZH),
            "EN" => Ok(Self::EN),
            "JA" => Ok(Self::JA),
            "KO" => Ok(Self::KO),
            "RU" => Ok(Self::RU),
            "DE" => Ok(Self::DE),
            "FR" => Ok(Self::FR),
            "IT" => Ok(Self::IT),
            "ES" => Ok(Self::ES),
            "PT" => Ok(Self::PT),
            "AR" => Ok(Self::AR),
            "TH" => Ok(Self::TH),
            "VI" => Ok(Self::VI),
            "ID" => Ok(Self::ID),
            "MS" => Ok(Self::MS),
            "UK" => Ok(Self::UK),
            _ => Err(AppError::UnsupportedLanguage(code.to_string())),
        }
    }

    pub fn parse_or_default(code: &str) -> Self {
        Self::parse(code).unwrap_or(Self::JA)
    }

    pub fn supported_items() -> Vec<(&'static str, &'static str)> {
        vec![
            ("ZH", "中文"),
            ("EN", "英语"),
            ("JA", "日语"),
            ("KO", "韩语"),
            ("RU", "俄语"),
            ("DE", "德语"),
            ("FR", "法语"),
            ("IT", "意大利语"),
            ("ES", "西班牙语"),
            ("PT", "葡萄牙语"),
            ("AR", "阿拉伯语"),
            ("TH", "泰语"),
            ("VI", "越南语"),
            ("ID", "印尼语"),
            ("MS", "马来语"),
            ("UK", "乌克兰语"),
        ]
    }
}
