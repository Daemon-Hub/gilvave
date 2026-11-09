use pulldown_cmark::{Options, Parser, html};
use std::borrow::Cow;
use sycamore::prelude::*;
use sycamore::web::tags::div;
use sycamore::web::{GlobalProps, HtmlGlobalAttributes};

use crate::components::common::{EmojiPack, emoji_to_pack_url};

/// Рендеринг базового Markdown (для модальных окон, настроек и справок)
pub fn render_markdown(md: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    let parser = Parser::new_ext(md, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

/// Предобработка Markdown сообщения для сохранения переводов строк
fn preprocess_chat_markdown(md: &str) -> String {
    let mut out = String::with_capacity(md.len() + 64);
    let mut in_code_block = false;

    for line in md.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_code_block = !in_code_block;
            out.push_str(line);
            out.push('\n');
            continue;
        }

        out.push_str(line);
        if !in_code_block && !line.ends_with("  ") {
            out.push_str("  ");
        }
        out.push('\n');
    }

    out
}

/// Постобработка HTML сообщения для ссылок и эмодзи
fn postprocess_chat_html(raw_html: &str, default_pack: EmojiPack) -> String {
    let mut out = String::with_capacity(raw_html.len() + 128);
    let mut remaining = raw_html;
    let mut in_code_depth: usize = 0;

    while !remaining.is_empty() {
        if remaining.starts_with('<')
            && let Some(end_idx) = remaining.find('>')
        {
            let tag = &remaining[..=end_idx];
            let tag_lower = tag.to_ascii_lowercase();

            if tag_lower.starts_with("<code") || tag_lower.starts_with("<pre") {
                in_code_depth += 1;
                out.push_str(tag);
            } else if tag_lower.starts_with("</code") || tag_lower.starts_with("</pre") {
                in_code_depth = in_code_depth.saturating_sub(1);
                out.push_str(tag);
            } else if tag_lower.starts_with("<a ") {
                let mut a_tag = String::with_capacity(tag.len() + 40);
                if let Some(pos) = tag.find("<a ") {
                    a_tag.push_str(&tag[..pos + 3]);
                    a_tag.push_str("target=\"_blank\" rel=\"noopener noreferrer\" ");
                    a_tag.push_str(&tag[pos + 3..]);
                } else {
                    a_tag.push_str(tag);
                }
                out.push_str(&a_tag);
            } else {
                out.push_str(tag);
            }

            remaining = &remaining[end_idx + 1..];
            continue;
        }

        if in_code_depth > 0 {
            let next_tag = remaining.find('<').unwrap_or(remaining.len());
            out.push_str(&remaining[..next_tag]);
            remaining = &remaining[next_tag..];
            continue;
        }

        if remaining.starts_with("&lt;e:")
            && let Some(end_idx) = remaining.find("&gt;")
        {
            let inner = &remaining[6..end_idx];
            if let Some((pack_str, emoji_str)) = inner.split_once(':')
                && let Some(pack) = EmojiPack::try_from_id(pack_str)
            {
                let name = emojis::get(emoji_str)
                    .map(|e| e.name())
                    .unwrap_or(emoji_str);
                let url = emoji_to_pack_url(emoji_str, pack);
                use std::fmt::Write;
                let _ = write!(
                    out,
                    r#"<img class="chat-inline-emoji" src="{url}" alt="{emoji_str}" title="{name}" draggable="false" loading="lazy" />"#
                );
                remaining = &remaining[end_idx + 4..];
                continue;
            }
        }

        let Some(first_char) = remaining.chars().next() else {
            break;
        };

        let is_candidate = matches!(
            first_char,
            '#' | '*'
                | '0'..='9'
                | '\u{2000}'..='\u{3299}'
                | '\u{1F000}'..='\u{1FAFF}'
        );

        if is_candidate {
            let mut matched: Option<(&'static emojis::Emoji, usize)> = None;
            let mut byte_len = 0;
            let mut char_count = 0;

            for ch in remaining.chars() {
                byte_len += ch.len_utf8();
                char_count += 1;
                if byte_len > 32 || char_count > 12 {
                    break;
                }
                if let Some(emoji) = emojis::get(&remaining[..byte_len]) {
                    matched = Some((emoji, byte_len));
                }
            }

            if let Some((emoji, len)) = matched {
                let emoji_str = emoji.as_str();
                let url = emoji_to_pack_url(emoji_str, default_pack);
                let name = emoji.name();
                use std::fmt::Write;
                let _ = write!(
                    out,
                    r#"<img class="chat-inline-emoji" src="{url}" alt="{emoji_str}" title="{name}" draggable="false" loading="lazy" />"#
                );
                remaining = &remaining[len..];
                continue;
            }
        }

        out.push(first_char);
        remaining = &remaining[first_char.len_utf8()..];
    }

    out
}

/// Рендеринг Markdown сообщения с поддержкой ссылок и эмодзи
pub fn render_chat_markdown(md: &str, default_pack: EmojiPack) -> String {
    let preprocessed = preprocess_chat_markdown(md);

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    let parser = Parser::new_ext(&preprocessed, options);
    let mut html_raw = String::new();
    html::push_html(&mut html_raw, parser);

    postprocess_chat_html(&html_raw, default_pack)
}

/// Компонент отображения Markdown
#[component(inline_props)]
pub fn MarkdownView(content: String) -> View {
    let html_str = render_markdown(&content);
    let el = div()
        .class("markdown-rendered-view")
        .dangerously_set_inner_html(Cow::Owned(html_str));
    View::from(el)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_markdown_basic() {
        let md = "# Title\n\nSome **bold** text and `inline code`.";
        let html = render_markdown(md);
        assert!(html.contains("<h1>Title</h1>"));
        assert!(html.contains("<strong>bold</strong>"));
        assert!(html.contains("<code>inline code</code>"));
    }

    #[test]
    fn test_render_markdown_table() {
        let md = "| Key | Value |\n|---|---|\n| id | custom-theme |";
        let html = render_markdown(md);
        assert!(html.contains("<table>"));
        assert!(html.contains("<th>Key</th>"));
        assert!(html.contains("<td>id</td>"));
        assert!(html.contains("<td>custom-theme</td>"));
    }

    #[test]
    fn test_render_markdown_code_block() {
        let md = "```json\n{\n  \"name\": \"Test\"\n}\n```";
        let html = render_markdown(md);
        assert!(html.contains("<pre>"));
        assert!(html.contains("<code"));
        assert!(html.contains("\"name\": \"Test\""));
    }

    #[test]
    fn test_render_chat_markdown_formatting_and_emojis() {
        let md = "Привет **мир**!\nИ эмодзи: <e:apple:🍏> 😀";
        let html = render_chat_markdown(md, EmojiPack::Fluent3D);

        assert!(html.contains("<strong>мир</strong>"));
        assert!(html.contains("<br />"));
        assert!(html.contains("emoji-datasource-apple"));
        assert!(html.contains("fluent-emoji-3d"));
    }

    #[test]
    fn test_render_chat_markdown_code_preserves_plain_text() {
        let md = "```rust\nlet x = \"😀\";\n```";
        let html = render_chat_markdown(md, EmojiPack::Fluent3D);
        assert!(html.contains("<pre><code"));
        assert!(html.contains("let x = \"😀\";"));
        assert!(!html.contains("class=\"chat-inline-emoji\""));
    }

    #[test]
    fn test_render_chat_markdown_links() {
        let md = "Проверь сайт [Gilvave](https://gilvave.com)";
        let html = render_chat_markdown(md, EmojiPack::Fluent3D);
        assert!(html.contains("href=\"https://gilvave.com\""));
        assert!(html.contains("target=\"_blank\""));
        assert!(html.contains("rel=\"noopener noreferrer\""));
    }
}
