#[derive(serde::Deserialize)]
pub struct Config {
    pub connection_string: String,
    pub database_name: String,
}
