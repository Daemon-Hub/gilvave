use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;

const STORAGE_KEY_ACTIVE_THEME: &str = "gilvave_theme";
const STORAGE_KEY_CUSTOM_THEMES: &str = "gilvave_installed_custom_themes";
const STORAGE_KEY_WINDOWED_MODE: &str = "gilvave_windowed_mode";
const STYLE_TAG_ACTIVE_CUSTOM: &str = "gilvave-custom-theme";
const STYLE_TAG_CUSTOM_PREVIEWS: &str = "gilvave-custom-theme-previews";

pub fn load_windowed_mode() -> bool {
    if super::helpers::is_mobile_device() {
        return false;
    }
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(STORAGE_KEY_WINDOWED_MODE).ok().flatten())
        .map(|v| v != "false")
        .unwrap_or(true)
}

pub fn save_windowed_mode(is_windowed: bool) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item(
                STORAGE_KEY_WINDOWED_MODE,
                if is_windowed { "true" } else { "false" },
            );
        }
    }
}

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

fn default_author() -> String {
    "community".to_string()
}

fn default_version() -> String {
    "1.0.0".to_string()
}

fn default_base() -> String {
    "standard".to_string()
}

#[derive(Clone, PartialEq, Eq, Default, Debug, Serialize, Deserialize)]
pub struct ThemePalette {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bg_page: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bg_sidebar: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bg_channels: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bg_chat: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bg_input: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bg_modal: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bg_message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_end: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accent_warm: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_heading: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_chat: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_muted: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_author: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub online: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idle: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub danger: Option<String>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct CustomTheme {
    pub id: String,
    pub name: String,
    #[serde(default = "default_author")]
    pub author: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_base")]
    pub base: String,
    #[serde(default)]
    pub palette: ThemePalette,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub variables: BTreeMap<String, String>,
}

impl CustomTheme {
    pub fn from_json_str(raw: &str) -> Result<Self, String> {
        let mut theme: Self = serde_json::from_str(raw)
            .map_err(|e| format!("Ошибка синтаксиса JSON: {e}"))?;

        theme.id = sanitize_id(&theme.id);
        theme.name = theme.name.trim().to_string();
        if theme.name.is_empty() {
            return Err("Поле \"name\" (название темы) не может быть пустым".to_string());
        }
        if theme.id.is_empty() {
            theme.id = sanitize_id(&theme.name);
        }
        if theme.id.is_empty() || theme.id == "standard" || theme.id == "advanced" || theme.id == "light" {
            #[cfg(target_arch = "wasm32")]
            {
                theme.id = format!("custom-{}", js_sys::Date::now() as u64);
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                theme.id = "custom-user-theme".to_string();
            }
        }
        if theme.author.trim().is_empty() {
            theme.author = default_author();
        }
        if theme.version.trim().is_empty() {
            theme.version = default_version();
        }
        if !theme.base.eq_ignore_ascii_case("advanced")
            && !theme.base.eq_ignore_ascii_case("standard")
            && !theme.base.eq_ignore_ascii_case("light")
        {
            theme.base = "standard".to_string();
        }

        // Validate that at least one palette color or CSS variable is provided
        let generated = theme.build_variable_map();
        if generated.is_empty() {
            return Err(
                "Тема не содержит ни одного валидного цвета в \"palette\" или переменной в \"variables\""
                    .to_string(),
            );
        }

        Ok(theme)
    }

    pub fn to_pretty_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    pub fn load_all() -> Vec<Self> {
        let saved = web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
            .and_then(|s| s.get_item(STORAGE_KEY_CUSTOM_THEMES).ok().flatten());

        match saved {
            Some(json) => serde_json::from_str::<Vec<Self>>(&json).unwrap_or_default(),
            None => Vec::new(),
        }
    }

