use teloxide::types::{MessageEntity, MessageEntityKind};

use crate::utils::{TELEGRAM_MAX_LENGTH, format_message, split_telegram_message};

#[test]
fn test_format_message_basic() {
    let (text, entities) = format_message(
        123456,
        "testchannel".to_string(),
        1,
        "Hello world".to_string(),
        vec![],
    );

    assert!(text.contains("📢 @testchannel"));
    assert!(text.contains("Hello world"));
    assert!(text.contains("Source"));
    assert!(text.contains("──────────"));

    // Should have 2 entities: bold channel name + source link
    assert_eq!(entities.len(), 2);
    assert!(matches!(entities[0].kind, MessageEntityKind::Bold));
    assert!(matches!(
        entities[1].kind,
        MessageEntityKind::TextLink { .. }
    ));
}

#[test]
fn test_format_message_preserves_entities() {
    let original_entities = vec![MessageEntity::new(MessageEntityKind::Bold, 0, 5)];

    let (_text, entities) = format_message(
        123456,
        "test".to_string(),
        1,
        "Hello world".to_string(),
        original_entities,
    );

    // Should have 3 entities: bold channel + shifted original bold + source link
    assert_eq!(entities.len(), 3);

    // First entity is bold channel name
    assert!(matches!(entities[0].kind, MessageEntityKind::Bold));

    // Second entity is the shifted original (offset should be > 0 now)
    assert!(matches!(entities[1].kind, MessageEntityKind::Bold));
    assert!(entities[1].offset > 0);
    assert_eq!(entities[1].length, 5);

    // Third is source link
    assert!(matches!(
        entities[2].kind,
        MessageEntityKind::TextLink { .. }
    ));
}

#[test]
fn test_format_message_source_link_correct() {
    let (_, entities) = format_message(
        123456,
        "mychannel".to_string(),
        42,
        "Test".to_string(),
        vec![],
    );

    let source_entity = entities.last().unwrap();
    if let MessageEntityKind::TextLink { url } = &source_entity.kind {
        assert_eq!(url.as_str(), "https://t.me/c/123456/42");
    } else {
        panic!("Expected TextLink");
    }
}

#[test]
fn test_format_message_utf16_offsets() {
    // Test with emoji in channel name (emoji is 2 UTF-16 code units)
    let (text, entities) = format_message(
        123456,
        "test".to_string(),
        1,
        "👋 Hello".to_string(), // emoji at start
        vec![MessageEntity::new(MessageEntityKind::Bold, 2, 5)], // "Hello" is bold
    );

    // The shifted entity should still point to "Hello"
    let shifted = &entities[1];
    assert_eq!(shifted.length, 5);

    // Verify text contains emoji
    assert!(text.contains("👋"));
}

#[test]
fn test_format_message_empty_text() {
    let (text, entities) = format_message(123456, "channel".to_string(), 1, "".to_string(), vec![]);

    assert!(text.contains("📢 @channel"));
    assert!(text.contains("Source"));
    assert_eq!(entities.len(), 2); // bold channel + source link
}

#[test]
fn test_source_offset_calculation() {
    let channel_handle = "test";
    let message_text = "Hello";

    let (full_text, entities) = format_message(
        1,
        channel_handle.to_string(),
        1,
        message_text.to_string(),
        vec![],
    );

    let source_entity = entities.last().unwrap();

    // Verify "Source" is at the correct position
    let source_start = source_entity.offset;
    let utf16_vec: Vec<u16> = full_text.encode_utf16().collect();
    let source_text: String = String::from_utf16_lossy(&utf16_vec[source_start..source_start + 6]);

    assert_eq!(source_text, "Source");
}

#[test]
fn test_split_telegram_message_short() {
    let text = "Hello world";
    let parts = split_telegram_message(text.to_string());

    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0], "Hello world");
}

#[test]
fn test_split_telegram_message_exact_limit() {
    let text = "a".repeat(TELEGRAM_MAX_LENGTH);
    let parts = split_telegram_message(text.clone());

    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].chars().count(), TELEGRAM_MAX_LENGTH);
}

#[test]
fn test_split_telegram_message_over_limit() {
    let text = "a".repeat(TELEGRAM_MAX_LENGTH + 100);
    let parts = split_telegram_message(text.clone());

    assert_eq!(parts.len(), 2);
    assert!(parts[0].chars().count() <= TELEGRAM_MAX_LENGTH);
    assert!(parts[1].chars().count() <= TELEGRAM_MAX_LENGTH);

    // Combined should equal original
    let combined: String = parts.concat();
    assert_eq!(combined, text);
}

#[test]
fn test_split_telegram_message_splits_at_newline() {
    let first_part = "a".repeat(TELEGRAM_MAX_LENGTH - 100);
    let second_part = "b".repeat(200);
    let text = format!("{}\n{}", first_part, second_part);

    let parts = split_telegram_message(text);

    assert_eq!(parts.len(), 2);
    assert!(parts[0].ends_with('\n') || parts[0] == first_part);
    assert!(!parts[1].starts_with('\n') || parts[1].trim() == second_part);
}

#[test]
fn test_split_telegram_message_preserves_content() {
    let text = "Hello\n".repeat(1000); // Well over limit
    let parts = split_telegram_message(text.clone());

    let combined: String = parts.concat();
    assert_eq!(combined, text);
}

#[test]
fn test_split_telegram_message_with_emoji() {
    // Emoji are multi-byte in UTF-8
    let emoji_text = "👋".repeat(TELEGRAM_MAX_LENGTH + 100);
    let parts = split_telegram_message(emoji_text.clone());

    assert!(parts.len() >= 2);

    // Each part should be within limit
    for part in &parts {
        assert!(part.chars().count() <= TELEGRAM_MAX_LENGTH);
    }

    // Combined should equal original
    let combined: String = parts.concat();
    assert_eq!(combined, emoji_text);
}

#[test]
fn test_split_telegram_message_empty() {
    let parts = split_telegram_message("".to_string());

    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0], "");
}

#[test]
fn test_split_telegram_message_multiple_splits() {
    let text = "👋".repeat(TELEGRAM_MAX_LENGTH * 3);
    let parts = split_telegram_message(text.clone());

    assert!(parts.len() >= 3);

    println!("{}", parts.len());

    for part in &parts {
        assert!(part.chars().count() <= TELEGRAM_MAX_LENGTH);
    }

    let combined: String = parts.concat();
    assert_eq!(combined, text);
}
