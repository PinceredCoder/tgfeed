pub enum BotEvent {
    NewMessage {
        channel_id: i64,
        channel_handle: String,
        message_id: i32,
        text: String,
        subscribers: Vec<i64>,
        entities: Vec<teloxide::types::MessageEntity>,
    },
}