    pub fn save_all(themes: &[Self]) {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(json) = serde_json::to_string(themes) {
                    let _ = storage.set_item(STORAGE_KEY_CUSTOM_THEMES, &json);
                }
            }
        }
        Self::sync_preview_styles(themes);
    }

    pub fn sync_preview_styles(themes: &[Self]) {
        let Some(window) = web_sys::window() else {
            return;
        };
        let Some(doc) = window.document() else {
            return;
        };

        let mut css = String::new();
        for cat_theme in Self::catalog_themes() {
            if !themes.iter().any(|t| t.id == cat_theme.id) {
                css.push_str(&cat_theme.to_preview_css());
                css.push('\n');
            }
        }
        for theme in themes {
            css.push_str(&theme.to_preview_css());
            css.push('\n');
        }
        set_style_tag(&doc, STYLE_TAG_CUSTOM_PREVIEWS, &css);
    }

    pub fn catalog_themes() -> Vec<Self> {
        vec![
            Self {
                id: "catppuccin-mocha".to_string(),
                name: "Catppuccin Mocha".to_string(),
                author: "catppuccin".to_string(),
                version: "1.0.0".to_string(),
                description: "Уютная пастельная тёмная тема в кофейно-лавандовых тонах с мягким контрастом."
                    .to_string(),
                base: "standard".to_string(),
                palette: ThemePalette {
                    bg_page: Some("#11111b".to_string()),
                    bg_sidebar: Some("#11111b".to_string()),
                    bg_channels: Some("#181825".to_string()),
                    bg_chat: Some("#1e1e2e".to_string()),
                    bg_input: Some("#313244".to_string()),
                    bg_modal: Some("#1e1e2e".to_string()),
                    bg_message: Some("rgba(17, 17, 27, 0.55)".to_string()),
                    border: Some("#45475a".to_string()),
                    primary: Some("#cba6f7".to_string()),
                    primary_end: Some("#f5c2e7".to_string()),
                    accent: Some("#89b4fa".to_string()),
                    accent_warm: Some("#fab387".to_string()),
                    text_heading: Some("#cdd6f4".to_string()),
                    text_chat: Some("#bac2de".to_string()),
                    text_muted: Some("#7f849c".to_string()),
                    text_author: Some("#89dceb".to_string()),
                    online: Some("#a6e3a1".to_string()),
                    idle: Some("#f9e2af".to_string()),
                    danger: Some("#f38ba8".to_string()),
                },
                variables: BTreeMap::new(),
            },
            Self {
                id: "tokyo-night".to_string(),
                name: "Tokyo Night".to_string(),
                author: "enkia".to_string(),
                version: "1.0.0".to_string(),
                description: "Неоновые огни ночного Токио: глубокий индиго, электрический голубой и пурпурный."
                    .to_string(),
                base: "advanced".to_string(),
                palette: ThemePalette {
                    bg_page: Some("#0f0f17".to_string()),
                    bg_sidebar: Some("#101018".to_string()),
                    bg_channels: Some("#16161e".to_string()),
                    bg_chat: Some("#1a1b26".to_string()),
                    bg_input: Some("#24283b".to_string()),
                    bg_modal: Some("#16161e".to_string()),
                    bg_message: Some("rgba(16, 16, 24, 0.6)".to_string()),
                    border: Some("#292e42".to_string()),
                    primary: Some("#7aa2f7".to_string()),
                    primary_end: Some("#bb9af7".to_string()),
                    accent: Some("#7dcfff".to_string()),
                    accent_warm: Some("#e0af68".to_string()),
                    text_heading: Some("#c0caf5".to_string()),
                    text_chat: Some("#a9b1d6".to_string()),
                    text_muted: Some("#565f89".to_string()),
                    text_author: Some("#7dcfff".to_string()),
                    online: Some("#73daca".to_string()),
                    idle: Some("#e0af68".to_string()),
                    danger: Some("#f7768e".to_string()),
                },
                variables: BTreeMap::new(),
            },
            Self {
                id: "dracula-official".to_string(),
                name: "Dracula Official".to_string(),
                author: "zenorocha".to_string(),
                version: "1.2.0".to_string(),
                description: "Классическая вампирская тёмная палитра с яркими фиолетовыми, розовыми и мятными акцентами."
                    .to_string(),
                base: "standard".to_string(),
                palette: ThemePalette {
                    bg_page: Some("#191a21".to_string()),
                    bg_sidebar: Some("#191a21".to_string()),
                    bg_channels: Some("#21222c".to_string()),
                    bg_chat: Some("#282a36".to_string()),
                    bg_input: Some("#343746".to_string()),
                    bg_modal: Some("#21222c".to_string()),
                    bg_message: Some("rgba(25, 26, 33, 0.58)".to_string()),
                    border: Some("#44475a".to_string()),
                    primary: Some("#bd93f9".to_string()),
                    primary_end: Some("#ff79c6".to_string()),
                    accent: Some("#8be9fd".to_string()),
                    accent_warm: Some("#ffb86c".to_string()),
                    text_heading: Some("#f8f8f2".to_string()),
                    text_chat: Some("#e6e6e0".to_string()),
                    text_muted: Some("#6272a4".to_string()),
                    text_author: Some("#8be9fd".to_string()),
                    online: Some("#50fa7b".to_string()),
                    idle: Some("#f1fa8c".to_string()),
                    danger: Some("#ff5555".to_string()),
                },
                variables: BTreeMap::new(),
            },
            Self {
                id: "nord-polar".to_string(),
                name: "Nord Polar Night".to_string(),
                author: "arcticicestudio".to_string(),
                version: "1.0.0".to_string(),
                description: "Холодная арктическая палитра: морозный синий, ледяной циан и спокойный сланцевый фон."
                    .to_string(),
                base: "standard".to_string(),
                palette: ThemePalette {
                    bg_page: Some("#242933".to_string()),
                    bg_sidebar: Some("#242933".to_string()),
                    bg_channels: Some("#2e3440".to_string()),
                    bg_chat: Some("#3b4252".to_string()),
                    bg_input: Some("#434c5e".to_string()),
                    bg_modal: Some("#2e3440".to_string()),
                    bg_message: Some("rgba(36, 41, 51, 0.55)".to_string()),
                    border: Some("#4c566a".to_string()),
                    primary: Some("#88c0d0".to_string()),
                    primary_end: Some("#81a1c1".to_string()),
                    accent: Some("#8fbcbb".to_string()),
                    accent_warm: Some("#d08770".to_string()),
                    text_heading: Some("#eceff4".to_string()),
                    text_chat: Some("#e5e9f0".to_string()),
                    text_muted: Some("#9099ab".to_string()),
                    text_author: Some("#88c0d0".to_string()),
                    online: Some("#a3be8c".to_string()),
                    idle: Some("#ebcb8b".to_string()),
                    danger: Some("#bf616a".to_string()),
                },
                variables: BTreeMap::new(),
            },
            Self {
                id: "cyberpunk-2077".to_string(),
                name: "Cyberpunk 2077".to_string(),
                author: "nightcity".to_string(),
                version: "2.0.77".to_string(),
                description: "Высококонтрастный неон Найт-Сити: горячая маджента, электрический циан и хромово-жёлтый."
                    .to_string(),
                base: "advanced".to_string(),
                palette: ThemePalette {
                    bg_page: Some("#070510".to_string()),
                    bg_sidebar: Some("#090714".to_string()),
                    bg_channels: Some("#100c22".to_string()),
                    bg_chat: Some("#140f2d".to_string()),
                    bg_input: Some("#1e1740".to_string()),
                    bg_modal: Some("#100c22".to_string()),
                    bg_message: Some("rgba(9, 7, 20, 0.68)".to_string()),
                    border: Some("#382368".to_string()),
                    primary: Some("#ff2a6d".to_string()),
                    primary_end: Some("#05d9e8".to_string()),
                    accent: Some("#05d9e8".to_string()),
                    accent_warm: Some("#fcee09".to_string()),
                    text_heading: Some("#d1f7ff".to_string()),
                    text_chat: Some("#c2e8f5".to_string()),
                    text_muted: Some("#7a6b9e".to_string()),
                    text_author: Some("#fcee09".to_string()),
                    online: Some("#00ff9f".to_string()),
                    idle: Some("#fcee09".to_string()),
                    danger: Some("#ff2a6d".to_string()),
                },
                variables: BTreeMap::new(),
            },
            Self {
                id: "rose-pine".to_string(),
                name: "Rosé Pine".to_string(),
                author: "rose-pine".to_string(),
                version: "1.1.0".to_string(),
                description: "Натуральная сосновая хвоя, искусственный мех и бархатные розово-золотые сумерки."
                    .to_string(),
                base: "standard".to_string(),
                palette: ThemePalette {
                    bg_page: Some("#12101a".to_string()),
                    bg_sidebar: Some("#12101a".to_string()),
                    bg_channels: Some("#191724".to_string()),
                    bg_chat: Some("#1f1d2e".to_string()),
                    bg_input: Some("#26233a".to_string()),
                    bg_modal: Some("#191724".to_string()),
                    bg_message: Some("rgba(18, 16, 26, 0.58)".to_string()),
                    border: Some("#403d52".to_string()),
                    primary: Some("#ebbcba".to_string()),
                    primary_end: Some("#c4a7e7".to_string()),
                    accent: Some("#9ccfd8".to_string()),
                    accent_warm: Some("#f6c177".to_string()),
                    text_heading: Some("#e0def4".to_string()),
                    text_chat: Some("#d2d0e8".to_string()),
                    text_muted: Some("#6e6a86".to_string()),
                    text_author: Some("#9ccfd8".to_string()),
                    online: Some("#31748f".to_string()),
                    idle: Some("#f6c177".to_string()),
                    danger: Some("#eb6f92".to_string()),
                },
                variables: BTreeMap::new(),
            },
            Self {
                id: "catppuccin-latte".to_string(),
                name: "Catppuccin Latte".to_string(),
                author: "catppuccin".to_string(),
                version: "1.0.0".to_string(),
                description: "Нежная, тёплая и контрастная светлая палитра Catppuccin в молочно-кофейных тонах."
                    .to_string(),
                base: "light".to_string(),
                palette: ThemePalette {
                    bg_page: Some("#dce0e8".to_string()),
                    bg_sidebar: Some("#e6e9ef".to_string()),
                    bg_channels: Some("#eff1f5".to_string()),
                    bg_chat: Some("#ffffff".to_string()),
                    bg_input: Some("#e6e9ef".to_string()),
                    bg_modal: Some("#ffffff".to_string()),
                    bg_message: Some("rgba(230, 233, 239, 0.6)".to_string()),
                    border: Some("#ccd0da".to_string()),
                    primary: Some("#8839ef".to_string()),
                    primary_end: Some("#ea76cb".to_string()),
                    accent: Some("#1e66f5".to_string()),
                    accent_warm: Some("#fe640b".to_string()),
                    text_heading: Some("#4c4f69".to_string()),
                    text_chat: Some("#5c5f77".to_string()),
                    text_muted: Some("#8c8fa1".to_string()),
                    text_author: Some("#1e66f5".to_string()),
                    online: Some("#40a02b".to_string()),
                    idle: Some("#df8e1d".to_string()),
                    danger: Some("#d20f39".to_string()),
                },
                variables: BTreeMap::new(),
            },
        ]
    }

    pub fn template_json() -> String {
        let template = Self {
            id: "my-custom-theme".to_string(),
            name: "My Custom Theme".to_string(),
            author: "your_nickname".to_string(),
            version: "1.0.0".to_string(),
            description: "Описание вашей пользовательской темы для Gilvave.".to_string(),
            base: "standard".to_string(),
            palette: ThemePalette {
                bg_page: Some("#0d1117".to_string()),
                bg_sidebar: Some("#090d13".to_string()),
                bg_channels: Some("#111722".to_string()),
                bg_chat: Some("#161b26".to_string()),
                bg_input: Some("#1f2735".to_string()),
                bg_modal: Some("#161b26".to_string()),
                bg_message: Some("rgba(9, 13, 19, 0.55)".to_string()),
                border: Some("#30363d".to_string()),
                primary: Some("#58a6ff".to_string()),
                primary_end: Some("#bc8cff".to_string()),
                accent: Some("#39c5cf".to_string()),
                accent_warm: Some("#f0883e".to_string()),
                text_heading: Some("#f0f6fc".to_string()),
                text_chat: Some("#c9d1d9".to_string()),
                text_muted: Some("#8b949e".to_string()),
                text_author: Some("#79c0ff".to_string()),
                online: Some("#3fb950".to_string()),
                idle: Some("#d29922".to_string()),
                danger: Some("#f85149".to_string()),
            },
            variables: BTreeMap::from([
                (
                    "--gradient-send-btn".to_string(),
                    "linear-gradient(135deg, #58a6ff, #bc8cff)".to_string(),
                ),
            ]),
        };
        template.to_pretty_json()
    }

    pub fn build_variable_map(&self) -> BTreeMap<String, String> {
        let mut vars = BTreeMap::new();
        let p = &self.palette;

        let clean = |opt: &Option<String>| -> Option<String> {
            opt.as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty() && is_safe_css_value(s))
                .map(ToString::to_string)
        };

        let bg_page = clean(&p.bg_page);
        let bg_sidebar = clean(&p.bg_sidebar);
        let bg_channels = clean(&p.bg_channels);
        let bg_chat = clean(&p.bg_chat);
        let bg_input = clean(&p.bg_input);
        let bg_modal = clean(&p.bg_modal);
        let bg_message = clean(&p.bg_message);
        let border = clean(&p.border);
        let primary = clean(&p.primary);
        let primary_end = clean(&p.primary_end).or_else(|| primary.clone());
        let accent = clean(&p.accent).or_else(|| primary.clone());
        let accent_warm = clean(&p.accent_warm)
            .or_else(|| accent.clone())
            .or_else(|| primary.clone());
        let text_heading = clean(&p.text_heading);
        let text_chat = clean(&p.text_chat);
        let text_muted = clean(&p.text_muted);
        let text_author = clean(&p.text_author).or_else(|| accent.clone());
        let online = clean(&p.online);
        let idle = clean(&p.idle);
        let danger = clean(&p.danger);

        if let Some(ref c) = bg_page {
            vars.insert("--color-bg-page".into(), c.clone());
            vars.insert("--color-bg-page-mid".into(), c.clone());
            vars.insert("--color-bg-page-end".into(), c.clone());
            vars.insert("--color-bg-page-overlay".into(), with_alpha(c, 0.72));
            let mid = bg_channels.as_ref().or(bg_chat.as_ref()).unwrap_or(c);
            vars.insert(
                "--gradient-body-main".into(),
                format!("linear-gradient(135deg, {c}, {mid}, {c})"),
            );
            vars.insert("--gradient-body-before".into(), "none".into());
            vars.insert("--gradient-body-after".into(), "none".into());
        }

        if let Some(ref c) = bg_sidebar {
            vars.insert("--color-bg-panel-darkest".into(), c.clone());
            vars.insert(
                "--color-bg-panel-darkest-hover".into(),
                format!("color-mix(in srgb, {c} 85%, black)"),
            );
            vars.insert(
                "--color-bg-panel-footer".into(),
                format!("color-mix(in srgb, {c} 92%, white)"),
            );
        }

        if let Some(ref c) = bg_channels {
            vars.insert("--color-bg-panel-dark".into(), c.clone());
            vars.insert("--color-char-counter-bg".into(), with_alpha(c, 0.95));
        }

        if let Some(ref c) = bg_chat {
            vars.insert("--color-bg-panel".into(), c.clone());
            vars.insert(
                "--color-bg-panel-hover".into(),
                format!("color-mix(in srgb, {c} 88%, white)"),
            );
            vars.insert(
                "--color-bg-panel-active".into(),
                format!("color-mix(in srgb, {c} 78%, white)"),
            );
            if let Some(ref ch) = bg_channels {
                vars.insert(
                    "--gradient-server-banner".into(),
                    format!("linear-gradient(135deg, {ch}, {c})"),
                );
            }
        }

        if let Some(ref c) = bg_input {
            vars.insert("--color-bg-input".into(), c.clone());
            vars.insert("--color-bg-input-panel".into(), c.clone());
            vars.insert(
                "--color-bg-input-focus".into(),
                format!("color-mix(in srgb, {c} 88%, white)"),
            );
            vars.insert(
                "--color-bg-input-panel-focus".into(),
                format!("color-mix(in srgb, {c} 90%, white)"),
            );
        }

        if let Some(ref c) = bg_modal {
            vars.insert("--color-bg-container".into(), c.clone());
            vars.insert(
                "--color-bg-container-dark".into(),
                format!("color-mix(in srgb, {c} 82%, black)"),
            );
            vars.insert("--color-bg-glass".into(), with_alpha(c, 0.95));
            vars.insert(
                "--color-bg-social".into(),
                format!("color-mix(in srgb, {c} 88%, white)"),
            );
        }

        if let Some(ref c) = bg_message {
            vars.insert("--color-bg-message".into(), c.clone());
        } else if let Some(ref c) = bg_sidebar {
            vars.insert("--color-bg-message".into(), with_alpha(c, 0.55));
        }

        if let Some(ref c) = border {
            vars.insert("--color-border".into(), c.clone());
            vars.insert("--color-border-header".into(), c.clone());
            vars.insert("--color-icon-slate-arc".into(), c.clone());
            vars.insert("--color-border-subtle".into(), with_alpha(c, 0.35));
            vars.insert("--color-border-container".into(), with_alpha(c, 0.55));
            vars.insert("--color-border-bubble".into(), with_alpha(c, 0.45));
            vars.insert("--color-border-input-panel".into(), with_alpha(c, 0.42));
            vars.insert("--color-border-slate-arc".into(), with_alpha(c, 0.45));
            vars.insert("--color-editor-counter-bg".into(), with_alpha(c, 0.35));
        }

        if let Some(ref c) = primary {
            let hover = format!("color-mix(in srgb, {c} 85%, black)");
            let light = format!("color-mix(in srgb, {c} 76%, white)");
            let lighter = format!("color-mix(in srgb, {c} 58%, white)");
            let dark = format!("color-mix(in srgb, {c} 78%, black)");

            vars.insert("--color-primary".into(), c.clone());
            vars.insert("--color-primary-hover".into(), hover);
            vars.insert("--color-primary-light".into(), light);
            vars.insert("--color-primary-lighter".into(), lighter);
            vars.insert("--color-primary-dark".into(), dark);
            vars.insert("--color-icon-violet".into(), c.clone());
            vars.insert("--color-icon-blue".into(), c.clone());

            for (suffix, alpha) in [
                ("14", 0.14),
                ("15", 0.15),
                ("16", 0.16),
                ("20", 0.20),
                ("24", 0.24),
                ("25", 0.25),
                ("28", 0.28),
                ("30", 0.30),
                ("32", 0.32),
                ("38", 0.38),
                ("40", 0.40),
                ("45", 0.45),
                ("50", 0.50),
            ] {
                vars.insert(format!("--color-primary-alpha-{suffix}"), with_alpha(c, alpha));
            }

            vars.insert(
                "--shadow-submit-hover".into(),
                format!("0 5px 20px {}", with_alpha(c, 0.4)),
            );
            vars.insert("--color-quick-settings-border".into(), with_alpha(c, 0.5));
            vars.insert(
                "--shadow-quick-settings".into(),
                format!("0 6px 20px {}", with_alpha(c, 0.15)),
            );
            vars.insert("--color-quick-settings-text".into(), c.clone());
        }

        if let Some(ref pe) = primary_end {
            let p_start = primary.as_ref().unwrap_or(pe);
            vars.insert("--color-primary-end".into(), pe.clone());
            vars.insert(
                "--color-primary-end-light".into(),
                format!("color-mix(in srgb, {pe} 75%, white)"),
            );
            vars.insert("--color-icon-magenta".into(), pe.clone());
            vars.insert("--color-primary-end-alpha-15".into(), with_alpha(pe, 0.15));
            vars.insert("--color-primary-end-alpha-25".into(), with_alpha(pe, 0.25));
            vars.insert("--color-primary-end-alpha-30".into(), with_alpha(pe, 0.30));

            let grad = format!("linear-gradient(135deg, {p_start}, {pe})");
            vars.insert("--gradient-home-btn".into(), grad.clone());
            vars.insert("--gradient-home-btn-ring".into(), grad.clone());
            vars.insert("--gradient-server-icon".into(), grad.clone());
            vars.insert("--gradient-message-avatar".into(), grad.clone());
            vars.insert("--gradient-send-btn".into(), grad.clone());
            vars.insert("--gradient-group-icon".into(), grad.clone());
            vars.insert(
                "--gradient-banner-default".into(),
                format!(
                    "linear-gradient(135deg, {}, {})",
                    with_alpha(p_start, 0.6),
                    with_alpha(pe, 0.6)
                ),
            );
            vars.insert(
                "--shadow-send-btn".into(),
                format!("0 2px 10px {}", with_alpha(p_start, 0.4)),
            );
            vars.insert(
                "--shadow-send-btn-hover".into(),
                format!("0 4px 16px {}", with_alpha(pe, 0.5)),
            );
            vars.insert("--color-quick-dms-border".into(), with_alpha(pe, 0.5));
            vars.insert(
                "--shadow-quick-dms".into(),
                format!("0 6px 20px {}", with_alpha(pe, 0.15)),
            );
            vars.insert("--color-quick-dms-text".into(), pe.clone());
            vars.insert("--gradient-voice-btn".into(), grad.clone());
            vars.insert(
                "--shadow-voice-btn".into(),
                format!("0 4px 16px {}", with_alpha(p_start, 0.25)),
            );
            vars.insert(
                "--shadow-voice-btn-hover".into(),
                format!("0 6px 20px {}", with_alpha(pe, 0.35)),
            );
            vars.insert("--gradient-voice-radar".into(), grad.clone());
            vars.insert(
                "--shadow-voice-radar".into(),
                format!("0 4px 16px {}", with_alpha(p_start, 0.25)),
            );
            vars.insert("--color-voice-ring".into(), with_alpha(p_start, 0.25));
        }

        if let Some(ref acc) = accent {
            let acc_bright = format!("color-mix(in srgb, {acc} 72%, white)");
            vars.insert("--color-icon-cyan".into(), acc.clone());
            vars.insert("--color-icon-cyan-bright".into(), acc_bright.clone());
            vars.insert("--color-primary-blue".into(), acc.clone());
            vars.insert("--color-channel-hash".into(), acc.clone());
            vars.insert("--color-channel-hash-hover".into(), acc_bright.clone());
            vars.insert("--color-channel-hash-active".into(), acc.clone());
            vars.insert(
                "--shadow-channel-active".into(),
                format!("inset 2px 0 0 {acc}"),
            );
            vars.insert(
                "--shadow-channel-hash-glow".into(),
                format!("0 0 10px {}", with_alpha(acc, 0.35)),
            );
            vars.insert(
                "--gradient-channel-active".into(),
                format!(
                    "linear-gradient(90deg, {} 0%, {} 100%)",
                    with_alpha(acc, 0.2),
                    with_alpha(acc, 0.08)
                ),
            );
            vars.insert("--gradient-pill-hover".into(), acc.clone());
            vars.insert(
                "--shadow-server-icon-hover".into(),
                format!("0 0 0 2.5px {acc}, 0 0 14px {}", with_alpha(acc, 0.4)),
            );
            vars.insert(
                "--shadow-home-btn-hover".into(),
                format!("0 0 0 3px {}, 0 0 16px {}", with_alpha(acc, 0.6), with_alpha(acc, 0.4)),
            );
            vars.insert("--color-border-bubble-hover".into(), with_alpha(acc, 0.32));
            vars.insert("--color-border-input-focus".into(), with_alpha(acc, 0.55));
            vars.insert(
                "--shadow-input-focus".into(),
                format!("0 0 0 3px {}", with_alpha(acc, 0.2)),
            );
            vars.insert("--color-server-new-border".into(), with_alpha(acc, 0.45));
            vars.insert("--color-server-new-text".into(), acc_bright.clone());
            vars.insert("--color-editor-counter-text".into(), acc_bright.clone());
            vars.insert("--color-quick-join-border".into(), with_alpha(acc, 0.5));
            vars.insert(
                "--shadow-quick-join".into(),
                format!("0 6px 20px {}", with_alpha(acc, 0.15)),
            );
            vars.insert("--color-quick-join-text".into(), acc_bright);
        }

        if let Some(ref warm) = accent_warm {
            let spark = format!("color-mix(in srgb, {warm} 60%, white)");
            vars.insert("--color-icon-amber".into(), warm.clone());
            vars.insert("--color-icon-orange".into(), warm.clone());
            vars.insert("--color-icon-spark".into(), spark.clone());
            vars.insert("--gradient-pill-active".into(), warm.clone());
            vars.insert(
                "--filter-pill-active".into(),
                format!("drop-shadow(0 0 6px {})", with_alpha(warm, 0.65)),
            );
            vars.insert(
                "--shadow-server-icon-active".into(),
                format!("0 0 0 2.5px {warm}, 0 0 16px {}", with_alpha(warm, 0.4)),
            );
            vars.insert(
                "--shadow-home-btn-active".into(),
                format!("0 0 0 2.5px {warm}, 0 0 18px {}", with_alpha(warm, 0.45)),
            );
            vars.insert("--color-home-badge-border".into(), warm.clone());
            vars.insert("--color-home-badge-border-hover".into(), spark.clone());
            vars.insert("--color-channel-add-hover-text".into(), warm.clone());
            vars.insert("--color-channel-add-hover-bg".into(), with_alpha(warm, 0.16));
            vars.insert("--color-char-counter-border".into(), with_alpha(warm, 0.45));
            vars.insert("--color-char-counter-text".into(), spark.clone());
            vars.insert("--color-read-more-border".into(), with_alpha(warm, 0.4));
            vars.insert("--color-read-more-text".into(), warm.clone());
            vars.insert("--color-read-more-hover-border".into(), with_alpha(warm, 0.65));
            vars.insert("--color-read-more-hover-text".into(), spark.clone());
            vars.insert("--gradient-read-more".into(), with_alpha(warm, 0.16));
            vars.insert("--gradient-read-more-hover".into(), with_alpha(warm, 0.28));
            vars.insert("--color-editor-badge-border".into(), with_alpha(warm, 0.4));
            vars.insert("--color-editor-badge-text".into(), warm.clone());
            vars.insert("--gradient-editor-badge".into(), with_alpha(warm, 0.18));
            vars.insert("--color-server-new-hover-border".into(), warm.clone());
            vars.insert("--color-server-new-hover-text".into(), spark);
            vars.insert("--color-server-new-hover-bg".into(), with_alpha(warm, 0.18));
            vars.insert(
                "--shadow-server-new-hover".into(),
                format!("0 0 0 2px {}, 0 0 14px {}", with_alpha(warm, 0.55), with_alpha(warm, 0.35)),
            );
        }

        if let (Some(acc), Some(pr)) = (&accent, &primary) {
            let warm = accent_warm.as_ref().unwrap_or(pr);
            vars.insert(
                "--gradient-server-separator".into(),
                format!(
                    "linear-gradient(90deg, {}, {}, {})",
                    with_alpha(acc, 0.45),
                    with_alpha(pr, 0.55),
                    with_alpha(warm, 0.45)
                ),
            );
            vars.insert(
                "--gradient-footer-top-line".into(),
                format!(
                    "linear-gradient(90deg, transparent 0%, {} 25%, {} 55%, {} 85%, transparent 100%)",
                    with_alpha(acc, 0.5),
                    with_alpha(pr, 0.55),
                    with_alpha(warm, 0.5)
                ),
            );
            vars.insert(
                "--gradient-date-divider".into(),
                format!(
                    "linear-gradient(90deg, transparent 0%, {} 22%, {} 50%, {} 78%, transparent 100%)",
                    with_alpha(acc, 0.35),
                    with_alpha(pr, 0.45),
                    with_alpha(warm, 0.35)
                ),
            );
            vars.insert(
                "--gradient-hero-banner".into(),
                format!(
                    "linear-gradient(135deg, {} 0%, {} 50%, {} 100%)",
                    with_alpha(acc, 0.2),
                    with_alpha(pr, 0.22),
                    with_alpha(warm, 0.18)
                ),
            );
            vars.insert("--color-hero-banner-border".into(), with_alpha(acc, 0.3));
        }

        if let Some(ref pr) = primary {
            let acc = accent.as_ref().unwrap_or(pr);
            let pr_end = primary_end.as_ref().unwrap_or(pr);
            let warm = accent_warm.as_ref().unwrap_or(acc);
            vars.insert(
                "--gradient-aurora-orb-1".into(),
                format!("radial-gradient(circle, {acc} 0%, {pr} 40%, transparent 70%)"),
            );
            vars.insert(
                "--gradient-aurora-orb-2".into(),
                format!("radial-gradient(circle, {pr} 0%, {pr_end} 45%, transparent 70%)"),
            );
            vars.insert(
                "--gradient-aurora-orb-3".into(),
                format!("radial-gradient(circle, {warm} 0%, transparent 65%)"),
            );
        }


        if let Some(ref c) = text_heading {
            vars.insert("--color-text-heading".into(), c.clone());
            vars.insert("--color-text-white".into(), c.clone());
        }

        if let Some(ref c) = text_chat {
            vars.insert("--color-text-chat".into(), c.clone());
            vars.insert("--color-text-member".into(), c.clone());
            vars.insert("--color-text-social".into(), c.clone());
        }

        if let Some(ref c) = text_muted {
            vars.insert("--color-text-muted".into(), c.clone());
            vars.insert("--color-text-label".into(), c.clone());
            vars.insert("--color-text-channel".into(), c.clone());
            vars.insert("--color-text-tab".into(), c.clone());
            vars.insert(
                "--color-text-dim".into(),
                format!("color-mix(in srgb, {c} 75%, black)"),
            );
        }

        if let Some(ref c) = text_author {
            vars.insert("--color-text-author".into(), c.clone());
            vars.insert("--color-text-date-badge".into(), c.clone());
            vars.insert(
                "--color-text-author-hover".into(),
                format!("color-mix(in srgb, {c} 75%, white)"),
            );
        }

        if let Some(ref c) = online {
            vars.insert("--color-status-online".into(), c.clone());
            vars.insert("--color-success".into(), c.clone());
            vars.insert("--color-success-alpha-20".into(), with_alpha(c, 0.2));
        }

        if let Some(ref c) = idle {
            vars.insert("--color-status-idle".into(), c.clone());
        }

        if let Some(ref c) = danger {
            vars.insert("--color-danger".into(), c.clone());
            vars.insert("--color-error".into(), c.clone());
            vars.insert("--color-status-dnd".into(), c.clone());
            vars.insert("--color-icon-coral".into(), c.clone());
            vars.insert(
                "--color-danger-hover".into(),
                format!("color-mix(in srgb, {c} 85%, black)"),
            );
            vars.insert(
                "--color-danger-light".into(),
                format!("color-mix(in srgb, {c} 72%, white)"),
            );
            vars.insert("--color-error-alpha-15".into(), with_alpha(c, 0.15));
            vars.insert("--color-danger-alpha-08".into(), with_alpha(c, 0.08));
            vars.insert("--color-danger-alpha-15".into(), with_alpha(c, 0.15));
            vars.insert("--color-danger-alpha-18".into(), with_alpha(c, 0.18));
            vars.insert("--color-danger-alpha-40".into(), with_alpha(c, 0.40));
            vars.insert("--color-danger-alpha-75".into(), with_alpha(c, 0.75));
            vars.insert("--gradient-unread-badge".into(), c.clone());
            vars.insert(
                "--shadow-unread-badge".into(),
                format!("0 2px 8px {}", with_alpha(c, 0.45)),
            );
            vars.insert("--color-quick-create-border".into(), with_alpha(c, 0.5));
            vars.insert(
                "--shadow-quick-create".into(),
                format!("0 6px 20px {}", with_alpha(c, 0.15)),
            );
            vars.insert("--color-quick-create-text".into(), c.clone());
        }

        for (raw_key, raw_val) in &self.variables {
            let key = raw_key.trim();
            let val = raw_val.trim();
            if is_allowed_css_var(key) && is_safe_css_value(val) {
                vars.insert(key.to_string(), val.to_string());
            }
        }

        if !vars.is_empty() {
            vars.entry("--color-text-on-accent".into())
                .or_insert_with(|| "#ffffff".into());
        }

        vars
    }

    pub fn to_active_css(&self) -> String {
        let vars = self.build_variable_map();
        let mut out = String::from(":root {\n");
        for (k, v) in vars {
            out.push_str(&format!("    {k}: {v};\n"));
        }
        out.push_str("}\n");
        out
    }

    pub fn to_preview_css(&self) -> String {
        let safe_id = sanitize_id(&self.id);
        let p = &self.palette;
        let pick = |opt: &Option<String>, fallback: &str| -> String {
            opt.as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty() && is_safe_css_value(s))
                .unwrap_or(fallback)
                .to_string()
        };

        let bg_sidebar = pick(&p.bg_sidebar, "#18191c");
        let bg_channels = pick(&p.bg_channels, "#2f3136");
        let bg_chat = pick(&p.bg_chat, "#36393f");
        let bg_input = pick(&p.bg_input, "#40444b");
        let primary = pick(&p.primary, "#a78bfa");
        let primary_end = pick(&p.primary_end, &primary);
        let accent = pick(&p.accent, &primary);
        let online = pick(&p.online, "#43b581");
        let danger = pick(&p.danger, "#f04747");

        format!(
            ".theme-preview-card[data-custom-theme=\"{safe_id}\"] {{\n\
                --preview-bg-sidebar: {bg_sidebar};\n\
                --preview-bg-channels: {bg_channels};\n\
                --preview-bg-chat: {bg_chat};\n\
                --preview-bg-input: {bg_input};\n\
                --preview-primary: {primary};\n\
                --preview-primary-end: {primary_end};\n\
                --preview-accent: {accent};\n\
                --preview-online: {online};\n\
                --preview-danger: {danger};\n\
                --preview-badge-bg: {};\n\
                --preview-badge-border: {};\n\
            }}",
            with_alpha(&primary, 0.18),
            with_alpha(&primary, 0.4),
        )
    }
}

