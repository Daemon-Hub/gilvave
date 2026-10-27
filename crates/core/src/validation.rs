//! Модуль безопасности и валидации Unicode для Gilvave
//! Защищает от:
//! - Zalgo / избыточных комбинируемых диакритических знаков
//! - Невидимых символов и символов нулевой ширины (ZWSP, BOM, мягкий перенос, теги)
//! - Атак спуфинга направления текста BiDi (LRE, RLE, RLO, LRO, изоляты)
//! - Атак с омоглифами и смешиванием алфавитов (например, кириллическая 'А' вместо латинской 'A' в 'Аdmin')

#[inline]
pub fn is_bidi_override(c: char) -> bool {
    matches!(
        c,
        '\u{202A}'..='\u{202E}' // LRE, RLE, PDF, LRO, RLO (управление направлением BiDi)
            | '\u{2066}'..='\u{2069}' // LRI, RLI, FSI, PDI (изоляты BiDi)
            | '\u{061C}' // Метка арабской буквы
            | '\u{200E}' // Метка слева направо (LRM)
            | '\u{200F}' // Метка справа налево (RLM)
    )
}

#[inline]
pub fn is_invisible_char(c: char) -> bool {
    matches!(
        c,
        '\u{200B}' // Пробел нулевой ширины (ZWSP)
            | '\u{200C}' // Разделитель нулевой ширины (ZWNJ)
            | '\u{200D}' // Соединитель нулевой ширины (ZWJ)
            | '\u{FEFF}' // Неразрывный пробел нулевой ширины / BOM
            | '\u{00AD}' // Мягкий перенос (Soft Hyphen)
            | '\u{2060}' // Соединитель слов (Word Joiner)
            | '\u{180E}' // Разделитель монгольских гласных
            | '\u{E0000}'..='\u{E007F}' // Блок скрытых тегов
            | '\u{FFF0}'..='\u{FFFF}' // Блок специальных символов
    )
}

#[inline]
pub fn is_combining_mark(c: char) -> bool {
    matches!(
        c,
        '\u{0300}'..='\u{036F}'
            | '\u{1AB0}'..='\u{1AFF}'
            | '\u{1DC0}'..='\u{1DFF}'
            | '\u{20D0}'..='\u{20FF}'
            | '\u{FE20}'..='\u{FE2F}'
    )
}

#[inline]
pub fn is_latin_letter(c: char) -> bool {
    matches!(c, 'a'..='z' | 'A'..='Z' | '\u{00C0}'..='\u{024F}')
}

#[inline]
pub fn is_cyrillic_letter(c: char) -> bool {
    matches!(c, '\u{0400}'..='\u{04FF}' | '\u{0500}'..='\u{052F}')
}

#[inline]
pub fn is_greek_letter(c: char) -> bool {
    matches!(c, '\u{0370}'..='\u{03FF}')
}

#[inline]
pub fn is_allowed_username_symbol(c: char) -> bool {
    matches!(c, '_' | '.' | '-')
}

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

    let char_count = trimmed.chars().count();
    if !(2..=32).contains(&char_count) {
        return Err("Имя пользователя должно содержать от 2 до 32 символов");
    }

    let mut has_latin = false;
    let mut has_cyrillic = false;
    let mut has_greek = false;
    let mut consecutive_combining = 0;
    let mut total_combining = 0;

    for c in trimmed.chars() {
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

    // Защита от спуфинга: обнаружение атак со смешиванием алфавитов (например, кириллическая А с латинскими dmin)
    let script_count = (has_latin as u8) + (has_cyrillic as u8) + (has_greek as u8);
    if script_count > 1 {
        return Err(
            "Смешивание разных алфавитов (латиницы и кириллицы) в имени пользователя запрещено (защита от омоглифов)",
        );
    }

    Ok(())
}

/// Санитизирует текст сообщения чата:
/// 1. Вырезает символы переворота BiDi, которые могут инвертировать отображение и подменять ссылки
/// 2. Вырезает невидимые символы нулевой ширины (сохраняя корректные ZWJ и селекторы вариантов в эмодзи)
/// 3. Ограничивает количество комбинируемых диакритических знаков до максимум 2 на символ (нейтрализация Zalgo)
/// 4. Сохраняет естественный мультиязычный текст (кириллицу, латиницу, акценты, эмодзи, фрагменты кода)
pub fn sanitize_message(content: &str) -> String {
    let mut sanitized = String::with_capacity(content.len());
    let mut consecutive_combining = 0;
    let mut consecutive_zwj = 0;

    for c in content.chars() {
        // Немедленно отбрасываем BiDi-оверрайды
        if is_bidi_override(c) {
            continue;
        }

        // Отбрасываем отдельные невидимые символы
        if matches!(
            c,
            '\u{200B}' // Пробел нулевой ширины
                | '\u{FEFF}' // BOM / Неразрывный пробел нулевой ширины
                | '\u{00AD}' // Мягкий перенос
                | '\u{2060}' // Соединитель слов
                | '\u{180E}' // Разделитель монгольских гласных
                | '\u{E0000}'..='\u{E007F}' // Блок тегов
                | '\u{FFF0}'..='\u{FFFF}' // Блок спецсимволов
        ) {
            continue;
        }

        // Ограничиваем подряд идущие ZWJ (соединители нулевой ширины для эмодзи) максимум до 1
        if c == '\u{200D}' || c == '\u{200C}' {
            consecutive_zwj += 1;
            if consecutive_zwj > 1 {
                continue;
            }
            sanitized.push(c);
            continue;
        } else {
            consecutive_zwj = 0;
        }

        // Ограничиваем подряд идущие диакритические знаки максимум до 2 на базовый символ (нейтрализует Zalgo)
        if is_combining_mark(c) {
            consecutive_combining += 1;
            if consecutive_combining > 2 {
                continue; // отбрасываем 3-й и последующие диакритические знаки
            }
        } else {
            consecutive_combining = 0;
        }

        sanitized.push(c);
    }

    sanitized
}

/// Максимальная допустимая длина сообщения чата в символах
pub const MAX_MESSAGE_CHARS: usize = 16_384;

/// Порог длины сообщения, после которого текст сворачивается под кнопку «Далее»
pub const MESSAGE_COLLAPSE_CHARS: usize = 1_024;

/// Порог длины набираемого сообщения, начиная с которого над полем ввода появляется плавающий счётчик символов
pub const MESSAGE_COUNTER_VISIBLE_CHARS: usize = 4_096;

/// Валидирует содержимое сообщения чата:
/// - Очищает от вредоносных эксплойтов Unicode
/// - Отклоняет визуально пустые сообщения (состоящие только из пробелов или удалённых скрытых символов)
/// - Ограничивает максимальную длину сообщения (не более 16384 символов)
pub fn validate_message(content: &str) -> Result<String, &'static str> {
    let sanitized = sanitize_message(content);
    let trimmed = sanitized.trim();

    if trimmed.is_empty() {
        return Err("Сообщение не может быть пустым");
    }

    if trimmed.chars().count() > MAX_MESSAGE_CHARS {
        return Err("Сообщение слишком длинное (максимум 16384 символов)");
    }

    Ok(sanitized)
}

#[cfg(test)]
mod tests {
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
        // Сообщение длиной ровно 16 384 символа разрешено
        assert!(validate_message(&"а".repeat(16_384)).is_ok());
        // Превышение лимита 16 384 символа отклоняется
        assert!(validate_message(&"а".repeat(16_385)).is_err());
    }
}
