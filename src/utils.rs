use std::io::{BufRead, Write};

pub fn prompt(message: &str) -> anyhow::Result<String> {
    print!("{}", message);
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().lock().read_line(&mut input)?;

    Ok(input.trim().to_string())
}

/// Parse a command from message text
/// Returns (command, args) or None if not a command
pub fn parse_command(text: &str) -> Option<(&str, &str)> {
    let text = text.trim();

    if !text.starts_with('/') {
        return None;
    }

    let mut parts = text.splitn(2, |c: char| c.is_whitespace());
    let command = parts.next()?.trim_start_matches('/');
    let args = parts.next().unwrap_or("");

    Some((command, args))
}
