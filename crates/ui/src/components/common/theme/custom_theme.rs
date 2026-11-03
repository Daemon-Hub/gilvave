//! User-installed and community JSON themes with automatic derivation of 120+ CSS variables.

use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

use super::{STORAGE_KEY_CUSTOM_THEMES, STYLE_TAG_CUSTOM_PREVIEWS};
use super::css_engine::{is_safe_css_value, sanitize_id, set_style_tag, with_alpha};


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
    #[serde(default, skip_serializing_if = "String::is_empty")]
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
        let mut theme: Self =
            serde_json::from_str(raw).map_err(|e| format!("Ошибка синтаксиса JSON: {e}"))?;

        theme.id = sanitize_id(&theme.id);
        theme.name = theme.name.trim().to_string();
        if theme.name.is_empty() {
            return Err("Поле \"name\" (название темы) не может быть пустым".to_string());
        }
        if theme.id.is_empty() {
            theme.id = sanitize_id(&theme.name);
        }
        if theme.id.is_empty()
            || theme.id == "standard"
            || theme.id == "advanced"
            || theme.id == "light"
        {
            #[cfg(target_arch = "wasm32")]
            {
                theme.id = format!("custom-{}", js_sys::Date::now() as u64);
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                theme.id = "custom-user-theme".to_string();
            }
        }
        theme.author = theme.author.trim().to_string();
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
        if let Some(window) = web_sys::window()
            && let Ok(Some(storage)) = window.local_storage()
            && let Ok(json) = serde_json::to_string(themes)
        {
            let _ = storage.set_item(STORAGE_KEY_CUSTOM_THEMES, &json);
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
        vec![]
    }

    pub fn template_json() -> String {
        let template = Self {
            id: "my-custom-theme".to_string(),
            name: "My Custom Theme".to_string(),
            author: String::new(),
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
            variables: BTreeMap::from([(
                "--gradient-send-btn".to_string(),
                "linear-gradient(135deg, #58a6ff, #bc8cff)".to_string(),
            )]),
        };
        template.to_pretty_json()
    }

    pub fn build_variable_map(&self) -> BTreeMap<String, String> {
        super::palette_vars::build_palette_variable_map(&self.palette, &self.variables)
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

impl From<gilvave_core::dto::users::Theme> for CustomTheme {
    fn from(t: gilvave_core::dto::users::Theme) -> Self {
        if let Ok(mut c) = serde_json::from_value::<CustomTheme>(t.config.clone()) {
            if c.name.is_empty() {
                c.name = t.name;
            }
            if c.author.is_empty() {
                c.author = t.author;
            }
            if c.description.is_empty() {
                c.description = t.description;
            }
            if c.id.is_empty() {
                c.id = sanitize_id(&c.name);
            }
            return c;
        }

        #[derive(Deserialize)]
        struct PartialConfig {
            #[serde(default)]
            id: Option<String>,
            #[serde(default = "default_version")]
            version: String,
            #[serde(default = "default_base")]
            base: String,
            #[serde(default)]
            palette: ThemePalette,
            #[serde(default)]
            variables: BTreeMap<String, String>,
        }

        if let Ok(p) = serde_json::from_value::<PartialConfig>(t.config) {
            let id =
                p.id.filter(|s| !s.is_empty())
                    .unwrap_or_else(|| sanitize_id(&t.name));
            CustomTheme {
                id,
                name: t.name,
                author: t.author,
                version: p.version,
                description: t.description,
                base: p.base,
                palette: p.palette,
                variables: p.variables,
            }
        } else {
            CustomTheme {
                id: sanitize_id(&t.name),
                name: t.name,
                author: t.author,
                version: default_version(),
                description: t.description,
                base: default_base(),
                palette: ThemePalette::default(),
                variables: BTreeMap::new(),
            }
        }
    }
}


