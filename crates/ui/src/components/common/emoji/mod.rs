//! Twemoji URL generator, text segmenter, and contenteditable DOM synchronization.

mod editor_dom;
#[cfg(test)]
mod tests;
mod twemoji;

pub use editor_dom::{
    focus_editor_at_end, insert_emoji_into_editor, insert_text_into_editor,
    serialize_contenteditable, sync_editor_content,
};
pub use twemoji::{emoji_to_twemoji_url, parse_message_segments, MessageSegment};
