use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;
use teloxide::utils::command::BotCommands;
use tgfeed_common::command::MonitorCommand;
use tgfeed_common::event::BotEvent;
use tokio::sync::{mpsc, oneshot};

use crate::TgFeedBot;
use crate::command::Command;

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
                            let mut response = "📋 Active subscriptions:\n".to_string();
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
                            Ok(Ok(summary)) => summary,
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

fn format_message(
    channel_id: i64,
    channel_handle: String,
    message_id: i32,
    text: String,
    entities: Vec<teloxide::types::MessageEntity>,
) -> (String, Vec<teloxide::types::MessageEntity>) {
    use teloxide::types::{MessageEntity, MessageEntityKind};

    let channel_part = format!("📢 @{channel_handle}");
    let separator = "──────────";
    let source_link = format!("https://t.me/c/{channel_id}/{message_id}");

    let full_text = format!("{channel_part}\n{separator}\n{text}\n{separator}\nSource",);

    // Calculate UTF-16 offsets
    let channel_prefix_len = "📢 @".encode_utf16().count(); // "📢 @" before handle
    let channel_handle_len = channel_handle.encode_utf16().count();

    let prefix_total = channel_part.encode_utf16().count()
    + 1  // \n
    + separator.encode_utf16().count()
    + 1; // \n

    let text_len = text.encode_utf16().count();

    let source_offset = prefix_total
    + text_len
    + 1  // \n
    + separator.encode_utf16().count()
    + 1; // \n

    // Build entities
    let mut fmt_entities = Vec::with_capacity(entities.len() + 2);

    // Bold for @channel_handle
    fmt_entities.push(MessageEntity::new(
        MessageEntityKind::Bold,
        channel_prefix_len,
        channel_handle_len,
    ));

    // Shift original text entities
    for e in &entities {
        fmt_entities.push(MessageEntity::new(
            e.kind.clone(),
            e.offset + prefix_total,
            e.length,
        ));
    }

    // TextLink for "Source"
    fmt_entities.push(MessageEntity::new(
        MessageEntityKind::TextLink {
            url: reqwest::Url::parse(&source_link).unwrap(),
        },
        source_offset,
        6, // "Source"
    ));

    (full_text, fmt_entities)
}
