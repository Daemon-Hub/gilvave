//! Twemoji SVG URL generation and text segmentation.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageSegment {
    Text(String),
    Emoji {
        char: String,
        url: String,
        name: String,
    },
}

/// Converts a Unicode emoji sequence into its corresponding Twemoji SVG CDN URL.
pub fn emoji_to_twemoji_url(emoji: &str) -> String {
    use std::fmt::Write;

    let has_zwj = emoji.contains('\u{200D}');
    let mut url = String::with_capacity(75 + emoji.len() * 4);
    url.push_str("https://cdn.jsdelivr.net/gh/jdecked/twemoji@latest/assets/svg/");
    let mut first = true;
    for ch in emoji.chars() {
        if !has_zwj && ch == '\u{FE0F}' {
            continue;
        }
        if !first {
            url.push('-');
        }
        first = false;
        let _ = write!(url, "{:x}", ch as u32);
    }
    url.push_str(".svg");
    url
}

/// Parses message text into plain text chunks and recognized emoji segments.
pub fn parse_message_segments(text: &str) -> Vec<MessageSegment> {
    let mut segments = Vec::new();
    let mut current_text = String::new();
    let mut remaining = text;

    while !remaining.is_empty() {
        let Some(first_char) = remaining.chars().next() else {
            break;
        };

        // Fast-path filter: skip characters that cannot start any valid emoji
        let is_candidate = matches!(
            first_char,
            '#' | '*'
                | '0'..='9'
                | '\u{2000}'..='\u{3299}'
                | '\u{1F000}'..='\u{1FAFF}'
        );

        if !is_candidate {
            current_text.push(first_char);
            remaining = &remaining[first_char.len_utf8()..];
            continue;
        }

        // Greedy matching: find the longest matching emoji sequence starting here
        let mut matched: Option<(&'static emojis::Emoji, usize)> = None;
        let mut byte_len = 0;
        let mut char_count = 0;

        for ch in remaining.chars() {
            byte_len += ch.len_utf8();
            char_count += 1;
            if byte_len > 32 || char_count > 12 {
                break;
            }
            if let Some(emoji) = emojis::get(&remaining[..byte_len]) {
                matched = Some((emoji, byte_len));
            }
        }

        if let Some((emoji, len)) = matched {
            if !current_text.is_empty() {
                segments.push(MessageSegment::Text(std::mem::take(&mut current_text)));
            }
            let emoji_str = emoji.as_str().to_string();
            let url = emoji_to_twemoji_url(&emoji_str);
            segments.push(MessageSegment::Emoji {
                char: emoji_str,
                url,
                name: emoji.name().to_string(),
            });
            remaining = &remaining[len..];
        } else {
            current_text.push(first_char);
            remaining = &remaining[first_char.len_utf8()..];
        }
    }

    if !current_text.is_empty() {
        segments.push(MessageSegment::Text(current_text));
    }

    segments
}
