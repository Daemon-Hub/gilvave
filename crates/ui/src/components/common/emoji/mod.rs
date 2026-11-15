//! Emoji packs, URL generator, text segmenter, and contenteditable DOM synchronization.

mod editor_dom;
pub mod pack;
#[cfg(test)]
mod tests;
mod twemoji;

#[allow(unused_imports)]
pub use editor_dom::{
    focus_editor_at_end, insert_emoji_into_editor, insert_image_into_editor,
    insert_text_into_editor, serialize_contenteditable, sync_editor_content,
};
#[allow(unused_imports)]
pub use pack::{
    ALL_EMOJI_PACKS, EmojiPack, STORAGE_KEY_EMOJI_PACK, emoji_to_codepoint, emoji_to_pack_url,
    handle_emoji_img_error,
};
#[allow(unused_imports)]
pub use twemoji::{
    MessageSegment, emoji_to_twemoji_url, parse_message_segments, parse_message_segments_default,
};
