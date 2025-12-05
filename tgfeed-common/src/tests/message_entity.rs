use grammers_tl_types as tl;

use crate::utils::convert_entities;

#[test]
fn test_convert_empty_entities() {
    let result = convert_entities(None);
    assert!(result.is_empty());

    let result = convert_entities(Some(&vec![]));
    assert!(result.is_empty());
}

#[test]
fn test_convert_bold() {
    let entities = vec![tl::enums::MessageEntity::Bold(
        tl::types::MessageEntityBold {
            offset: 0,
            length: 4,
        },
    )];

    let result = convert_entities(Some(&entities));

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].offset, 0);
    assert_eq!(result[0].length, 4);
    assert!(matches!(
        result[0].kind,
        teloxide::types::MessageEntityKind::Bold
    ));
}

#[test]
fn test_convert_italic() {
    let entities = vec![tl::enums::MessageEntity::Italic(
        tl::types::MessageEntityItalic {
            offset: 5,
            length: 6,
        },
    )];

    let result = convert_entities(Some(&entities));

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].offset, 5);
    assert_eq!(result[0].length, 6);
    assert!(matches!(
        result[0].kind,
        teloxide::types::MessageEntityKind::Italic
    ));
}

#[test]
fn test_convert_multiple_entities() {
    let entities = vec![
        tl::enums::MessageEntity::Bold(tl::types::MessageEntityBold {
            offset: 0,
            length: 4,
        }),
        tl::enums::MessageEntity::Italic(tl::types::MessageEntityItalic {
            offset: 5,
            length: 6,
        }),
        tl::enums::MessageEntity::Code(tl::types::MessageEntityCode {
            offset: 12,
            length: 8,
        }),
    ];

    let result = convert_entities(Some(&entities));

    assert_eq!(result.len(), 3);
    assert!(matches!(
        result[0].kind,
        teloxide::types::MessageEntityKind::Bold
    ));
    assert!(matches!(
        result[1].kind,
        teloxide::types::MessageEntityKind::Italic
    ));
    assert!(matches!(
        result[2].kind,
        teloxide::types::MessageEntityKind::Code
    ));
}

#[test]
fn test_convert_text_url() {
    let entities = vec![tl::enums::MessageEntity::TextUrl(
        tl::types::MessageEntityTextUrl {
            offset: 0,
            length: 4,
            url: "https://example.com".to_string(),
        },
    )];

    let result = convert_entities(Some(&entities));

    assert_eq!(result.len(), 1);
    assert!(matches!(
        result[0].kind,
        teloxide::types::MessageEntityKind::TextLink { .. }
    ));
}

#[test]
fn test_convert_invalid_url_skipped() {
    let entities = vec![tl::enums::MessageEntity::TextUrl(
        tl::types::MessageEntityTextUrl {
            offset: 0,
            length: 4,
            url: "not a valid url".to_string(),
        },
    )];

    let result = convert_entities(Some(&entities));

    assert!(result.is_empty()); // Invalid URL should be skipped
}

#[test]
fn test_convert_unknown_entity_skipped() {
    let entities = vec![tl::enums::MessageEntity::BankCard(
        tl::types::MessageEntityBankCard {
            offset: 0,
            length: 16,
        },
    )];

    let result = convert_entities(Some(&entities));

    assert!(result.is_empty()); // BankCard is not handled
}
