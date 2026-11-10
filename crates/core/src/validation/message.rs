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
    let bytes = content.as_bytes();
    let is_all_ascii = {
        let (chunks, rem) = bytes.as_chunks::<8>();
        let mut has_high = false;
        for chunk in chunks {
            let word = u64::from_le_bytes(*chunk);
            if (word & 0x8080_8080_8080_8080) != 0 {
                has_high = true;
                break;
            }
        }
        if !has_high {
            for &b in rem {
                if b >= 0x80 {
                    has_high = true;
                    break;
                }
            }
        }
        !has_high
    };

    if is_all_ascii {
        return content.to_string();
    }

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
    let bytes = content.as_bytes();
    let len = bytes.len();
    let mut count = 0;
    let mut i = 0;

    while i < len {
        // Проверяем начало изображения ![alt](url)
        if bytes[i] == b'!' && i + 1 < len && bytes[i + 1] == b'[' {
            let mut j = i + 2;
            while j < len && bytes[j] != b']' {
                j += 1;
            }
            if j < len && bytes[j] == b']' && j + 1 < len && bytes[j + 1] == b'(' {
                let mut k = j + 2;
                while k < len && bytes[k] != b')' {
                    k += 1;
                }
                if k < len && bytes[k] == b')' {
                    // Изображения не считаются за напечатанный текст
                    i = k + 1;
                    continue;
                }
            }
        }

        // Проверяем ссылку на файл [name](file:id) или [name](url)
        if bytes[i] == b'[' {
            let mut j = i + 1;
            while j < len && bytes[j] != b']' {
                j += 1;
            }
            if j < len && bytes[j] == b']' && j + 1 < len && bytes[j + 1] == b'(' {
                let mut k = j + 2;
                while k < len && bytes[k] != b')' {
                    k += 1;
                }
                if k < len && bytes[k] == b')' {
                    let link_target = &bytes[j + 2..k];
                    if link_target.starts_with(b"file:")
                        || link_target.starts_with(b"data:")
                        || link_target.windows(7).any(|w| w == b"/files/")
                    {
                        // Учитываем видимое имя файла [name], но пропускаем технический URL вложения
                        let name_bytes = &bytes[i + 1..j];
                        for &b in name_bytes {
                            if (b & 0xC0) != 0x80 {
                                count += 1;
                            }
                        }
                        i = k + 1;
                        continue;
                    }
                }
            }
        }

        // Проверяем прямые data: URL (например, data:image/...;base64,...)
        if i + 5 <= len && &bytes[i..i + 5] == b"data:" {
            while i < len
                && bytes[i] != b' '
                && bytes[i] != b'\t'
                && bytes[i] != b'\n'
                && bytes[i] != b'\r'
                && bytes[i] != b')'
                && bytes[i] != b'"'
            {
                i += 1;
            }
            continue;
        }

        // SWAR fast-path: scan 8 bytes of plain ASCII text in a single step
        if i + 8 <= len {
            let chunk: [u8; 8] = bytes[i..i + 8].try_into().unwrap_or_default();
            let word = u64::from_le_bytes(chunk);
            let has_high_bit = (word & 0x8080_8080_8080_8080) != 0;

            let xor_excl = word ^ 0x2121_2121_2121_2121;
            let has_excl = (xor_excl.wrapping_sub(0x0101_0101_0101_0101) & !xor_excl & 0x8080_8080_8080_8080) != 0;

            let xor_brack = word ^ 0x5B5B_5B5B_5B5B_5B5B;
            let has_brack = (xor_brack.wrapping_sub(0x0101_0101_0101_0101) & !xor_brack & 0x8080_8080_8080_8080) != 0;

            let xor_d = word ^ 0x6464_6464_6464_6464;
            let has_d = (xor_d.wrapping_sub(0x0101_0101_0101_0101) & !xor_d & 0x8080_8080_8080_8080) != 0;

            if !has_high_bit && !has_excl && !has_brack && !has_d {
                count += 8;
                i += 8;
                continue;
            }
        }

        if (bytes[i] & 0xC0) != 0x80 {
            count += 1;
        }
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
