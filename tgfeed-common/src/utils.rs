use grammers_tl_types as tl;
use teloxide::types::{MessageEntity, MessageEntityKind};

/// Convert grammers MessageEntity to teloxide MessageEntity
pub fn convert_entities(entities: Option<&Vec<tl::enums::MessageEntity>>) -> Vec<MessageEntity> {
    let Some(entities) = entities else {
        return Vec::new();
    };

    entities
        .iter()
        .filter_map(|entity| {
            let (offset, length, kind) = match entity {
                tl::enums::MessageEntity::Bold(e) => (
                    e.offset as usize,
                    e.length as usize,
                    MessageEntityKind::Bold,
                ),
                tl::enums::MessageEntity::Italic(e) => (
                    e.offset as usize,
                    e.length as usize,
                    MessageEntityKind::Italic,
                ),
                tl::enums::MessageEntity::Code(e) => (
                    e.offset as usize,
                    e.length as usize,
                    MessageEntityKind::Code,
                ),
                tl::enums::MessageEntity::Pre(e) => (
                    e.offset as usize,
                    e.length as usize,
                    MessageEntityKind::Pre {
                        language: Some(e.language.clone()).filter(|s| !s.is_empty()),
                    },
                ),
                tl::enums::MessageEntity::Underline(e) => (
                    e.offset as usize,
                    e.length as usize,
                    MessageEntityKind::Underline,
                ),
                tl::enums::MessageEntity::Strike(e) => (
                    e.offset as usize,
                    e.length as usize,
                    MessageEntityKind::Strikethrough,
                ),
                tl::enums::MessageEntity::TextUrl(e) => (
                    e.offset as usize,
                    e.length as usize,
                    MessageEntityKind::TextLink {
                        url: reqwest::Url::parse(&e.url).ok()?,
                    },
                ),
                tl::enums::MessageEntity::Spoiler(e) => (
                    e.offset as usize,
                    e.length as usize,
                    MessageEntityKind::Spoiler,
                ),
                tl::enums::MessageEntity::Mention(e) => (
                    e.offset as usize,
                    e.length as usize,
                    MessageEntityKind::Mention,
                ),
                tl::enums::MessageEntity::Hashtag(e) => (
                    e.offset as usize,
                    e.length as usize,
                    MessageEntityKind::Hashtag,
                ),
                tl::enums::MessageEntity::Url(e) => {
                    (e.offset as usize, e.length as usize, MessageEntityKind::Url)
                }
                // Skip other types we don't need
                _ => return None,
            };

            Some(MessageEntity::new(kind, offset, length))
        })
        .collect()
}
