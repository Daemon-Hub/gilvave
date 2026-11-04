//! Unit tests for Unicode security and validation functions.

use super::*;

#[test]
fn test_valid_usernames() {
    assert!(validate_username("Alex").is_ok());
    assert!(validate_username("Дмитрий_123").is_ok());
    assert!(validate_username("user-name").is_ok());
    assert!(validate_username("john.doe").is_ok());
    assert!(validate_username("Jose\u{0301}").is_ok()); // 1 диакритический знак (e с акутом)
    assert!(validate_username("René").is_ok());
}

#[test]
fn test_invalid_length() {
    assert!(validate_username("").is_err());
    assert!(validate_username("a").is_err());
    assert!(validate_username(&"a".repeat(33)).is_err());
}

#[test]
fn test_spaces_rejection() {
    assert!(validate_username("user name").is_err());
    assert!(validate_username("user\tname").is_err());
    assert!(validate_username("user\nname").is_err());
}

#[test]
fn test_invisible_chars_rejection() {
    // Пробел нулевой ширины
    assert!(validate_username("admin\u{200B}").is_err());
    // BOM
    assert!(validate_username("admin\u{FEFF}").is_err());
    // Мягкий перенос
    assert!(validate_username("ad\u{00AD}min").is_err());
    // Соединитель слов
    assert!(validate_username("ad\u{2060}min").is_err());
}

#[test]
fn test_bidi_override_rejection() {
    // RLO (переворот текста справа налево)
    assert!(validate_username("\u{202E}nimdA").is_err());
    // LRI
    assert!(validate_username("user\u{2066}name").is_err());
}

#[test]
fn test_zalgo_rejection() {
    let zalgo = "ž̵̓";
    assert!(validate_username(zalgo).is_err());
    assert!(validate_username("a\u{0300}\u{0301}\u{0302}").is_err());
}

#[test]
fn test_mixed_script_homoglyph_rejection() {
    // Кириллическая 'А' (U+0410) с латинскими 'dmin'
    assert!(validate_username("Аdmin").is_err());
    // Латинская 'p' с кириллическими 'аypаl'
    assert!(validate_username("pаypаl").is_err());
    // Чистая кириллица разрешена
    assert!(validate_username("Админ").is_ok());
    // Чистая латиница разрешена
    assert!(validate_username("Admin").is_ok());
}

#[test]
fn test_mathematical_alphanumeric_font_rejection() {
    // Математический жирный шрифт A-d-m-i-n (𝔄𝔡𝔪𝔦𝔫 / 𝐀𝐝𝐦𝐢𝐧)
    assert!(validate_username("𝐀𝐝𝐦𝐢𝐧").is_err());
    // Полноширинная латиница (Ａｄｍｉｎ)
    assert!(validate_username("Ａｄｍｉｎ").is_err());
}

#[test]
fn test_sanitize_message_bidi() {
    // Атака Trojan Source / подмена URL через переворот BiDi
    let malicious = "Check this: \u{202E}moc.elpmaxe//:sptth\u{202C} now!";
    let sanitized = sanitize_message(malicious);
    assert!(!sanitized.contains('\u{202E}'));
    assert!(!sanitized.contains('\u{202C}'));
    assert_eq!(sanitized, "Check this: moc.elpmaxe//:sptth now!");
}

#[test]
fn test_sanitize_message_invisible() {
    let text = "Hello\u{200B}\u{FEFF}\u{00AD} World\u{2060}!";
    let sanitized = sanitize_message(text);
    assert_eq!(sanitized, "Hello World!");
}

#[test]
fn test_sanitize_message_zalgo_capping() {
    // 6 диакритических знаков на букву H
    let zalgo = "H\u{0300}\u{0301}\u{0302}\u{0303}\u{0304}\u{0305}ello";
    let sanitized = sanitize_message(zalgo);
    // Должен остаться базовый символ и не более 2 диакритических знаков
    assert_eq!(sanitized, "H\u{0300}\u{0301}ello");
}

#[test]
fn test_sanitize_message_preserves_multilingual_and_emojis() {
    let msg = "Привет, мир! Hello world! Ça va? René. Emojis: 👨‍👩‍👧 ❤️ 🚀 🎉";
    let sanitized = sanitize_message(msg);
    assert_eq!(sanitized, msg);
}

#[test]
fn test_validate_message_empty() {
    // Чистые пробельные символы
    assert!(validate_message("    ").is_err());
    // Чистые невидимые символы
    assert!(validate_message("\u{200B}\u{FEFF}\u{00AD}").is_err());
    assert!(validate_message("  \u{200B}  \u{FEFF}  ").is_err());
    // Корректное сообщение
    assert!(validate_message("Привет!").is_ok());
    // Сообщение допустимой длины разрешено
    assert!(validate_message(&"а".repeat(MAX_MESSAGE_CHARS)).is_ok());
    // Превышение лимита отклоняется
    assert!(validate_message(&"а".repeat(MAX_MESSAGE_CHARS + 1)).is_err());
}

#[test]
fn test_count_typed_chars_with_attachments() {
    // Сообщение только с картинкой (data-URL на 50 000 символов)
    let fake_data_url = format!("data:image/png;base64,{}", "A".repeat(50_000));
    let msg_img_only = format!("![screenshot.png]({fake_data_url})");
    assert_eq!(count_typed_chars(&msg_img_only), 0);
    assert!(validate_message(&msg_img_only).is_ok());

    // Сообщение с текстом и ссылкой на файл
    let msg_with_text = format!("Привет! ![test.png]({fake_data_url}) Как дела?");
    assert_eq!(count_typed_chars(&msg_with_text), 18); // "Привет!  Как дела?"
    assert!(validate_message(&msg_with_text).is_ok());

    // Ссылка на файл [doc.pdf](file:UUID)
    let msg_file = "[document.pdf](file:0192e2b3-8a4e-7b70-8000-123456789abc)";
    assert_eq!(count_typed_chars(msg_file), 12); // "document.pdf"
    assert!(validate_message(msg_file).is_ok());
}
