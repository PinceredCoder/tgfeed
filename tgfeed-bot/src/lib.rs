mod command;
mod config;
mod handler;
mod rate_limit;

use std::sync::Arc;

pub use config::Config;
use teloxide::dispatching::UpdateFilterExt;
use teloxide::prelude::Requester;
use teloxide::utils::command::BotCommands;
use tgfeed_common::command::MonitorCommand;
use tgfeed_common::event::BotEvent;
use tokio::sync::mpsc;

use crate::command::Command;
use crate::rate_limit::RateLimiters;

#[derive(Clone)]
pub struct TgFeedBot {
    bot_token: String,
    monitor_tx: mpsc::Sender<MonitorCommand>,
    rate_limiters: Arc<RateLimiters>,
}

impl TgFeedBot {
    pub fn new(config: &Config, monitor_tx: mpsc::Sender<MonitorCommand>) -> Self {
        let rate_limiters = Arc::new(RateLimiters::new());

        Self {
            monitor_tx,
            rate_limiters,
            bot_token: config.token.clone(),
        }
    }

    pub async fn run(
        self,
        event_rx: mpsc::Receiver<BotEvent>,
    ) -> Result<(), teloxide::RequestError> {
        tracing::info!("Starting Telegram bot...");

        let bot = teloxide::prelude::Bot::new(&self.bot_token);

        bot.set_my_commands(Command::bot_commands()).await?;

        let handler = teloxide::prelude::Update::filter_message().endpoint(handler::handle_command);

        let event_handle = {
            let bot = bot.clone();
            tokio::spawn(async move {
                handler::handle_events(bot, event_rx).await;
            })
        };

        teloxide::prelude::Dispatcher::builder(bot, handler)
            .dependencies(teloxide::prelude::dptree::deps![self])
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;

        event_handle.await.expect("event handler loop failed");

        Ok(())
    }
}
