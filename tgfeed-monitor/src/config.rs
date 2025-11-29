use std::path::PathBuf;

#[derive(serde::Deserialize)]
pub struct Config {
    pub api_id: i32,
    pub api_hash: String,
    pub session_file: PathBuf,
}
