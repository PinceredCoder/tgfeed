pub(crate) const TELEGRAM_MAX_LENGTH: usize = 4096;

pub fn format_message(
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

pub fn split_telegram_message(text: String) -> Vec<String> {
    let utf16 = text.encode_utf16().collect::<Vec<_>>();
    let total_len = utf16.len();

    if total_len <= TELEGRAM_MAX_LENGTH {
        return vec![text.to_string()];
    }

    let mut parts = Vec::new();
    let mut start = 0;

    while start < total_len {
        let mut end = (start + TELEGRAM_MAX_LENGTH).min(total_len);

        if end < total_len {
            // Look for a newline to split at
            let search_start = start + TELEGRAM_MAX_LENGTH / 2;
            if let Some(newline_pos) = utf16[search_start..end]
                .iter()
                .rposition(|&c| c == b'\n' as u16)
            {
                end = search_start + newline_pos + 1;
            }
        }

        let part_utf16 = &utf16[start..end];
        parts.push(String::from_utf16_lossy(part_utf16));
        start = end;
    }

    parts
}
