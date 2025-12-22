use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;
use teloxide::utils::command::BotCommands;
use tgfeed_common::command::MonitorCommand;
use tgfeed_common::event::BotEvent;
use tokio::sync::{mpsc, oneshot};

use crate::TgFeedBot;
use crate::command::Command;
use crate::utils::{format_message, split_telegram_message};

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

    if let Err(error) = this.rate_limiters.commands.check_key(&user_id) {
        tracing::warn!(%user_id, %error, "rate limit reached");
        bot.send_message(msg.chat.id, "⏳ Please wait a moment")
            .await?;

        return Ok(());
    }

    // TODO: extract handlers
    if let Some(text) = msg.text() {
        let response = match BotCommands::parse(text, me.username()) {
            Ok(cmd) => match cmd {
                Command::Start => "👋 Hello! This is a Telegram channels aggregator. Run /help to see the available commands.".to_string(),

                Command::Help => Command::descriptions().to_string(),

                Command::Subscribe(channel_handle) => {
                    let channel_handle = channel_handle.trim().trim_start_matches('@').to_string();
                    if channel_handle.is_empty() {
                        "Usage: /subscribe @channelname".to_string()
                    } else {
                        let (tx, rx) = oneshot::channel();
                        let _ = this
                            .monitor_tx
                            .send(MonitorCommand::Subscribe {
                                user_id,
                                channel_handle: channel_handle.clone(),
                                response: tx,
                            })
                            .await;

                        match rx.await {
                            Ok(Ok(_)) => format!("✅ Subscribed to @{}", channel_handle),
                            Ok(Err(e)) => format!("❌ Failed to subscribe: {e}"),
                            Err(_) => "❌ Internal error: monitor not responding".to_string(),
                        }
                    }
                }

                Command::Unsubscribe(channel_handle) => {
                    let channel_handle = channel_handle.trim().trim_start_matches('@').to_string();
                    if channel_handle.is_empty() {
                        "Usage: /unsubscribe @channelname".to_string()
                    } else {
                        let (tx, rx) = oneshot::channel();
                        let _ = this
                            .monitor_tx
                            .send(MonitorCommand::Unsubscribe {
                                user_id,
                                channel_handle: channel_handle.clone(),
                                response: tx,
                            })
                            .await;

                        match rx.await {
                            Ok(Ok(())) => format!("✅ Unsubscribed from @{}", channel_handle),
                            Ok(Err(e)) => format!("❌ Failed to unsubscribe: {e}"),
                            Err(_) => "❌ Internal error: monitor not responding".to_string(),
                        }
                    }
                }

                Command::List => {
                    let (tx, rx) = oneshot::channel();
                    let _ = this
                        .monitor_tx
                        .send(MonitorCommand::ListSubscriptions {
                            user_id,
                            response: tx,
                        })
                        .await;

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
                        Err(_) => "❌ Internal error: monitor not responding".to_string(),
                    }
                }

                Command::Summarize => {
                    if let Err(error) = this.rate_limiters.summarize.check_key(&user_id) {
                        tracing::warn!(%user_id, %error, "/summarize rate limit reached");
                        "⏳ /summarize is limited to once per hour".to_string()
                    } else {
                        // TODO: edit message instead
                        let _message = bot
                            .send_message(msg.chat.id, "⏳ Generating summary...")
                            .await?;

                        let (tx, rx) = oneshot::channel();
                        let _ = this
                            .monitor_tx
                            .send(MonitorCommand::Summarize {
                                user_id,
                                response: tx,
                            })
                            .await;

                        match rx.await {
                            Ok(Ok(summary)) => {
                                let parts = split_telegram_message(summary);

                                for part in parts {
                                    bot.send_message(msg.chat.id, part)
                                        .parse_mode(teloxide::types::ParseMode::Html)
                                        .await?;
                                }

                                return Ok(())
                            },
                            Ok(Err(e)) => format!("❌ Failed summarizing: {e}"),
                            Err(_) => "❌ Internal error: monitor not responding".to_string(),
                        }
                    }
                }
            },
            Err(_) => "❌ Unknown command".to_string(),
        };

        bot.send_message(msg.chat.id, response)
            .parse_mode(teloxide::types::ParseMode::Html)
            .await?;
    }

    Ok(())
}

pub(crate) async fn handle_events(
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
