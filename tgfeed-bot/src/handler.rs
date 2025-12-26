use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;
use teloxide::utils::command::BotCommands;
use tgfeed_common::command::MonitorCommand;
use tgfeed_common::event::BotEvent;
use tokio::sync::{mpsc, oneshot};

use crate::command::Command;
use crate::utils::{format_message, split_telegram_message};
use crate::{TgFeedBot, response};

pub async fn handle_command(
    bot: teloxide::prelude::Bot,
    msg: teloxide::prelude::Message,
    me: teloxide::types::Me,
    this: TgFeedBot,
) -> teloxide::prelude::ResponseResult<()> {
    let user_id = match msg.from.as_ref() {
        Some(user) => user.id.0 as i64,
        None => return Ok(()),
    };

    let chat_id = msg.chat.id;

    if let Err(error) = this.rate_limiters.commands.check_key(&user_id) {
        tracing::warn!(%user_id, %error, "rate limit reached");
        bot.send_message(chat_id, "⏳ Please wait a moment").await?;

        return Ok(());
    }

    if let Some(text) = msg.text() {
        tracing::info!(
            %user_id,
            command = text,
            "new command"
        );

        let response = match BotCommands::parse(text, me.username()) {
            Ok(cmd) => match cmd {
                Command::Start => response::start(),
                Command::Help => response::help(),
                Command::Subscribe(channel_handle) => {
                    this.handle_subscribe(user_id, channel_handle).await
                }
                Command::Unsubscribe(channel_handle) => {
                    this.handle_unsubscribe(user_id, channel_handle).await
                }
                Command::List => this.handle_list(user_id).await,

                Command::Summarize => match this.handle_summarize(user_id, chat_id, &bot).await {
                    Ok(summary) => {
                        let parts = split_telegram_message(summary);

                        for part in parts {
                            bot.send_message(msg.chat.id, part)
                                .parse_mode(teloxide::types::ParseMode::Html)
                                .await?;
                        }

                        return Ok(());
                    }
                    Err(error_response) => error_response.to_string(),
                },
            },
            Err(_) => response::unknown_command(),
        };

        bot.send_message(msg.chat.id, response)
            .parse_mode(teloxide::types::ParseMode::Html)
            .await?;
    }

    Ok(())
}

pub(crate) async fn handle_monitor_events(
    bot: teloxide::prelude::Bot,
    mut event_rx: mpsc::Receiver<BotEvent>,
) {
    let retrier = retrier::RetryPolicy::exponential(tokio::time::Duration::from_secs(1));

    while let Some(event) = event_rx.recv().await {
        match event {
            BotEvent::NewMessage {
                channel_id,
                channel_handle,
                text,
                message_id,
                subscribers,
                entities,
            } => {
                let (full_text, fmt_entities) =
                    format_message(channel_id, channel_handle, message_id, text, entities);

                for user_id in subscribers {
                    tracing::info!(
                        %user_id,
                        "sending message to user"
                    );

                    if let Err(error) = retrier
                        .retry(|| {
                            let send_msg_fut = bot
                                .send_message(teloxide::types::ChatId(user_id), full_text.clone())
                                .disable_notification(true)
                                .protect_content(true)
                                .entities(fmt_entities.clone());

                            tokio::time::timeout(tokio::time::Duration::from_secs(30), send_msg_fut)
                        })
                        .await
                    {
                        tracing::error!(
                            %error,
                            user_id,
                            "Failed to send message to user"
                        );
                    } else {
                        tracing::info!(
                            %user_id,
                            "message sent"
                        );
                    }

                    // TODO: make map for each user
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
        }
    }
}

macro_rules! send_logging_error {
    ($self:ident, $monitor_command:expr) => {
        if let Err(error) = $self
            .monitor_tx
            .send($monitor_command)
            .await
        {
            tracing::error!(%error, "communication with monitor failed");
            return response::internal_server_error();
        };
    };

    (bail, $self:ident, $monitor_command:expr) => {
        if let Err(error) = $self
            .monitor_tx
            .send($monitor_command)
            .await
        {
            tracing::error!(%error, "communication with monitor failed");
            anyhow::bail!(response::internal_server_error());
        };
    };
}

impl TgFeedBot {
    async fn handle_subscribe(&self, user_id: i64, channel_handle: String) -> String {
        let channel_handle = channel_handle.trim().trim_start_matches('@').to_string();
        if channel_handle.is_empty() {
            response::usage()
        } else {
            let (tx, rx) = oneshot::channel();

            send_logging_error!(self, MonitorCommand::Subscribe {
                user_id,
                channel_handle: channel_handle.clone(),
                response: tx,
            });

            match rx.await {
                Ok(Ok(_)) => format!("✅ Subscribed to @{channel_handle}"),
                Ok(Err(e)) => format!("❌ Failed to subscribe: {e}"),
                Err(_) => response::internal_server_error(),
            }
        }
    }

    async fn handle_unsubscribe(&self, user_id: i64, channel_handle: String) -> String {
        let channel_handle = channel_handle.trim().trim_start_matches('@').to_string();
        if channel_handle.is_empty() {
            response::usage()
        } else {
            let (tx, rx) = oneshot::channel();

            send_logging_error!(self, MonitorCommand::Unsubscribe {
                user_id,
                channel_handle: channel_handle.clone(),
                response: tx,
            });

            match rx.await {
                Ok(Ok(())) => format!("✅ Unsubscribed from @{channel_handle}"),
                Ok(Err(e)) => format!("❌ Failed to unsubscribe: {e}"),
                Err(_) => response::internal_server_error(),
            }
        }
    }

    async fn handle_list(&self, user_id: i64) -> String {
        let (tx, rx) = oneshot::channel();

        send_logging_error!(self, MonitorCommand::ListSubscriptions {
            user_id,
            response: tx,
        });

        match rx.await {
            Ok(Ok(subs)) if subs.is_empty() => "No active subscriptions".to_string(),
            Ok(Ok(subs)) => {
                let mut response = format!("📋 Active subscriptions ({}):\n", subs.len());
                for sub in subs {
                    response.push_str(&format!("• @{sub}\n"));
                }
                response
            }
            Ok(Err(e)) => format!("❌ Failed to list subscriptions: {e}"),
            Err(_) => response::internal_server_error(),
        }
    }

    async fn handle_summarize(
        &self,
        user_id: i64,
        chat_id: teloxide::types::ChatId,
        bot: &teloxide::prelude::Bot,
    ) -> anyhow::Result<String> {
        if let Err(error) = self.rate_limiters.summarize.check_key(&user_id) {
            tracing::warn!(%user_id, %error, "/summarize rate limit reached");
            anyhow::bail!("⏳ /summarize is limited to once per hour")
        } else {
            // TODO: edit message instead
            let _message = bot
                .send_message(chat_id, "⏳ Generating summary...")
                .await?;

            let (tx, rx) = oneshot::channel();

            send_logging_error!(bail, self, MonitorCommand::Summarize {
                user_id,
                response: tx,
            });

            match rx.await {
                Ok(Ok(summary)) => Ok(summary),
                Ok(Err(error)) => anyhow::bail!("❌ Failed to summarize: {error}"),
                Err(_) => anyhow::bail!(response::internal_server_error()),
            }
        }
    }
}
