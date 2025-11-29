use teloxide::prelude::Requester;
use teloxide::utils::command::BotCommands;
use tgfeed_common::command::MonitorCommand;
use tokio::sync::{mpsc, oneshot};

use crate::command::Command;

pub async fn handle_command(
    bot: teloxide::prelude::Bot,
    msg: teloxide::prelude::Message,
    me: teloxide::types::Me,
    monitor_tx: mpsc::Sender<MonitorCommand>,
) -> teloxide::prelude::ResponseResult<()> {
    if let Some(text) = msg.text() {
        let response = match teloxide::utils::command::BotCommands::parse(text, me.username()) {
            Ok(cmd) => {
                match cmd {
                    Command::Help | Command::Start => Command::descriptions().to_string(),

                    Command::Subscribe(channel_handle) => {
                        let channel_handle =
                            channel_handle.trim().trim_start_matches('@').to_string();
                        if channel_handle.is_empty() {
                            "Usage: /subscribe @channelname".to_string()
                        } else {
                            let (tx, rx) = oneshot::channel();
                            let _ = monitor_tx
                                .send(MonitorCommand::Subscribe {
                                    channel_handle: channel_handle.clone(),
                                    response: tx,
                                })
                                .await;

                            match rx.await {
                                Ok(Ok(())) => format!("✅ Subscribed to @{}", channel_handle),
                                Ok(Err(e)) => format!("❌ Failed to subscribe: {}", e),
                                Err(_) => "❌ Internal error: monitor not responding".to_string(),
                            }
                        }
                    }

                    Command::Unsubscribe(channel_handle) => {
                        let channel_handle =
                            channel_handle.trim().trim_start_matches('@').to_string();
                        if channel_handle.is_empty() {
                            "Usage: /unsubscribe @channelname".to_string()
                        } else {
                            let (tx, rx) = oneshot::channel();
                            let _ = monitor_tx
                                .send(MonitorCommand::Unsubscribe {
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
                            .send(MonitorCommand::ListSubscriptions { response: tx })
                            .await;

                        match rx.await {
                            Ok(subs) if subs.is_empty() => "No active subscriptions".to_string(),
                            Ok(subs) => {
                                let mut response = "📋 Active subscriptions:\n".to_string();
                                for sub in subs {
                                    response.push_str(&format!("• @{}\n", sub));
                                }
                                response
                            }
                            Err(_) => "❌ Internal error: monitor not responding".to_string(),
                        }
                    }

                    Command::Summarize => {
                        // TODO: Implement summarize via repo
                        "📰 Summarize not yet implemented".to_string()
                    }
                }
            }
            Err(_) => "❌ Unknown command".to_string(),
        };

        bot.send_message(msg.chat.id, response).await?;
    }

    Ok(())
}
