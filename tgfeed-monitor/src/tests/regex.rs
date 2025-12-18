use crate::update::get_ad_pattern;

#[test]
fn test_ad_pattern_hashtag_lowercase() {
    let pattern = get_ad_pattern();
    assert!(pattern.is_match("#реклама"));
}

#[test]
fn test_ad_pattern_hashtag_uppercase() {
    let pattern = get_ad_pattern();
    assert!(pattern.is_match("#РЕКЛАМА"));
}

#[test]
fn test_ad_pattern_hashtag_mixed_case() {
    let pattern = get_ad_pattern();
    assert!(pattern.is_match("#Реклама"));
}

#[test]
fn test_ad_pattern_hashtag_in_text() {
    let pattern = get_ad_pattern();
    assert!(pattern.is_match("Это рекламный пост #реклама"));
}

#[test]
fn test_ad_pattern_erid_space() {
    let pattern = get_ad_pattern();
    assert!(pattern.is_match("erid LjN8KXck9"));
}

#[test]
fn test_ad_pattern_erid_colon() {
    let pattern = get_ad_pattern();
    assert!(pattern.is_match("erid: 2VtzqvQXYfG"));
}

#[test]
fn test_ad_pattern_erid_equals() {
    let pattern = get_ad_pattern();
    assert!(pattern.is_match("erid=abc123def"));
}

#[test]
fn test_ad_pattern_erid_uppercase() {
    let pattern = get_ad_pattern();
    assert!(pattern.is_match("ERID: LjN8K123"));
}

#[test]
fn test_ad_pattern_erid_mixed_case() {
    let pattern = get_ad_pattern();
    assert!(pattern.is_match("Erid: 2VtzqvQXYfG"));
}

#[test]
fn test_ad_pattern_erid_with_slash() {
    let pattern = get_ad_pattern();
    assert!(pattern.is_match("https://example.com/erid=LjN8KXck9"));
}

#[test]
fn test_ad_pattern_erid_with_backslash() {
    let pattern = get_ad_pattern();
    assert!(pattern.is_match("path\\erid:2VtzqvQXYfG"));
}

#[test]
fn test_ad_pattern_erid_with_question_mark() {
    let pattern = get_ad_pattern();
    assert!(pattern.is_match("https://example.com?erid=LjN8KXck9"));
}

#[test]
fn test_ad_pattern_erid_with_ampersand() {
    let pattern = get_ad_pattern();
    assert!(pattern.is_match("https://example.com?foo=bar&erid=LjN8K1234"));
}

#[test]
fn test_ad_pattern_erid_at_start() {
    let pattern = get_ad_pattern();
    assert!(pattern.is_match("erid:LjN8KXck9 some text"));
}

#[test]
fn test_ad_pattern_erid_multiline() {
    let pattern = get_ad_pattern();
    assert!(pattern.is_match("Some text\nerid: LjN8KXck9\nmore text"));
}

#[test]
fn test_ad_pattern_no_match_erid_in_word() {
    let pattern = get_ad_pattern();
    assert!(!pattern.is_match("administered the test"));
    assert!(!pattern.is_match("inheridance is important"));
}

#[test]
fn test_ad_pattern_no_match_erid_without_token() {
    let pattern = get_ad_pattern();
    // These should not match because there's no alphanumeric token after erid
    assert!(!pattern.is_match("erid is a new standard"));
    assert!(!pattern.is_match("What is erid?"));
    assert!(!pattern.is_match("erid: "));
    assert!(!pattern.is_match("erid= "));
}

#[test]
fn test_ad_pattern_no_match_random_text() {
    let pattern = get_ad_pattern();
    assert!(!pattern.is_match("This is just a regular message"));
    assert!(!pattern.is_match("No ads here!"));
}

#[test]
fn test_ad_pattern_real_world_example_1() {
    let pattern = get_ad_pattern();
    let text = "Скидка 50% на все товары!\n\n#реклама\nООО \"Компания\"";
    assert!(pattern.is_match(text));
}

#[test]
fn test_ad_pattern_real_world_example_2() {
    let pattern = get_ad_pattern();
    let text = "Реклама. erid: 2VtzqvQXYfG\nПокупайте наш продукт!";
    assert!(pattern.is_match(text));
}

#[test]
fn test_ad_pattern_real_world_example_3() {
    let pattern = get_ad_pattern();
    let text = "Супер предложение! https://example.com?erid=LjN8KXck9";
    assert!(pattern.is_match(text));
}
