mod config;

fn main() {
    tracing_subscriber::fmt::init();

    let config = config::Config::new();

    tracing::info!(
        server_addr = %config.server_addr,
        healthcheck_addr = %config.healthcheck_addr,
        "starting service"
    );
}
