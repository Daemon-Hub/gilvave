//! Unit tests for emoji URL generation, pack resolution, and text segmentation.

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
    }

    #[test]
    fn test_emoji_codepoints_and_packs() {
        // Fluent 3D URL
        let fluent_url = emoji_to_pack_url("😀", EmojiPack::Fluent3D);
        assert_eq!(
            fluent_url,
            "https://cdn.jsdelivr.net/npm/@lobehub/fluent-emoji-3d@1.1.0/assets/1f600.webp"
        );

        // Apple / Telegram URL
        let apple_url = emoji_to_pack_url("🫠", EmojiPack::Apple);
        assert_eq!(
            apple_url,
            "https://cdn.jsdelivr.net/npm/emoji-datasource-apple@15.1.2/img/apple/64/1fae0.png"
        );

        // Google Noto URL
        let noto_url = emoji_to_pack_url("🇷🇺", EmojiPack::Noto);
        assert_eq!(
            noto_url,
            "https://cdn.jsdelivr.net/npm/emoji-datasource-google@15.1.2/img/google/64/1f1f7-1f1fa.png"
        );

        // Keycap padded codepoints (0031-fe0f-20e3)
        let keycap_code = emoji_to_codepoint("1️⃣");
        assert_eq!(keycap_code, "0031-fe0f-20e3");
        let keycap_url = emoji_to_pack_url("1️⃣", EmojiPack::Fluent3D);
        assert_eq!(
            keycap_url,
            "https://cdn.jsdelivr.net/npm/@lobehub/fluent-emoji-3d@1.1.0/assets/0031-fe0f-20e3.webp"
        );
    }

    #[test]
    fn test_parse_message_segments() {
        let msg = "Привет 🇷🇺! Как дела 🫠? Супер 👍 и 🏴‍☠️ ого! Цифра 42.";
        let segments = parse_message_segments(msg, EmojiPack::Fluent3D);

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
        let segments = parse_message_segments(msg, EmojiPack::Apple);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0], MessageSegment::Text(msg.to_string()));
    }

    #[test]
    fn test_parse_explicit_pack_emoji_segments() {
        let msg = "Смотри: <e:apple:🍏> и <e:fluent_3d:💎> и обычный 😀!";
        let segments = parse_message_segments(msg, EmojiPack::Noto);
        assert_eq!(segments[0], MessageSegment::Text("Смотри: ".to_string()));
        assert!(matches!(&segments[1], MessageSegment::Emoji { char, pack: Some(EmojiPack::Apple), .. } if char == "🍏"));
        assert_eq!(segments[2], MessageSegment::Text(" и ".to_string()));
        assert!(matches!(&segments[3], MessageSegment::Emoji { char, pack: Some(EmojiPack::Fluent3D), .. } if char == "💎"));
        assert_eq!(segments[4], MessageSegment::Text(" и обычный ".to_string()));
        assert!(matches!(&segments[5], MessageSegment::Emoji { char, pack: None, .. } if char == "😀"));
        assert_eq!(segments[6], MessageSegment::Text("!".to_string()));
    }

    #[test]
    fn test_pack_mixing_and_permanence() {
        // Tagged emojis preserve their explicit pack regardless of the default pack parameter passed
        let mixed_msg = "<e:fluent_3d:🔥><e:apple:🍏><e:noto:🤖>";
        for fallback_pack in [EmojiPack::Fluent3D, EmojiPack::Apple, EmojiPack::Noto] {
            let segs = parse_message_segments(mixed_msg, fallback_pack);
            assert_eq!(segs.len(), 3);
            assert!(matches!(&segs[0], MessageSegment::Emoji { char, pack: Some(EmojiPack::Fluent3D), url, .. }
                if char == "🔥" && url.contains("fluent-emoji-3d")));
            assert!(matches!(&segs[1], MessageSegment::Emoji { char, pack: Some(EmojiPack::Apple), url, .. }
                if char == "🍏" && url.contains("emoji-datasource-apple")));
            assert!(matches!(&segs[2], MessageSegment::Emoji { char, pack: Some(EmojiPack::Noto), url, .. }
                if char == "🤖" && url.contains("emoji-datasource-google")));
        }
    }
}
