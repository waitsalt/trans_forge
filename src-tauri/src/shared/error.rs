use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{field} 不能为空")]
    RequiredField { field: &'static str },

    #[error("未找到{entity}: {name}")]
    NotFound { entity: &'static str, name: String },

    #[error("{entity}已存在: {name}")]
    AlreadyExists { entity: &'static str, name: String },

    #[error("不支持的语言：{0}")]
    UnsupportedLanguage(String),

    #[error("{0}")]
    Validation(String),

    #[error("翻译任务正在运行")]
    TranslationTaskRunning,

    #[error("翻译已停止")]
    TranslationStopped,

    #[error("没有可用的 API key")]
    NoAvailableApiKey,

    #[error("没有可翻译的条目")]
    NoTranslatableItems,

    #[error("无效的 API 格式")]
    InvalidApiFormat,

    #[error("响应格式无效")]
    InvalidResponseFormat,

    #[error("无效的项目文件名")]
    InvalidProjectFileName,

    #[error("没有可写入的翻译条目")]
    NoWritableTranslationItems,

    #[error("请求失败：{0}")]
    RequestFailed(String),

    #[error("API 错误：{0}")]
    ApiError(String),

    #[error("解析响应失败：{0}")]
    ResponseParseFailed(String),

    #[error("获取模型列表失败：{0}")]
    ModelListFetchFailed(String),

    #[error("解析模型列表失败：{0}")]
    ModelListParseFailed(String),
}
