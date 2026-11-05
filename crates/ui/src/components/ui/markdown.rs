use pulldown_cmark::{Options, Parser, html};
use std::borrow::Cow;
use sycamore::prelude::*;
use sycamore::web::tags::div;
use sycamore::web::{GlobalProps, HtmlGlobalAttributes};

use crate::components::common::{
    EmojiPack, emoji_to_pack_url,
    media::{is_file_cached, is_video_source},
};

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
                } else if let Some(src) = extract_attribute(tag, "src") {
                    let alt = extract_attribute(tag, "alt").unwrap_or("video");
                    if is_video_source(src, alt) {
                        use std::fmt::Write;
                        let _ = write!(
                            out,
                            r#"<div class="chat-message-video-card custom-player is-paused" title="{alt}"><div class="video-fullscreen-topbar"><div class="video-topbar-info"><svg class="video-topbar-icon" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="2" width="20" height="20" rx="2.18" ry="2.18"></rect><line x1="7" y1="2" x2="7" y2="22"></line><line x1="17" y1="2" x2="17" y2="22"></line><line x1="2" y1="12" x2="22" y2="12"></line><line x1="2" y1="7" x2="7" y2="7"></line><line x1="2" y1="17" x2="7" y2="17"></line><line x1="17" y1="17" x2="22" y2="17"></line><line x1="17" y1="7" x2="22" y2="7"></line></svg><span class="video-topbar-title">{alt}</span></div><button type="button" class="video-btn video-fullscreen-close-btn" title="Закрыть (Esc)"><svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg></button></div><div class="video-stage"><video class="chat-custom-video" preload="metadata" playsinline title="{alt}"><source src="{src}" />Ваш браузер не поддерживает видео.</video><button type="button" class="video-center-play-btn" title="Воспроизвести"><svg class="play-icon" viewBox="0 0 24 24" fill="currentColor"><polygon points="5 3 19 12 5 21 5 3"></polygon></svg></button></div><div class="video-custom-controls"><div class="video-scrubber-track" title="Перемотка"><div class="video-scrubber-fill"><div class="video-scrubber-thumb"></div></div></div><div class="video-controls-row"><div class="video-controls-left"><button type="button" class="video-btn video-play-toggle" title="Воспроизведение / Пауза"><svg class="icon-play" viewBox="0 0 24 24" fill="currentColor"><polygon points="5 3 19 12 5 21 5 3"></polygon></svg><svg class="icon-pause" viewBox="0 0 24 24" fill="currentColor"><rect x="6" y="4" width="4" height="16"></rect><rect x="14" y="4" width="4" height="16"></rect></svg></button><div class="video-time-display"><span class="video-time-current">0:00</span><span class="video-time-divider">/</span><span class="video-time-duration">0:00</span></div><span class="video-track-name" title="{alt}">{alt}</span></div><div class="video-controls-right"><button type="button" class="video-btn video-volume-toggle" title="Звук"><svg class="icon-volume-on" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"></polygon><path d="M19.07 4.93a10 10 0 0 1 0 14.14M15.54 8.46a5 5 0 0 1 0 7.07"></path></svg><svg class="icon-volume-muted" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"></polygon><line x1="23" y1="9" x2="17" y2="15"></line><line x1="17" y1="9" x2="23" y2="15"></line></svg></button><button type="button" class="video-btn video-fullscreen-toggle" title="На весь экран"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="15 3 21 3 21 9"></polyline><polyline points="9 21 3 21 3 15"></polyline><line x1="21" y1="3" x2="14" y2="10"></line><line x1="3" y1="21" x2="10" y2="14"></line></svg></button><a href="{src}" download="{alt}" target="_blank" rel="noopener noreferrer" class="video-btn video-download-link" title="Скачать {alt}"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg></a></div></div></div></div>"#
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
                } else {
                    out.push_str(tag);
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

                        let (file_id_str, download_url) = if let Some(fid) = href.strip_prefix("file:") {
                            (fid.trim(), format!("{}/files/{}", gilvave_core::settings::BASE_HTTP_URL, fid.trim()))
                        } else if let Some(fid) = href.strip_prefix("file_id:") {
                            (fid.trim(), format!("{}/files/{}", gilvave_core::settings::BASE_HTTP_URL, fid.trim()))
                        } else if let Some(pos) = href.rfind("/files/") {
                            let p = &href[pos + 7..];
                            let stem = p.split(['/', '?', '#', '.']).next().unwrap_or(p);
                            (stem, if href.starts_with("http") { href.to_string() } else { format!("{}{}", gilvave_core::settings::BASE_HTTP_URL, href) })
                        } else {
                            ("", if href.starts_with("http") { href.to_string() } else { format!("{}{}", gilvave_core::settings::BASE_HTTP_URL, href) })
                        };

                        let cached = !file_id_str.is_empty() && is_file_cached(file_id_str);

                        use std::fmt::Write;
                        if cached {
                            let _ = write!(
                                out,
                                r#"<a href="{download_url}" data-file-id="{file_id_str}" data-filename="{clean_filename}" class="chat-message-file is-cached" title="Открыть {clean_filename}"><span class="chat-message-file-icon"><svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline></svg></span><span class="chat-message-file-info"><span class="chat-message-file-name">{clean_filename}</span><span class="chat-message-file-action cached-action"><svg class="cached-check-icon" viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"></polyline></svg>Файл сохранён · Нажмите для открытия</span></span><span class="chat-message-file-download" title="Открыть"><svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path><polyline points="15 3 21 3 21 9"></polyline><line x1="10" y1="14" x2="21" y2="3"></line></svg></span></a>"#
                            );
                        } else {
                            let _ = write!(
                                out,
                                r#"<a href="{download_url}" data-file-id="{file_id_str}" data-filename="{clean_filename}" class="chat-message-file" title="Скачать {clean_filename}"><span class="chat-message-file-icon"><svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline></svg></span><span class="chat-message-file-info"><span class="chat-message-file-name">{clean_filename}</span><span class="chat-message-file-action">Нажмите для скачивания</span></span><span class="chat-message-file-download"><svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg></span></a>"#
                            );
                        }

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

/// Распознавание единичного медиа элемента
fn try_consume_media_item(s: &str) -> Option<(String, &str)> {
    if s.starts_with("<img ")
        && s.contains("class=\"chat-message-image\"")
        && let Some(end) = s.find('>')
    {
        return Some((s[..=end].to_string(), &s[end + 1..]));
    }
    if s.starts_with("<div class=\"chat-message-video-card") {
        let mut depth = 0;
        let mut cur = s;
        let mut total = 0;
        while let Some(pos) = cur.find('<') {
            total += pos;
            cur = &cur[pos..];
            if cur.starts_with("<div") {
                depth += 1;
            } else if cur.starts_with("</div") {
                depth -= 1;
                if depth == 0
                    && let Some(close_tag_end) = cur.find('>')
                {
                    total += close_tag_end + 1;
                    return Some((s[..total].to_string(), &s[total..]));
                }
            }
            if let Some(end_tag) = cur.find('>') {
                total += end_tag + 1;
                cur = &cur[end_tag + 1..];
            } else {
                break;
            }
        }
    }
    None
}

/// Извлечение медиа элементов из блока
fn extract_media_from_block(inner: &str) -> (String, Vec<String>) {
    let mut clean = String::with_capacity(inner.len());
    let mut media = Vec::new();
    let mut cur = inner;

    while !cur.is_empty() {
        if let Some((item, rest)) = try_consume_media_item(cur) {
            media.push(item);
            cur = rest;
            continue;
        }
        let ch = cur.chars().next().unwrap();
        clean.push(ch);
        cur = &cur[ch.len_utf8()..];
    }

    let mut trimmed = clean.trim();
    while trimmed.ends_with("<br />")
        || trimmed.ends_with("<br>")
        || trimmed.starts_with("<br />")
        || trimmed.starts_with("<br>")
    {
        trimmed = trimmed
            .trim_start_matches("<br />")
            .trim_start_matches("<br>")
            .trim_end_matches("<br />")
            .trim_end_matches("<br>")
            .trim();
    }

    (trimmed.to_string(), media)
}

/// Группировка медиа элементов (картинки, видео) сообщения в общую галерею
fn group_chat_media_galleries(html: &str) -> String {
    let mut out = String::with_capacity(html.len() + 128);
    let mut remaining = html;
    let mut pending_gallery: Vec<String> = Vec::new();

    let flush_gallery = |buf: &mut String, gallery: &mut Vec<String>| {
        if gallery.is_empty() {
            return;
        }
        let modifier = if gallery.len() == 1 {
            "single-media"
        } else {
            "multi-media"
        };
        buf.push_str("<div class=\"chat-message-media-gallery ");
        buf.push_str(modifier);
        buf.push_str("\">");
        for item in gallery.drain(..) {
            buf.push_str(&item);
        }
        buf.push_str("</div>");
    };

    while !remaining.is_empty() {
        if remaining.starts_with("<p>")
            && let Some(end_p) = remaining.find("</p>")
        {
            let inner = &remaining[3..end_p];
            let (cleaned_inner, media_elements) = extract_media_from_block(inner);
            if cleaned_inner.trim().is_empty() {
                pending_gallery.extend(media_elements);
            } else {
                flush_gallery(&mut out, &mut pending_gallery);
                out.push_str("<p>");
                out.push_str(&cleaned_inner);
                out.push_str("</p>");
                pending_gallery.extend(media_elements);
            }
            remaining = &remaining[end_p + 4..];
            continue;
        }

        if let Some((media_html, rest)) = try_consume_media_item(remaining) {
            pending_gallery.push(media_html);
            remaining = rest;
            continue;
        }

        if remaining.starts_with('\n') || remaining.starts_with("  ") {
            let ch = remaining.chars().next().unwrap();
            out.push(ch);
            remaining = &remaining[ch.len_utf8()..];
            continue;
        }

        flush_gallery(&mut out, &mut pending_gallery);
        let next_tag = remaining.find('<').unwrap_or(remaining.len());
        if next_tag == 0 {
            let next_end = remaining.find('>').map(|i| i + 1).unwrap_or(1);
            out.push_str(&remaining[..next_end]);
            remaining = &remaining[next_end..];
        } else {
            out.push_str(&remaining[..next_tag]);
            remaining = &remaining[next_tag..];
        }
    }

    flush_gallery(&mut out, &mut pending_gallery);
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

    let postprocessed = postprocess_chat_html(&html_raw, default_pack);
    group_chat_media_galleries(&postprocessed)
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

/// Инициализация глобального контроллера фирменного видеоплеера
#[cfg(target_arch = "wasm32")]
pub fn setup_custom_video_players() {
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;

    let Some(window) = web_sys::window() else { return; };
    let Some(document) = window.document() else { return; };

    fn format_time(secs: f64) -> String {
        if secs.is_nan() || secs.is_infinite() || secs < 0.0 {
            return "0:00".to_string();
        }
        let total = secs.floor() as u64;
        let s = total % 60;
        let m = (total / 60) % 60;
        let h = total / 3600;
        if h > 0 {
            format!("{h}:{m:02}:{s:02}")
        } else {
            format!("{m}:{s:02}")
        }
    }

    // 1. Делегированный клик
    let doc_click = document.clone();
    let click_closure = Closure::<dyn Fn(web_sys::MouseEvent)>::new(move |e: web_sys::MouseEvent| {
        let Some(target) = e.target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()) else { return; };

        // Обработка клика на файл в чате (открытие из кэша / через ОС opener)
        if let Ok(Some(file_card)) = target.closest(".chat-message-file") {
            e.prevent_default();
            e.stop_propagation();

            let fid = file_card.get_attribute("data-file-id").unwrap_or_default();
            let fname = file_card.get_attribute("data-filename").unwrap_or_else(|| "file".to_string());
            let url = file_card.get_attribute("href").unwrap_or_default();

            wasm_bindgen_futures::spawn_local(async move {
                crate::components::common::media::open_file_card_async(&fid, &fname, &url).await;
            });
            return;
        }

        if let Ok(Some(card)) = target.closest(".chat-message-video-card") {
            let is_fs_close = target.closest(".video-fullscreen-close-btn").ok().flatten().is_some();
            let is_center_btn = target.closest(".video-center-play-btn").ok().flatten().is_some();
            let is_play_toggle = target.closest(".video-play-toggle").ok().flatten().is_some();
            let is_video_stage = target.closest(".video-stage").ok().flatten().is_some()
                && target.closest(".video-custom-controls").ok().flatten().is_none();
            let is_volume_toggle = target.closest(".video-volume-toggle").ok().flatten().is_some();
            let is_fs_toggle = target.closest(".video-fullscreen-toggle").ok().flatten().is_some();
            let is_scrubber = target.closest(".video-scrubber-track").ok().flatten();

            let video_opt = card.query_selector("video.chat-custom-video").ok().flatten()
                .and_then(|v| v.dyn_into::<web_sys::HtmlVideoElement>().ok());

            if let Some(video) = video_opt {
                if is_fs_close {
                    e.stop_propagation();
                    if doc_click.fullscreen_element().is_some() {
                        let _ = doc_click.exit_fullscreen();
                    }
                    let _ = card.class_list().remove_1("is-fullscreen");
                    return;
                }

                if is_volume_toggle {
                    e.stop_propagation();
                    let new_muted = !video.muted();
                    video.set_muted(new_muted);
                    if new_muted {
                        let _ = card.class_list().add_1("is-muted");
                    } else {
                        let _ = card.class_list().remove_1("is-muted");
                    }
                    return;
                }

                if is_fs_toggle {
                    e.stop_propagation();
                    if doc_click.fullscreen_element().is_some() || card.class_list().contains("is-fullscreen") {
                        if doc_click.fullscreen_element().is_some() {
                            let _ = doc_click.exit_fullscreen();
                        }
                        let _ = card.class_list().remove_1("is-fullscreen");
                    } else {
                        let _ = card.request_fullscreen();
                        let _ = card.class_list().add_1("is-fullscreen");
                    }
                    return;
                }

                if let Some(scrubber_el) = is_scrubber {
                    e.stop_propagation();
                    let rect = scrubber_el.get_bounding_client_rect();
                    let width = rect.width();
                    if width > 0.0 {
                        let click_x = (e.client_x() as f64 - rect.left()).clamp(0.0, width);
                        let ratio = click_x / width;
                        let dur = video.duration();
                        if dur > 0.0 && !dur.is_nan() {
                            video.set_current_time(ratio * dur);
                        }
                    }
                    return;
                }

                if is_center_btn || is_play_toggle || is_video_stage {
                    e.stop_propagation();
                    if video.paused() {
                        let _ = video.play();
                    } else {
                        let _ = video.pause();
                    }
                }
            }
        }
    });
    let _ = document.add_event_listener_with_callback("click", click_closure.as_ref().unchecked_ref());
    click_closure.forget();

    // 2. Событие play
    let play_closure = Closure::<dyn Fn(web_sys::Event)>::new(move |e: web_sys::Event| {
        let Some(video) = e.target().and_then(|t| t.dyn_into::<web_sys::HtmlVideoElement>().ok()) else { return; };
        if let Ok(Some(card)) = video.closest(".chat-message-video-card") {
            let _ = card.class_list().remove_1("is-paused");
            let _ = card.class_list().add_1("is-playing");
        }
    });
    let _ = document.add_event_listener_with_callback_and_bool("play", play_closure.as_ref().unchecked_ref(), true);
    play_closure.forget();

    // 3. События pause / ended
    let pause_closure = Closure::<dyn Fn(web_sys::Event)>::new(move |e: web_sys::Event| {
        let Some(video) = e.target().and_then(|t| t.dyn_into::<web_sys::HtmlVideoElement>().ok()) else { return; };
        if let Ok(Some(card)) = video.closest(".chat-message-video-card") {
            let _ = card.class_list().remove_1("is-playing");
            let _ = card.class_list().add_1("is-paused");
        }
    });
    let _ = document.add_event_listener_with_callback_and_bool("pause", pause_closure.as_ref().unchecked_ref(), true);
    let _ = document.add_event_listener_with_callback_and_bool("ended", pause_closure.as_ref().unchecked_ref(), true);
    pause_closure.forget();

    // 4. Событие timeupdate
    let time_closure = Closure::<dyn Fn(web_sys::Event)>::new(move |e: web_sys::Event| {
        let Some(video) = e.target().and_then(|t| t.dyn_into::<web_sys::HtmlVideoElement>().ok()) else { return; };
        let Ok(Some(card)) = video.closest(".chat-message-video-card") else { return; };

        let cur = video.current_time();
        let dur = video.duration();
        let pct = if dur > 0.0 && !dur.is_nan() { (cur / dur * 100.0).clamp(0.0, 100.0) } else { 0.0 };

        if let Ok(Some(fill_el)) = card.query_selector(".video-scrubber-fill") {
            if let Ok(html_fill) = fill_el.dyn_into::<web_sys::HtmlElement>() {
                let _ = html_fill.style().set_property("width", &format!("{pct:.1}%"));
            }
        }
        if let Ok(Some(cur_el)) = card.query_selector(".video-time-current") {
            cur_el.set_text_content(Some(&format_time(cur)));
        }
        if let Ok(Some(dur_el)) = card.query_selector(".video-time-duration") {
            if dur > 0.0 && !dur.is_nan() {
                dur_el.set_text_content(Some(&format_time(dur)));
            }
        }
    });
    let _ = document.add_event_listener_with_callback_and_bool("timeupdate", time_closure.as_ref().unchecked_ref(), true);
    time_closure.forget();

    // 5. Событие loadedmetadata
    let meta_closure = Closure::<dyn Fn(web_sys::Event)>::new(move |e: web_sys::Event| {
        let Some(video) = e.target().and_then(|t| t.dyn_into::<web_sys::HtmlVideoElement>().ok()) else { return; };
        let Ok(Some(card)) = video.closest(".chat-message-video-card") else { return; };
        let dur = video.duration();
        if dur > 0.0 && !dur.is_nan() {
            if let Ok(Some(dur_el)) = card.query_selector(".video-time-duration") {
                dur_el.set_text_content(Some(&format_time(dur)));
            }
        }
    });
    let _ = document.add_event_listener_with_callback_and_bool("loadedmetadata", meta_closure.as_ref().unchecked_ref(), true);
    meta_closure.forget();

    // 6. Отслеживание перехода в полноэкранный режим
    let doc_fs = document.clone();
    let fs_closure = Closure::<dyn Fn(web_sys::Event)>::new(move |_| {
        if let Ok(cards) = doc_fs.query_selector_all(".chat-message-video-card") {
            for i in 0..cards.length() {
                if let Some(el) = cards.item(i).and_then(|n| n.dyn_into::<web_sys::Element>().ok()) {
                    let is_active_fs = doc_fs.fullscreen_element().map(|fe| fe == el).unwrap_or(false);
                    if is_active_fs {
                        let _ = el.class_list().add_1("is-fullscreen");
                    } else {
                        let _ = el.class_list().remove_1("is-fullscreen");
                    }
                }
            }
        }
    });
    let _ = document.add_event_listener_with_callback("fullscreenchange", fs_closure.as_ref().unchecked_ref());
    let _ = document.add_event_listener_with_callback("webkitfullscreenchange", fs_closure.as_ref().unchecked_ref());
    fs_closure.forget();

    // 7. Выход из полноэкранного режима по Escape
    let doc_esc = document.clone();
    let esc_closure = Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(move |e: web_sys::KeyboardEvent| {
        if e.key() == "Escape" {
            if let Ok(cards) = doc_esc.query_selector_all(".chat-message-video-card.is-fullscreen") {
                for i in 0..cards.length() {
                    if let Some(el) = cards.item(i).and_then(|n| n.dyn_into::<web_sys::Element>().ok()) {
                        let _ = el.class_list().remove_1("is-fullscreen");
                    }
                }
            }
        }
    });
    let _ = document.add_event_listener_with_callback("keydown", esc_closure.as_ref().unchecked_ref());
    esc_closure.forget();
}

#[cfg(not(target_arch = "wasm32"))]
pub fn setup_custom_video_players() {}

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
        // Check media gallery wrapper
        assert!(html.contains("chat-message-media-gallery"));
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
    fn test_render_chat_markdown_video() {
        let md = "Посмотри видео: ![Мой ролик](https://example.com/videos/clip.mp4)";
        let html = render_chat_markdown(md, EmojiPack::Fluent3D);
        assert!(html.contains("chat-message-media-gallery"));
        assert!(html.contains("chat-message-video-card custom-player"));
        assert!(html.contains("video-fullscreen-topbar"));
        assert!(html.contains("<video class=\"chat-custom-video\" preload=\"metadata\" playsinline"));
        assert!(html.contains("src=\"https://example.com/videos/clip.mp4\""));
        assert!(html.contains("video-download-link"));
        assert!(html.contains("class=\"video-center-play-btn\""));
        assert!(html.contains("class=\"video-custom-controls\""));
        assert!(!html.contains("<img"));
    }

    #[test]
    fn test_render_chat_markdown_multiple_images_gallery() {
        let md = "два рисунка\n\n![город](http://127.0.0.1:8080/files/11111111-1111-1111-1111-111111111111.png)\n\n![лис](http://127.0.0.1:8080/files/22222222-2222-2222-2222-222222222222.png)";
        let html = render_chat_markdown(md, EmojiPack::Fluent3D);
        assert!(html.contains("<p>два рисунка</p>"));
        assert!(html.contains("class=\"chat-message-media-gallery multi-media\""));
        assert!(html.contains("alt=\"город\""));
        assert!(html.contains("alt=\"лис\""));
    }

    #[test]
    fn test_render_chat_markdown_file_card() {
        let md = "Скачай документ: [Отчет 2026.pdf](file:0192e123-4567-789a-bcde-f0123456789a)";
        let html = render_chat_markdown(md, EmojiPack::Fluent3D);
        assert!(html.contains("class=\"chat-message-file\""));
        assert!(html.contains("data-filename=\"Отчет 2026.pdf\""));
        assert!(html.contains("data-file-id=\"0192e123-4567-789a-bcde-f0123456789a\""));
        assert!(html.contains("/files/0192e123-4567-789a-bcde-f0123456789a"));
        assert!(html.contains("Отчет 2026.pdf"));
    }
}
