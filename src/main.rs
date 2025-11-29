use tgfeed_common::command::MonitorCommand;
use tokio::sync::mpsc;

mod config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config = config::Config::new();

    tracing::info!(
        server_addr = %config.server_addr,
        healthcheck_addr = %config.healthcheck_addr,
        "Starting tgfeed service"
    );

    let repo = tgfeed_repo::Repo::new(&config.repo_config)
        .await
        .expect("failed to initialize repo");

    tracing::info!(
        database_name = %config.repo_config.database_name,
        "Connected to database"
    );

    let (monitor_tx, monitor_rx) = mpsc::channel(100);

    let monitor = tgfeed_monitor::MonitorService::new(&config.monitor_config, repo, monitor_rx)?;
    monitor.authorize().await?;

    let bot = tgfeed_bot::TgFeedBot::new(&config.bot_config, monitor_tx.clone());

    tracing::info!("Starting bot and monitor...");

    let monitor_handle = tokio::spawn(monitor.run());
    let bot_handle = tokio::spawn(bot.run());

    tokio::signal::ctrl_c().await?;

    monitor_tx.send(MonitorCommand::Shutdown).await?;

    monitor_handle.await??;
    bot_handle.await??;

    Ok(())
}
