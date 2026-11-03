//! Modular theme engine: built-in palettes, custom JSON themes, CSS variables derivation, and persistent preferences.

pub mod app_theme;
pub mod css_engine;
pub mod custom_theme;
pub mod palette_vars;
pub mod preferences;

#[cfg(test)]
mod tests;

pub(crate) const STORAGE_KEY_ACTIVE_THEME: &str = "gilvave_theme";
pub(crate) const STORAGE_KEY_CUSTOM_THEMES: &str = "gilvave_installed_custom_themes";
pub(crate) const STORAGE_KEY_WINDOWED_MODE: &str = "gilvave_windowed_mode";
pub(crate) const STORAGE_KEY_CLOSE_TO_TRAY: &str = "gilvave_close_to_tray";
pub(crate) const STORAGE_KEY_SHOW_GREETING: &str = "gilvave_show_greeting";
pub(crate) const STORAGE_KEY_SHOW_QUICK_ACTIONS: &str = "gilvave_show_quick_actions";
pub(crate) const STYLE_TAG_ACTIVE_CUSTOM: &str = "gilvave-custom-theme";
pub(crate) const STYLE_TAG_CUSTOM_PREVIEWS: &str = "gilvave-custom-theme-previews";

pub use app_theme::AppTheme;
pub use css_engine::download_json_file;
#[allow(unused_imports)]
pub use custom_theme::{CustomTheme, ThemePalette};
pub use preferences::{
    load_close_to_tray, load_show_greeting, load_show_quick_actions, load_windowed_mode,
    save_close_to_tray, save_show_greeting, save_show_quick_actions, save_windowed_mode,
};