fn sanitize_id(input: &str) -> String {
    input
        .trim()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c.to_ascii_lowercase()
            } else if c.is_whitespace() {
                '-'
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches(|c| c == '-' || c == '_')
        .to_string()
}

fn is_allowed_css_var(key: &str) -> bool {
    let valid_prefix = key.starts_with("--color-")
        || key.starts_with("--gradient-")
        || key.starts_with("--shadow-")
        || key.starts_with("--filter-");
    valid_prefix
        && key.len() <= 64
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
}

fn is_safe_css_value(val: &str) -> bool {
    if val.is_empty() || val.len() > 400 {
        return false;
    }
    let lower = val.to_ascii_lowercase();
    !val.contains('{')
        && !val.contains('}')
        && !val.contains(';')
        && !val.contains('<')
        && !val.contains('>')
        && ! val.contains('\\')
        && !lower.contains("url(")
        && !lower.contains("expression(")
        && !lower.contains("javascript:")
}

fn parse_hex_rgb(color: &str) -> Option<(u8, u8, u8)> {
    let hex = color.trim().strip_prefix('#')?;
    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
            Some((r, g, b))
        }
        6 | 8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some((r, g, b))
        }
        _ => None,
    }
}

fn with_alpha(color: &str, alpha: f32) -> String {
    if let Some((r, g, b)) = parse_hex_rgb(color) {
        format!("rgba({r}, {g}, {b}, {alpha:.2})")
    } else {
        let pct = (alpha * 100.0).round() as u32;
        format!("color-mix(in srgb, {color} {pct}%, transparent)")
    }
}

