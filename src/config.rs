use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(serde::Deserialize)]
pub(crate) struct Config {
    pub server_addr: SocketAddr,
    pub healthcheck_addr: SocketAddr,

    pub api_id: i32,
    pub api_hash: String,
    pub session_file: PathBuf,

    pub repo_config: tgfeed_repo::Config,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Config {
        let env_config = config::Environment::default()
            .separator("__")
            .list_separator(";")
            //.with_list_parse_key("auth_settings.allowed_origins")
            .try_parsing(true);

        let mut conf_builder = config::Config::builder().add_source(env_config);

        if std::path::Path::new("Settings.toml").exists() {
            conf_builder = conf_builder.add_source(config::File::with_name("./Settings.toml"));
        }

        conf_builder
            .build()
            .unwrap()
            .try_deserialize::<Config>()
            .unwrap_or_else(|e| panic!("Error parsing config: {e}"))
    }
}
