//! Unicode character classification and script categorization helpers for security validation.

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
