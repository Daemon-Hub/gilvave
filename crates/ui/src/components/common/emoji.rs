//! Twemoji URL generator and emoji text parser for cross-platform emoji rendering.

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
///
/// Follows the official Twemoji naming specification:
/// - If the emoji does not contain Zero Width Joiner (`\u{200D}`), strip all Variation Selector 16 (`\u{FE0F}`) characters.
/// - If the emoji contains `\u{200D}`, keep Variation Selector 16.
/// - Convert each code point to lowercase hex and join with `-`.
pub fn emoji_to_twemoji_url(emoji: &str) -> String {
    let has_zwj = emoji.contains('\u{200D}');
    let mut parts = Vec::new();
    for ch in emoji.chars() {
        if !has_zwj && ch == '\u{FE0F}' {
            continue;
        }
        parts.push(format!("{:x}", ch as u32));
    }
    format!(
        "https://cdn.jsdelivr.net/gh/jdecked/twemoji@latest/assets/svg/{}.svg",
        parts.join("-")
    )
}

/// Parses message text into plain text chunks and recognized emoji segments.
/// Emojis are matched greedily using the `emojis` crate catalog.
pub fn parse_message_segments(text: &str) -> Vec<MessageSegment> {
    let mut segments = Vec::new();
    let mut current_text = String::new();
    let mut remaining = text;

    while !remaining.is_empty() {
        let first_char = remaining.chars().next().unwrap();

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_twemoji_url_generation() {
        // Flag (Regional Indicator pair)
        assert_eq!(
            emoji_to_twemoji_url("🇷🇺"),
            "https://cdn.jsdelivr.net/gh/jdecked/twemoji@latest/assets/svg/1f1f7-1f1fa.svg"
        );
        // Simple emoji without VS16
        assert_eq!(
            emoji_to_twemoji_url("😀"),
            "https://cdn.jsdelivr.net/gh/jdecked/twemoji@latest/assets/svg/1f600.svg"
        );
        // Emoji with stripped VS16
        assert_eq!(
            emoji_to_twemoji_url("❤️"),
            "https://cdn.jsdelivr.net/gh/jdecked/twemoji@latest/assets/svg/2764.svg"
        );
        // Complex ZWJ sequence keeps VS16
        assert_eq!(
            emoji_to_twemoji_url("🏴‍☠️"),
            "https://cdn.jsdelivr.net/gh/jdecked/twemoji@latest/assets/svg/1f3f4-200d-2620-fe0f.svg"
        );
        // Keycap without ZWJ strips VS16
        assert_eq!(
            emoji_to_twemoji_url("1️⃣"),
            "https://cdn.jsdelivr.net/gh/jdecked/twemoji@latest/assets/svg/31-20e3.svg"
        );
    }

    #[test]
    fn test_parse_message_segments() {
        let msg = "Привет 🇷🇺! Как дела 🫠? Супер 👍 и 🏴‍☠️ ого! Цифра 42.";
        let segments = parse_message_segments(msg);

        // Check text and emoji separation
        assert_eq!(segments[0], MessageSegment::Text("Привет ".to_string()));
        assert!(matches!(&segments[1], MessageSegment::Emoji { char, .. } if char == "🇷🇺"));
        assert_eq!(segments[2], MessageSegment::Text("! Как дела ".to_string()));
        assert!(matches!(&segments[3], MessageSegment::Emoji { char, .. } if char == "🫠"));
        assert_eq!(segments[4], MessageSegment::Text("? Супер ".to_string()));
        assert!(matches!(&segments[5], MessageSegment::Emoji { char, .. } if char == "👍"));
        assert_eq!(segments[6], MessageSegment::Text(" и ".to_string()));
        assert!(matches!(&segments[7], MessageSegment::Emoji { char, .. } if char == "🏴‍☠️"));
        assert_eq!(segments[8], MessageSegment::Text(" ого! Цифра 42.".to_string()));
    }

    #[test]
    fn test_plain_text_without_emojis() {
        let msg = "Просто текст без смайлов 123.";
        let segments = parse_message_segments(msg);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0], MessageSegment::Text(msg.to_string()));
    }
}
