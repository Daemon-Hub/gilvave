//! Emoji pack definitions, persistent preferences, and multi-source URL resolution.

use serde::{Deserialize, Serialize};

/// Key used to persist the active emoji pack selection in browser local storage.
#[allow(dead_code)]
pub const STORAGE_KEY_EMOJI_PACK: &str = "gilvave_emoji_pack";

/// Available emoji visual packs supported across the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum EmojiPack {
    /// Microsoft Fluent 3D glossy WebP emojis.
    #[default]
    Fluent3D,
    /// Apple / iOS / Telegram style high-res PNG emojis.
    Apple,
    /// Google Noto Color Emoji standard Android style.
    Noto,
    /// Classic vector SVG Twemoji.
    Twemoji,
}

/// All selectable emoji packs presented in the appearance settings.
pub const ALL_EMOJI_PACKS: [EmojiPack; 3] = [
    EmojiPack::Fluent3D,
    EmojiPack::Apple,
    EmojiPack::Noto,
];

impl EmojiPack {
    /// Returns the unique programmatic identifier for the pack.
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Fluent3D => "fluent_3d",
            Self::Apple => "apple",
            Self::Noto => "noto",
            Self::Twemoji => "twemoji",
        }
    }

    /// Resolves an `EmojiPack` from its string identifier with fallback to Fluent 3D.
    #[allow(dead_code)]
    pub fn from_id(id: &str) -> Self {
        Self::try_from_id(id).unwrap_or_default()
    }

    /// Parses an `EmojiPack` from its string identifier without fallback.
    pub fn try_from_id(id: &str) -> Option<Self> {
        match id {
            "fluent_3d" => Some(Self::Fluent3D),
            "apple" => Some(Self::Apple),
            "noto" => Some(Self::Noto),
            "twemoji" => Some(Self::Twemoji),
            _ => None,
        }
    }

    /// Short title used in compact pack selector tabs.
    pub fn short_name(&self) -> &'static str {
        match self {
            Self::Fluent3D => "3D Fluent",
            Self::Apple => "Apple",
            Self::Noto => "Google Noto",
            Self::Twemoji => "Twemoji",
        }
    }

    /// Human-readable title displayed in the user settings UI.
    #[allow(dead_code)]
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Fluent3D => "Microsoft Fluent 3D",
            Self::Apple => "Apple (Telegram)",
            Self::Noto => "Google Noto Color",
            Self::Twemoji => "Twitter Twemoji",
        }
    }

    /// Localized description explaining the style of the pack.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Fluent3D => "Объёмные глянцевые 3D-эмодзи высокого разрешения",
            Self::Apple => "Классический детальный стиль iOS и Telegram",
            Self::Noto => "Стандартные современные эмодзи Google и Android",
            Self::Twemoji => "Классические плоские векторные SVG иконки",
        }
    }

    /// Loads the persisted emoji pack from `localStorage`, defaulting to `Fluent3D`.
    pub fn load_saved() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::window()
                .and_then(|w| w.local_storage().ok().flatten())
                .and_then(|s| s.get_item(STORAGE_KEY_EMOJI_PACK).ok().flatten())
                .map(|v| Self::from_id(&v))
                .unwrap_or_default()
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self::default()
        }
    }

    /// Persists the selected emoji pack to `localStorage`.
    pub fn save(&self) {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten())
            {
                let _ = storage.set_item(STORAGE_KEY_EMOJI_PACK, self.as_str());
            }
        }
    }
}

const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

/// Converts a Unicode emoji string into a standardized hyphen-separated lowercase hex codepoint string.
///
/// Fully adheres to the standard iamcal / Unicode Consortium data source specification:
/// - ASCII digits (0-9), '#' and '*' pad to 4 hex digits (e.g. `0031-fe0f-20e3`).
/// - Standard codepoints are formatted in lowercase hex (e.g. `1f600`, `2764-fe0f`).
pub fn emoji_to_codepoint(emoji: &str) -> String {
    let mut code = String::with_capacity(32);
    let mut first = true;
    for ch in emoji.chars() {
        if !first {
            code.push('-');
        }
        first = false;
        let cp = ch as u32;
        if cp < 0x1000 {
            code.push(HEX_DIGITS[((cp >> 12) & 0xF) as usize] as char);
            code.push(HEX_DIGITS[((cp >> 8) & 0xF) as usize] as char);
            code.push(HEX_DIGITS[((cp >> 4) & 0xF) as usize] as char);
            code.push(HEX_DIGITS[(cp & 0xF) as usize] as char);
        } else {
            let mut buf = [0u8; 8];
            let mut n = cp;
            let mut pos = 8;
            while n > 0 {
                pos -= 1;
                buf[pos] = HEX_DIGITS[(n & 0xF) as usize];
                n >>= 4;
            }
            for &b in &buf[pos..] {
                code.push(b as char);
            }
        }
    }
    code
}

/// Resolves the CDN URL for a given emoji under the requested visual pack.
///
/// Returns the direct image URL for the selected pack without mixing or fallback.
pub fn emoji_to_pack_url(emoji: &str, pack: EmojiPack) -> String {
    let code = emoji_to_codepoint(emoji);

    match pack {
        EmojiPack::Fluent3D => {
            format!("https://cdn.jsdelivr.net/npm/@lobehub/fluent-emoji-3d@1.1.0/assets/{code}.webp")
        }
        EmojiPack::Apple => {
            format!("https://cdn.jsdelivr.net/npm/emoji-datasource-apple@15.1.2/img/apple/64/{code}.png")
        }
        EmojiPack::Noto => {
            format!("https://cdn.jsdelivr.net/npm/emoji-datasource-google@15.1.2/img/google/64/{code}.png")
        }
        EmojiPack::Twemoji => {
            crate::components::common::emoji::twemoji::emoji_to_twemoji_url(emoji)
        }
    }
}

/// Event handler for `<img>` error events.
///
/// If an emoji fails to load within an emoji picker grid item, hides the entire
/// grid button to eliminate empty holes in the layout. Otherwise hides the image.
pub fn handle_emoji_img_error(e: web_sys::ErrorEvent) {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        if let Some(target) = e.target()
            && let Ok(img) = target.dyn_into::<web_sys::HtmlImageElement>()
        {
            if let Some(parent) = img.parent_element() {
                if parent.class_name().contains("emoji-grid-item") {
                    let _ = parent.set_attribute("style", "display: none !important");
                    return;
                }
            }
            let _ = img.set_attribute("style", "display: none !important");
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = e;
    }
}
