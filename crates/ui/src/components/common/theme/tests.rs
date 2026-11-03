//! Unit tests for theme parsing, template validity, and CSS injection security.

use super::*;

#[test]
fn template_and_starter_themes_parse_and_generate_css() {
    let tpl = CustomTheme::template_json();
    let parsed = CustomTheme::from_json_str(&tpl).expect("template JSON should be valid");
    assert_eq!(parsed.id, "my-custom-theme");
    let css = parsed.to_active_css();
    assert!(css.contains("--color-bg-panel:"));
    assert!(css.contains("--color-primary:"));

    let server_theme = gilvave_core::dto::users::Theme {
        name: "Server Dark".to_string(),
        author: "Gilvave".to_string(),
        description: "A dark theme from server".to_string(),
        config: serde_json::json!({
            "id": "server-dark",
            "base": "standard",
            "palette": {
                "primary": "#89b4fa"
            }
        }),
        created_at: time::OffsetDateTime::UNIX_EPOCH,
        updated_at: time::OffsetDateTime::UNIX_EPOCH,
    };
    let converted: CustomTheme = server_theme.into();
    assert_eq!(converted.name, "Server Dark");
    assert_eq!(converted.author, "Gilvave");
    assert_eq!(converted.palette.primary.as_deref(), Some("#89b4fa"));
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
