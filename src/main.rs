mod config;
mod service;
mod utils;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = config::Config::new();

    tracing::info!(
        server_addr = %config.server_addr,
        healthcheck_addr = %config.healthcheck_addr,
        "starting service"
    );

    let repo = tgfeed_repo::Repo::new(&config.repo_config)
        .await
        .expect("failed to initialize repo");

    tracing::info!(
        database_name = %config.repo_config.database_name,
        "connected to database"
    );

    let tgfeed =
        service::TgFeedService::new(&config, repo).expect("failed to initialize TGFeed service");

    tgfeed.authorize().await.expect("authorization failed");

    if let Err(e) = tgfeed.run().await {
        tracing::error!(%e, "service failed");
    }
}
