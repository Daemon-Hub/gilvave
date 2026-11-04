//! Модуль безопасности и валидации Unicode для Gilvave.
//! Защищает от:
//! - Zalgo / избыточных комбинируемых диакритических знаков
//! - Невидимых символов и символов нулевой ширины (ZWSP, BOM, мягкий перенос, теги)
//! - Атак спуфинга направления текста BiDi (LRE, RLE, RLO, LRO, изоляты)
//! - Атак с омоглифами и смешиванием алфавитов (например, кириллическая 'А' вместо латинской 'A' в 'Аdmin')

pub mod message;
pub mod unicode;
pub mod username;

#[cfg(test)]
mod tests;

pub use message::{
    MAX_MESSAGE_CHARS, MESSAGE_COLLAPSE_CHARS, MESSAGE_COUNTER_VISIBLE_CHARS,
    count_typed_chars, sanitize_message, validate_message,
};
pub use unicode::{
    is_allowed_username_symbol, is_bidi_override, is_combining_mark, is_cyrillic_letter,
    is_greek_letter, is_invisible_char, is_latin_letter,
};
pub use username::validate_username;
