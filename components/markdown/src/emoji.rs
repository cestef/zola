use std::borrow::Cow;

use libs::regex::{self, Regex};
use twemoji_assets::{svg::SvgTwemojiAsset, svg_twemoji_asset_from_name};

pub struct TwemojiReplacer {
    regex: Regex,
}

impl TwemojiReplacer {
    /// There is some small setup cost
    pub fn new() -> Self {
        Self { regex: Regex::new(r":([a-z1238+-][a-z0-9_-]*):").unwrap() }
    }

    /// Replaces all occurrences of `:emoji_names:` in the string
    ///
    /// It may return `Cow::Borrowed` if there were no emoji-like
    /// patterns in the string. Call `.to_string()` if you need
    /// `String` or `.as_ref()` to get `&str`.
    pub fn replace_all<'a>(&self, text: &'a str) -> Cow<'a, str> {
        self.regex.replace_all(text, TwemojiRegexReplacer)
    }
}

struct TwemojiRegexReplacer;

impl regex::Replacer for TwemojiRegexReplacer {
    fn replace_append(&mut self, capts: &regex::Captures<'_>, dst: &mut String) {
        dst.push_str(
            SvgTwemojiAsset::from_name(&capts[1])
                .unwrap_or(svg_twemoji_asset_from_name!("question")),
        );
    }
}
