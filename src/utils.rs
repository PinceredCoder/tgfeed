use std::io::{BufRead, Write};

pub fn prompt(message: &str) -> anyhow::Result<String> {
    print!("{}", message);
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().lock().read_line(&mut input)?;

    Ok(input.trim().to_string())
}
