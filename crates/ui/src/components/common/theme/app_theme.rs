//! Built-in and active application theme definitions.

use super::css_engine::set_style_tag;
use super::custom_theme::CustomTheme;
use super::{STORAGE_KEY_ACTIVE_THEME, STYLE_TAG_ACTIVE_CUSTOM};

#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub enum AppTheme {
    Standard,
    #[default]
    Advanced,
    Light,
    Custom(String),
}

impl AppTheme {
    pub fn as_id(&self) -> &str {
        match self {
            Self::Standard => "standard",
            Self::Advanced => "advanced",
            Self::Light => "light",
            Self::Custom(id) => id.as_str(),
        }
    }

    pub fn from_id(value: &str) -> Self {
        match value.trim() {
            "standard" => Self::Standard,
            "light" => Self::Light,
            "advanced" | "" => Self::Advanced,
            other => Self::Custom(other.to_string()),
        }
    }

    pub fn load_saved() -> Self {
        web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
            .and_then(|s| s.get_item(STORAGE_KEY_ACTIVE_THEME).ok().flatten())
            .map(|v| Self::from_id(&v))
            .unwrap_or_default()
    }

    pub fn apply(&self, custom_themes: &[CustomTheme]) {
        let Some(window) = web_sys::window() else {
            return;
        };
        let Some(doc) = window.document() else {
            return;
        };

        match self {
            Self::Standard => {
                if let Some(root) = doc.document_element() {
                    let _ = root.set_attribute("data-theme", "standard");
                }
                set_style_tag(&doc, STYLE_TAG_ACTIVE_CUSTOM, "");
            }
            Self::Advanced => {
                if let Some(root) = doc.document_element() {
                    let _ = root.set_attribute("data-theme", "advanced");
                }
                set_style_tag(&doc, STYLE_TAG_ACTIVE_CUSTOM, "");
            }
            Self::Light => {
                if let Some(root) = doc.document_element() {
                    let _ = root.set_attribute("data-theme", "light");
                }
                set_style_tag(&doc, STYLE_TAG_ACTIVE_CUSTOM, "");
            }
            Self::Custom(id) => {
                if let Some(theme) = custom_themes.iter().find(|t| &t.id == id) {
                    let base_attr = if theme.base.eq_ignore_ascii_case("advanced") {
                        "advanced"
                    } else if theme.base.eq_ignore_ascii_case("light") {
                        "light"
                    } else {
                        "standard"
                    };
                    if let Some(root) = doc.document_element() {
                        let _ = root.set_attribute("data-theme", base_attr);
                    }
                    let css = theme.to_active_css();
                    set_style_tag(&doc, STYLE_TAG_ACTIVE_CUSTOM, &css);
                } else {
                    if let Some(root) = doc.document_element() {
                        let _ = root.set_attribute("data-theme", "advanced");
                    }
                    set_style_tag(&doc, STYLE_TAG_ACTIVE_CUSTOM, "");
                }
            }
        }

        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item(STORAGE_KEY_ACTIVE_THEME, self.as_id());
        }
    }
}

