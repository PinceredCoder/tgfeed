mod command;
mod config;
mod handler;

pub use config::Config;
use teloxide::dispatching::UpdateFilterExt;
use tgfeed_common::command::MonitorCommand;
use tokio::sync::mpsc;

pub struct TgFeedBot {
    bot: teloxide::prelude::Bot,
    monitor_tx: mpsc::Sender<MonitorCommand>,
}

impl TgFeedBot {
    pub fn new(config: &Config, monitor_tx: mpsc::Sender<MonitorCommand>) -> Self {
        let bot = teloxide::prelude::Bot::new(&config.token);
        Self { bot, monitor_tx }
    }

    pub async fn run(self) {
        tracing::info!("Starting Telegram bot...");

        let handler = teloxide::prelude::Update::filter_message().endpoint(handler::handle_command);

        teloxide::prelude::Dispatcher::builder(self.bot, handler)
            .dependencies(teloxide::prelude::dptree::deps![self.monitor_tx])
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await
    }
}