fn set_style_tag(doc: &web_sys::Document, id: &str, css: &str) {
    if let Some(existing) = doc.get_element_by_id(id) {
        existing.set_text_content(Some(css));
        return;
    }
    if css.is_empty() {
        return;
    }
    if let Ok(style_el) = doc.create_element("style") {
        let _ = style_el.set_attribute("id", id);
        style_el.set_text_content(Some(css));
        if let Some(root) = doc.document_element() {
            let _ = root.append_child(&style_el);
        }
    }
}

pub fn download_json_file(filename: &str, content: &str) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(doc) = window.document() else {
        return;
    };
    let encoded = js_sys::encode_uri_component(content);
    let data_url = format!("data:application/json;charset=utf-8,{encoded}");
    if let Ok(el) = doc.create_element("a") {
        let _ = el.set_attribute("href", &data_url);
        let _ = el.set_attribute("download", filename);
        if let Ok(html_el) = el.dyn_into::<web_sys::HtmlElement>() {
            html_el.click();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_and_starter_themes_parse_and_generate_css() {
        let tpl = CustomTheme::template_json();
        let parsed = CustomTheme::from_json_str(&tpl).expect("template JSON should be valid");
        assert_eq!(parsed.id, "my-custom-theme");
        let css = parsed.to_active_css();
        assert!(css.contains("--color-bg-panel:"));
        assert!(css.contains("--color-primary:"));

        for starter in CustomTheme::catalog_themes() {
            let json = starter.to_pretty_json();
            let roundtrip = CustomTheme::from_json_str(&json).expect("catalog theme roundtrip");
            assert_eq!(roundtrip.id, starter.id);
            assert!(!roundtrip.to_active_css().is_empty());
            assert!(!roundtrip.to_preview_css().is_empty());
        }
    }

    #[test]
    fn rejects_unsafe_css_values_and_invalid_json() {
        let bad_json = r#"{
            "id": "evil",
            "name": "Evil Theme",
            "variables": {
                "--color-primary": "red; } body { display: none"
            }
        }"#;
        assert!(CustomTheme::from_json_str(bad_json).is_err());
    }
}

