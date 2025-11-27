mod config;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = config::Config::new();

    tracing::info!(
        server_addr = %config.server_addr,
        healthcheck_addr = %config.healthcheck_addr,
        "starting service"
    );

    let _repo = tgfeed_repo::Repo::new(&config.repo_config)
        .await
        .expect("failed to initialize repo");

    tracing::info!(
        database_name = %config.repo_config.database_name,
        "connected to database"
    );
}
