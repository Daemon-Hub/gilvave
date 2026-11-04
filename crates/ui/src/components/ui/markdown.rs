use pulldown_cmark::{Options, Parser, html};
use std::borrow::Cow;
use sycamore::prelude::*;
use sycamore::web::tags::div;
use sycamore::web::{GlobalProps, HtmlGlobalAttributes};

use crate::components::common::{EmojiPack, emoji_to_pack_url};

/// Рендеринг базового Markdown
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

/// Предобработка Markdown сообщения
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

/// Извлечение значения атрибута
fn extract_attribute<'a>(tag: &'a str, attr: &str) -> Option<&'a str> {
    let pattern = format!("{attr}=\"");
    if let Some(pos) = tag.find(&pattern) {
        let after = &tag[pos + pattern.len()..];
        let end = after.find('"')?;
        return Some(&after[..end]);
    }
    let pattern_single = format!("{attr}='");
    if let Some(pos) = tag.find(&pattern_single) {
        let after = &tag[pos + pattern_single.len()..];
        let end = after.find('\'')?;
        return Some(&after[..end]);
    }
    None
}

/// Проверка на видео URL
fn is_video_url(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    lower.contains("/videos/")
        || lower.ends_with(".mp4")
        || lower.ends_with(".webm")
        || lower.ends_with(".mov")
        || lower.ends_with(".ogg")
        || lower.ends_with(".ogv")
        || lower.ends_with(".avi")
        || lower.ends_with(".mkv")
        || lower.ends_with(".m4v")
}

/// Постобработка HTML сообщения
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
            } else if tag_lower.starts_with("<img") {
                if tag.contains("chat-inline-emoji") {
                    out.push_str(tag);
                } else if let Some(src) = extract_attribute(tag, "src")
                    && is_video_url(src)
                {
                    use std::fmt::Write;
                    let alt = extract_attribute(tag, "alt").unwrap_or("video");
                    let _ = write!(
                        out,
                        r#"<div class="chat-message-video-wrapper"><video class="chat-message-video" controls preload="metadata" title="{alt}"><source src="{src}" />Ваш браузер не поддерживает видео.</video></div>"#
                    );
                } else {
                    let mut img_tag = String::with_capacity(tag.len() + 64);
                    if let Some(pos) = tag.find("<img") {
                        img_tag.push_str(&tag[..pos + 4]);
                        img_tag.push_str(
                            " class=\"chat-message-image\" loading=\"lazy\" decoding=\"async\"",
                        );
                        img_tag.push_str(&tag[pos + 4..]);
                    } else {
                        img_tag.push_str(tag);
                    }
                    out.push_str(&img_tag);
                }
            } else if tag_lower.starts_with("<a ") {
                let href = extract_attribute(tag, "href").unwrap_or_default();
                let is_file_link = href.starts_with("file:")
                    || href.starts_with("file_id:")
                    || href.contains("/files/");

                if is_file_link {
                    let after_tag = &remaining[end_idx + 1..];
                    if let Some(close_pos) = after_tag.find("</a>") {
                        let filename = &after_tag[..close_pos];
                        let clean_filename = filename.trim();

                        let download_url = if let Some(file_id) = href.strip_prefix("file:") {
                            format!("{}/files/{}", gilvave_core::settings::BASE_HTTP_URL, file_id)
                        } else if let Some(file_id) = href.strip_prefix("file_id:") {
                            format!("{}/files/{}", gilvave_core::settings::BASE_HTTP_URL, file_id)
                        } else if href.starts_with("http://") || href.starts_with("https://") {
                            href.to_string()
                        } else {
                            format!("{}{}", gilvave_core::settings::BASE_HTTP_URL, href)
                        };

                        use std::fmt::Write;
                        let _ = write!(
                            out,
                            r#"<a href="{download_url}" target="_blank" rel="noopener noreferrer" download="{clean_filename}" class="chat-message-file" title="Скачать {clean_filename}"><span class="chat-message-file-icon"><svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline></svg></span><span class="chat-message-file-info"><span class="chat-message-file-name">{clean_filename}</span><span class="chat-message-file-action">Нажмите для скачивания</span></span><span class="chat-message-file-download"><svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg></span></a>"#
                        );

                        remaining = &after_tag[close_pos + 4..];
                        continue;
                    }
                }

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

/// Рендеринг Markdown сообщения
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
    fn test_render_chat_markdown_formatting_and_photos() {
        let md = "Привет **мир**!\nПосмотри на фото: ![Кот](https://example.com/cat.jpg)\nИ эмодзи: <e:apple:🍏> 😀";
        let html = render_chat_markdown(md, EmojiPack::Fluent3D);

        // Check bold
        assert!(html.contains("<strong>мир</strong>"));
        // Check line break
        assert!(html.contains("<br />"));
        // Check photo class
        assert!(html.contains("class=\"chat-message-image\""));
        assert!(html.contains("src=\"https://example.com/cat.jpg\""));
        assert!(html.contains("alt=\"Кот\""));
        // Check Apple emoji preserved
        assert!(html.contains("emoji-datasource-apple"));
        // Check Unicode emoji rendered as Fluent 3D image
        assert!(html.contains("fluent-emoji-3d"));
    }

    #[test]
    fn test_render_chat_markdown_code_preserves_plain_text() {
        let md = "```rust\nlet x = \"😀\";\n```";
        let html = render_chat_markdown(md, EmojiPack::Fluent3D);
        assert!(html.contains("<pre><code"));
        // Emoji inside code should not be replaced with img tag
        assert!(html.contains("let x = \"😀\";"));
        assert!(!html.contains("class=\"chat-inline-emoji\""));
    }

    #[test]
    fn test_render_chat_markdown_video() {
        let md = "Посмотри видео: ![Мой ролик](https://example.com/videos/clip.mp4)";
        let html = render_chat_markdown(md, EmojiPack::Fluent3D);
        assert!(html.contains("<video class=\"chat-message-video\" controls preload=\"metadata\""));
        assert!(html.contains("src=\"https://example.com/videos/clip.mp4\""));
        assert!(!html.contains("<img"));
    }

    #[test]
    fn test_render_chat_markdown_file_card() {
        let md = "Скачай документ: [Отчет 2026.pdf](file:0192e123-4567-789a-bcde-f0123456789a)";
        let html = render_chat_markdown(md, EmojiPack::Fluent3D);
        assert!(html.contains("class=\"chat-message-file\""));
        assert!(html.contains("download=\"Отчет 2026.pdf\""));
        assert!(html.contains("/files/0192e123-4567-789a-bcde-f0123456789a"));
        assert!(html.contains("Отчет 2026.pdf"));
    }
}
