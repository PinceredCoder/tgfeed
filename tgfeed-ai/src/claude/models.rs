#[derive(serde::Serialize)]
pub struct ClaudeRequest {
    pub model: String,
    pub max_tokens: u32,
    pub messages: Vec<ClaudeMessage>,
}

#[derive(serde::Serialize)]
pub struct ClaudeMessage {
    pub role: String,
    pub content: String,
}

#[derive(serde::Deserialize)]
pub struct ClaudeResponse {
    pub content: Vec<Content>,
    #[serde(default)]
    pub error: Option<ApiError>,
}

#[derive(serde::Deserialize)]
pub struct Content {
    pub text: String,
}

#[derive(serde::Deserialize)]
pub struct ApiError {
    pub message: String,
}
