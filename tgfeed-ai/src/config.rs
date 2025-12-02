#[derive(serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Config {
    Claude(crate::claude::Config),
}
