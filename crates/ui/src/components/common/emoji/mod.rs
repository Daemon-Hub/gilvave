//! Emoji packs, URL generator, text segmenter, and contenteditable DOM synchronization.

mod editor_dom;
pub mod pack;
#[cfg(test)]
mod tests;
mod twemoji;

pub use editor_dom::{
    focus_editor_at_end, insert_emoji_into_editor, insert_text_into_editor,
    serialize_contenteditable, sync_editor_content,
};
#[allow(unused_imports)]
pub use pack::{
    emoji_to_codepoint, emoji_to_pack_url, handle_emoji_img_error, EmojiPack,
    ALL_EMOJI_PACKS, STORAGE_KEY_EMOJI_PACK,
};
#[allow(unused_imports)]
pub use twemoji::{
    emoji_to_twemoji_url, parse_message_segments, parse_message_segments_default, MessageSegment,
};
