use teloxide::prelude::Requester;
use teloxide::utils::command::BotCommands;
use tgfeed_common::command::MonitorCommand;
use tgfeed_common::event::BotEvent;
use tokio::sync::{mpsc, oneshot};

use crate::command::Command;

pub async fn handle_command(
    bot: teloxide::prelude::Bot,
    msg: teloxide::prelude::Message,
    me: teloxide::types::Me,
    monitor_tx: mpsc::Sender<MonitorCommand>,
) -> teloxide::prelude::ResponseResult<()> {
    let user_id = match msg.from.as_ref() {
        Some(user) => user.id.0 as i64,
        None => return Ok(()),
    };

    if let Some(text) = msg.text() {
        let response = match BotCommands::parse(text, me.username()) {
            Ok(cmd) => match cmd {
                Command::Help | Command::Start => Command::descriptions().to_string(),

                Command::Subscribe(channel_handle) => {
                    let channel_handle = channel_handle.trim().trim_start_matches('@').to_string();
                    if channel_handle.is_empty() {
                        "Usage: /subscribe @channelname".to_string()
                    } else {
                        let (tx, rx) = oneshot::channel();
                        let _ = monitor_tx
                            .send(MonitorCommand::Subscribe {
                                user_id,
                                channel_handle: channel_handle.clone(),
                                response: tx,
                            })
                            .await;

                        match rx.await {
                            Ok(Ok(_)) => format!("✅ Subscribed to @{}", channel_handle),
                            Ok(Err(e)) => format!("❌ Failed to subscribe: {}", e),
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
                        let _ = monitor_tx
                            .send(MonitorCommand::Unsubscribe {
                                user_id,
                                channel_handle: channel_handle.clone(),
                                response: tx,
                            })
                            .await;

                        match rx.await {
                            Ok(Ok(())) => format!("✅ Unsubscribed from @{}", channel_handle),
                            Ok(Err(e)) => format!("❌ Failed to unsubscribe: {}", e),
                            Err(_) => "❌ Internal error: monitor not responding".to_string(),
                        }
                    }
                }

                Command::List => {
                    let (tx, rx) = oneshot::channel();
                    let _ = monitor_tx
                        .send(MonitorCommand::ListSubscriptions {
                            user_id,
                            response: tx,
                        })
                        .await;

                    match rx.await {
                        Ok(Ok(subs)) if subs.is_empty() => "No active subscriptions".to_string(),
                        Ok(Ok(subs)) => {
                            let mut response = "📋 Active subscriptions:\n".to_string();
                            for sub in subs {
                                response.push_str(&format!("• @{}\n", sub));
                            }
                            response
                        }
                        Ok(Err(e)) => format!("❌ Failed to list subscriptions: {}", e),
                        Err(_) => "❌ Internal error: monitor not responding".to_string(),
                    }
                }

                Command::Summarize => {
                    // TODO: Implement summarize
                    "📰 Summarize not yet implemented".to_string()
                }
            },
            Err(_) => "❌ Unknown command".to_string(),
        };

        bot.send_message(msg.chat.id, response).await?;
    }

    Ok(())
}

pub(crate) async fn handle_events(
    bot: teloxide::prelude::Bot,
    repo: tgfeed_repo::Repo,
    mut event_rx: mpsc::Receiver<BotEvent>,
) {
    let retrier = retrier::RetryPolicy::exponential(tokio::time::Duration::from_secs(1));

    while let Some(event) = event_rx.recv().await {
        match event {
            BotEvent::NewMessage {
                channel_handle,
                text,
                message_id,
                ..
            } => {
                let subscribers = match repo.get_channel_subscribers(&channel_handle).await {
                    Ok(subs) => subs,
                    Err(error) => {
                        tracing::error!(%error, "Failed to get subscribers");
                        continue;
                    }
                };

                let source_link = format!("https://t.me/{}/{}", channel_handle, message_id);

                // TODO: better
                let formatted = format!("📢 @{channel_handle}\n\n{text}\n\nSource: {source_link}",);

                for user_id in subscribers {
                    tracing::info!(
                        %user_id,
                        "sending message to user"
                    );

                    if let Err(error) = retrier
                        .retry(|| {
                            let send_msg_fut =
                                bot.send_message(teloxide::types::ChatId(user_id), &formatted);

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
                }

                // TODO: make map for each user
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    }
}
