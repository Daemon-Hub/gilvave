//! Unit tests for emoji URL generation and text segmentation.

#[cfg(test)]
mod tests {
    use super::super::*;

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
        assert_eq!(
            segments[8],
            MessageSegment::Text(" ого! Цифра 42.".to_string())
        );
    }

    #[test]
    fn test_plain_text_without_emojis() {
        let msg = "Просто текст без смайлов 123.";
        let segments = parse_message_segments(msg);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0], MessageSegment::Text(msg.to_string()));
    }
}
