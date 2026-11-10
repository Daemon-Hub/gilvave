//! Username validation with UTS #39 Unicode security and homoglyph spoofing defense.

use super::unicode::{
    is_allowed_username_symbol, is_bidi_override, is_combining_mark, is_cyrillic_letter,
    is_greek_letter, is_invisible_char, is_latin_letter,
};

/// Строго валидирует имя пользователя согласно лучшим практикам безопасности идентификаторов Unicode (UTS #39).
/// - Должно содержать от 2 до 32 символов
/// - Без пробелов, управляющих символов, невидимых символов и BiDi-оверрайдов
/// - Контроль единого алфавита (запрещает смешивание латиницы и кириллицы/греческого для защиты от омоглифов)
/// - Защита от избыточных диакритических знаков (Zalgo)
/// - Допустимые символы: буквы (латиница или кириллица), цифры (0-9) и безопасные знаки ('_', '.', '-')
pub fn validate_username(username: &str) -> Result<(), &'static str> {
    let trimmed = username.trim();
    if trimmed.is_empty() {
        return Err("Имя пользователя не может быть пустым");
    }

    let mut char_count = 0;
    let mut has_latin = false;
    let mut has_cyrillic = false;
    let mut has_greek = false;
    let mut consecutive_combining = 0;
    let mut total_combining = 0;

    for c in trimmed.chars() {
        char_count += 1;
        if char_count > 32 {
            return Err("Имя пользователя должно содержать от 2 до 32 символов");
        }

        if c.is_control() {
            return Err("Имя пользователя содержит недопустимые управляющие символы");
        }
        if c.is_whitespace() {
            return Err("Имя пользователя не должно содержать пробелов");
        }
        if is_bidi_override(c) {
            return Err("Имя пользователя содержит недопустимые символы направления (BiDi)");
        }
        if is_invisible_char(c) {
            return Err("Имя пользователя содержит недопустимые невидимые символы");
        }

        if is_combining_mark(c) {
            total_combining += 1;
            consecutive_combining += 1;
            if consecutive_combining > 1 || total_combining > 2 {
                return Err("Имя пользователя содержит недопустимые комбинируемые символы (Zalgo)");
            }
        } else {
            consecutive_combining = 0;
        }

        if is_latin_letter(c) {
            has_latin = true;
        } else if is_cyrillic_letter(c) {
            has_cyrillic = true;
        } else if is_greek_letter(c) {
            has_greek = true;
        } else if c.is_ascii_digit() || is_allowed_username_symbol(c) {
            // Цифры и безопасные знаки нейтральны
        } else if !is_combining_mark(c) {
            return Err(
                "Имя пользователя может содержать только буквы, цифры и символы '_', '.', '-'",
            );
        }
    }

    if char_count < 2 {
        return Err("Имя пользователя должно содержать от 2 до 32 символов");
    }

    // Защита от спуфинга: обнаружение атак со смешиванием алфавитов (например, кириллическая А с латинскими dmin)
    let script_count = (has_latin as u8) + (has_cyrillic as u8) + (has_greek as u8);
    if script_count > 1 {
        return Err(
            "Смешивание разных алфавитов (латиницы и кириллицы) в имени пользователя запрещено (защита от омоглифов)",
        );
    }

    Ok(())
}
