#[derive(Debug, thiserror::Error)]
pub enum TgfeedAiError {
    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),

    #[error("API error: {0}")]
    Api(String),
}

pub type TgfeedAiResult<T> = Result<T, TgfeedAiError>;
