//! Message content validation, sanitization, BiDi override removal, and Zalgo capping.

use super::unicode::{is_bidi_override, is_combining_mark};

/// Максимальная допустимая длина сообщения чата в символах
pub const MAX_MESSAGE_CHARS: usize = 16_384;

/// Порог длины сообщения, после которого текст сворачивается под кнопку «Далее»
pub const MESSAGE_COLLAPSE_CHARS: usize = 1_024;

/// Порог длины набираемого сообщения, начиная с которого над полем ввода появляется плавающий счётчик символов
pub const MESSAGE_COUNTER_VISIBLE_CHARS: usize = 4_096;

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
