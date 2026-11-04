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

/// Подсчитывает количество реальных напечатанных пользователем символов в сообщении,
/// исключая технические URL вложений (data-URL, S3-ссылки, file-ID).
pub fn count_typed_chars(content: &str) -> usize {
    let mut count = 0;
    let chars: Vec<char> = content.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // Проверяем начало изображения ![alt](url)
        if chars[i] == '!' && i + 1 < len && chars[i + 1] == '[' {
            let mut j = i + 2;
            while j < len && chars[j] != ']' {
                j += 1;
            }
            if j < len && chars[j] == ']' && j + 1 < len && chars[j + 1] == '(' {
                let mut k = j + 2;
                while k < len && chars[k] != ')' {
                    k += 1;
                }
                if k < len && chars[k] == ')' {
                    // Изображения не считаются за напечатанный текст
                    i = k + 1;
                    continue;
                }
            }
        }

        // Проверяем ссылку на файл [name](file:id) или [name](url)
        if chars[i] == '[' {
            let mut j = i + 1;
            while j < len && chars[j] != ']' {
                j += 1;
            }
            if j < len && chars[j] == ']' && j + 1 < len && chars[j + 1] == '(' {
                let mut k = j + 2;
                while k < len && chars[k] != ')' {
                    k += 1;
                }
                if k < len && chars[k] == ')' {
                    let link_target: String = chars[j + 2..k].iter().collect();
                    if link_target.starts_with("file:")
                        || link_target.starts_with("data:")
                        || link_target.contains("/files/")
                    {
                        // Учитываем видимое имя файла [name], но пропускаем технический URL вложения
                        count += j - (i + 1);
                        i = k + 1;
                        continue;
                    }
                }
            }
        }

        // Проверяем прямые data: URL (например, data:image/...;base64,...)
        if i + 5 <= len {
            let slice: String = chars[i..i + 5].iter().collect();
            if slice == "data:" {
                while i < len && !chars[i].is_whitespace() && chars[i] != ')' && chars[i] != '"' {
                    i += 1;
                }
                continue;
            }
        }

        count += 1;
        i += 1;
    }

    count
}

/// Валидирует содержимое сообщения чата:
/// - Очищает от вредоносных эксплойтов Unicode
/// - Отклоняет визуально пустые сообщения (состоящие только из пробелов или удалённых скрытых символов)
/// - Ограничивает максимальную длину напечатанного сообщения (не более 16384 символов)
pub fn validate_message(content: &str) -> Result<String, &'static str> {
    let sanitized = sanitize_message(content);
    let trimmed = sanitized.trim();

    if trimmed.is_empty() {
        return Err("Сообщение не может быть пустым");
    }

    if count_typed_chars(trimmed) > MAX_MESSAGE_CHARS {
        return Err("Сообщение слишком длинное (максимум 16384 символов)");
    }

    Ok(sanitized)
}
