//! Twemoji SVG URL generation and text segmentation with multi-pack support.

use super::pack::{emoji_to_pack_url, EmojiPack};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageSegment {
    Text(String),
    Emoji {
        char: String,
        url: String,
        name: String,
        pack: Option<EmojiPack>,
    },
}

const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

/// 256-bit lookup mask for candidate emoji/tag bytes:
/// '<', '#', '*', '0'..='9', 0xE2, 0xE3, 0xF0.
const CANDIDATE_BITMAP: [u64; 4] = {
    let mut map = [0u64; 4];
    map[0] |= 1u64 << 35; // '#'
    map[0] |= 1u64 << 42; // '*'
    let mut d = 48; // '0'..='9'
    while d <= 57 {
        map[0] |= 1u64 << d;
        d += 1;
    }
    map[0] |= 1u64 << 60; // '<'
    map[3] |= 1u64 << (226 - 192); // 0xE2
    map[3] |= 1u64 << (227 - 192); // 0xE3
    map[3] |= 1u64 << (240 - 192); // 0xF0
    map
};

#[inline(always)]
fn is_candidate_byte(b: u8) -> bool {
    (CANDIDATE_BITMAP[(b >> 6) as usize] & (1u64 << (b & 63))) != 0
}

/// Finds the index of the first candidate emoji/tag byte using 8-byte SWAR chunks.
#[inline(always)]
fn find_first_candidate(bytes: &[u8]) -> usize {
    let (chunks, rem) = bytes.as_chunks::<8>();
    let mut offset = 0;

    for chunk in chunks {
        let b0 = is_candidate_byte(chunk[0]);
        let b1 = is_candidate_byte(chunk[1]);
        let b2 = is_candidate_byte(chunk[2]);
        let b3 = is_candidate_byte(chunk[3]);
        let b4 = is_candidate_byte(chunk[4]);
        let b5 = is_candidate_byte(chunk[5]);
        let b6 = is_candidate_byte(chunk[6]);
        let b7 = is_candidate_byte(chunk[7]);

        if b0 | b1 | b2 | b3 | b4 | b5 | b6 | b7 {
            if b0 { return offset; }
            if b1 { return offset + 1; }
            if b2 { return offset + 2; }
            if b3 { return offset + 3; }
            if b4 { return offset + 4; }
            if b5 { return offset + 5; }
            if b6 { return offset + 6; }
            return offset + 7;
        }
        offset += 8;
    }

    for (i, &b) in rem.iter().enumerate() {
        if is_candidate_byte(b) {
            return offset + i;
        }
    }

    bytes.len()
}

/// Converts a Unicode emoji sequence into its corresponding Twemoji SVG CDN URL.
pub fn emoji_to_twemoji_url(emoji: &str) -> String {
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
        let cp = ch as u32;
        let mut buf = [0u8; 8];
        let mut n = cp;
        let mut pos = 8;
        if n == 0 {
            pos -= 1;
            buf[pos] = b'0';
        } else {
            while n > 0 {
                pos -= 1;
                buf[pos] = HEX_DIGITS[(n & 0xF) as usize];
                n >>= 4;
            }
        }
        for &b in &buf[pos..] {
            url.push(b as char);
        }
    }
    url.push_str(".svg");
    url
}

/// Parses message text into plain text chunks and recognized emoji segments using the specified pack.
pub fn parse_message_segments(text: &str, pack: EmojiPack) -> Vec<MessageSegment> {
    if text.is_empty() {
        return Vec::new();
    }

    // Fast-path: if text contains no emoji or pack candidate bytes, return single text segment
    let bytes = text.as_bytes();
    let first_cand = find_first_candidate(bytes);
    if first_cand == bytes.len() {
        return vec![MessageSegment::Text(text.to_string())];
    }

    let mut segments = Vec::with_capacity(4);
    let mut current_text = String::with_capacity(text.len());
    let mut remaining = text;

    while !remaining.is_empty() {
        // Fast match for explicit pack-tagged emojis: <e:{pack}:{emoji}>
        if remaining.starts_with("<e:")
            && let Some(end_idx) = remaining.find('>')
        {
            let inner = &remaining[3..end_idx];
            if let Some((pack_str, emoji_str)) = inner.split_once(':')
                && let Some(explicit_pack) = EmojiPack::try_from_id(pack_str)
                && !emoji_str.is_empty()
                && !emoji_str.contains('<')
            {
                if !current_text.is_empty() {
                    segments.push(MessageSegment::Text(std::mem::take(&mut current_text)));
                }
                let name = emojis::get(emoji_str)
                    .map(|e| e.name().to_string())
                    .unwrap_or_else(|| emoji_str.to_string());
                let url = emoji_to_pack_url(emoji_str, explicit_pack);
                segments.push(MessageSegment::Emoji {
                    char: emoji_str.to_string(),
                    url,
                    name,
                    pack: Some(explicit_pack),
                });
                remaining = &remaining[end_idx + 1..];
                continue;
            }
        }

        // Fast scan: consume plain-text spans that cannot contain any emoji candidates in bulk
        let rem_bytes = remaining.as_bytes();
        let non_candidate_len = find_first_candidate(rem_bytes);

        if non_candidate_len > 0 {
            current_text.push_str(&remaining[..non_candidate_len]);
            remaining = &remaining[non_candidate_len..];
            continue;
        }

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
            let url = emoji_to_pack_url(&emoji_str, pack);
            segments.push(MessageSegment::Emoji {
                char: emoji_str,
                url,
                name: emoji.name().to_string(),
                pack: None,
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

/// Convenience helper that parses segments using the currently saved emoji pack.
#[allow(dead_code)]
pub fn parse_message_segments_default(text: &str) -> Vec<MessageSegment> {
    parse_message_segments(text, EmojiPack::load_saved())
}
