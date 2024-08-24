use clap::builder::styling;
use clap::builder::Styles;

use regex::Regex;
use serde::{Deserialize, Serialize};

pub fn vault_styling() -> Styles {
    styling::Styles::styled()
        .header(styling::AnsiColor::Green.on_default() | styling::Effects::BOLD)
        .usage(styling::AnsiColor::Green.on_default() | styling::Effects::BOLD)
        .literal(styling::AnsiColor::Cyan.on_default() | styling::Effects::BOLD)
        .placeholder(styling::AnsiColor::Blue.on_default())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Snippet {
    pub tag: String,
    pub description: Option<String>,
    pub code: String,
    pub timestamp: String,
    pub language: Option<String>,
    pub id: u32,
}

pub fn strip_ansi_codes(input: &str) -> String {
    let re = Regex::new(r"\x1B\[[0-9;]*[a-zA-Z]").unwrap();
    re.replace_all(input, "").to_string()
}
