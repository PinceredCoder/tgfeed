pub enum BotEvent {
    NewMessage {
        channel_id: i64,
        // TODO: use channel_id everywhere
        channel_handle: String,
        message_id: i32,
        text: String,
    },
}
