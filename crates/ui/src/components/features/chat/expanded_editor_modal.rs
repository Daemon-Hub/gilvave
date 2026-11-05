//! Fullscreen / expanded rich-text editor modal for long messages and formatting.

use sycamore::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlElement, SubmitEvent};

use super::chat_input_area::{EmojiPicker, send_channel_message};
use crate::components::common::{
    ChannelContext, ServerContext, UiModalContext, UserProfileContext,
    insert_text_into_editor, serialize_contenteditable, sync_editor_content,
};

#[component]
pub fn ExpandedMessageEditorModal() -> View {
    let modal_context = use_context::<UiModalContext>();
    let channel_context = use_context::<ChannelContext>();
    let server_context = use_context::<ServerContext>();
    let user_profile = use_context::<UserProfileContext>();

    let draft_message = modal_context.draft_message;
    let is_emoji_open = create_signal(false);
    let is_emoji_expanded = create_signal(false);
    let emoji_search = create_signal(String::new());
    let emoji_category = create_signal("smileys".to_string());
    let has_scrollbar = create_signal(false);
    let expanded_editor_ref = create_node_ref();

    let target_title = create_memo(move || {
        let in_server = server_context.current.with(|s| s.is_some());
        if in_server {
            channel_context.current.with(|c| {
                c.as_ref().map_or_else(
                    || "Редактор сообщения".to_string(),
                    |ch| format!("#{}", ch.name),
                )
            })
        } else {
            modal_context.selected_dm_name.with(|dm| {
                dm.as_ref()
                    .map_or_else(|| "Редактор сообщения".to_string(), |name| name.clone())
            })
        }
    });

    let char_count_text = create_memo(move || {
        let count = draft_message.with(|msg| gilvave_core::validation::count_typed_chars(msg));
        format!("{count} / {}", gilvave_core::validation::MAX_MESSAGE_CHARS)
    });

    let is_over_limit = create_memo(move || {
        draft_message.with(|msg| gilvave_core::validation::count_typed_chars(msg) > gilvave_core::validation::MAX_MESSAGE_CHARS)
    });

    let counter_class = move || {
        if is_over_limit.get() {
            "editor-char-counter over-limit"
        } else {
            "editor-char-counter"
        }
    };

    let on_toggle_emoji = move |e: web_sys::MouseEvent| {
        e.stop_propagation();
        let next = !is_emoji_open.get();
        is_emoji_open.set(next);
        if !next {
            is_emoji_expanded.set(false);
        }
    };

    let on_collapse = move |_| {
        is_emoji_open.set(false);
        is_emoji_expanded.set(false);
        modal_context.is_message_editor_open.set(false);
    };

    let on_overlay_click = move |_| {
        if is_emoji_open.get() {
            is_emoji_open.set(false);
            is_emoji_expanded.set(false);
        } else {
            modal_context.is_message_editor_open.set(false);
        }
    };

    let on_window_click = move |e: web_sys::MouseEvent| {
        e.stop_propagation();
    };

    let do_send = move || {
        if let Some(node) = expanded_editor_ref.try_get()
            && let Ok(editor) = node.dyn_into::<HtmlElement>()
        {
            let current = serialize_contenteditable(&editor);
            if !current.trim().is_empty() {
                draft_message.set(current);
            }
        }

        let in_server = server_context.current.with(|s| s.is_some());
        if in_server {
            let has_attachments = modal_context.pending_attachments.with(|a| !a.is_empty());
            let (is_empty, is_too_long) = draft_message.with(|msg| {
                let trimmed = msg.trim();
                let count = gilvave_core::validation::count_typed_chars(trimmed);
                (trimmed.is_empty(), count > gilvave_core::validation::MAX_MESSAGE_CHARS)
            });
            if is_too_long || (is_empty && !has_attachments) {
                return;
            }
            send_channel_message(
                channel_context,
                modal_context,
                draft_message,
                user_profile,
                None,
            );
            modal_context.is_message_editor_open.set(false);
        } else if modal_context.selected_dm_name.with(|dm| dm.is_some()) {
            let has_text = draft_message.with(|msg| !msg.trim().is_empty());
            if has_text {
                draft_message.set(String::new());
                modal_context.is_message_editor_open.set(false);
            }
        }
    };

    let on_submit = move |e: SubmitEvent| {
        e.prevent_default();
        do_send();
    };

    create_effect(move || {
        let msg = draft_message.get_clone();
        if let Some(node) = expanded_editor_ref.try_get()
            && let Ok(editor) = node.dyn_into::<HtmlElement>()
        {
            let current = serialize_contenteditable(&editor);
            if current != msg {
                sync_editor_content(&editor, &msg);
                if msg.trim().is_empty() {
                    let _ = editor.set_attribute("data-empty", "true");
                } else {
                    let _ = editor.remove_attribute("data-empty");
                }
            }
        }
    });

    let on_expanded_input = move |event: Event| {
        if let Some(target) = event.current_target() {
            let el: HtmlElement = target.unchecked_into();
            let serialized = serialize_contenteditable(&el);
            if serialized.trim().is_empty() {
                let _ = el.set_attribute("data-empty", "true");
            } else {
                let _ = el.remove_attribute("data-empty");
            }
            draft_message.set(serialized);
        }
    };

    let add_expanded_pending_file = move |file: web_sys::File| {
        let file_size = file.size() as u64;
        if file_size == 0 {
            crate::components::ui::context_menu::clipboard::show_toast(
                &modal_context,
                true,
                format!("Файл «{}» пуст (0 байт)", file.name()),
            );
            return;
        }
        if file_size > gilvave_core::files::MAX_FILE_SIZE_BYTES {
            crate::components::ui::context_menu::clipboard::show_toast(
                &modal_context,
                true,
                format!(
                    "Файл «{}» ({:.1} МБ) превышает максимальный лимит {} МБ",
                    file.name(),
                    file_size as f64 / (1024.0 * 1024.0),
                    gilvave_core::files::MAX_FILE_SIZE_MB
                ),
            );
            return;
        }
        let att = crate::components::common::media::PendingAttachment::new(file);
        modal_context.pending_attachments.update(|list| list.push(att));
    };

    let on_expanded_paste = move |event: web_sys::Event| {
        if let Ok(paste_event) = event.dyn_into::<web_sys::ClipboardEvent>()
            && let Some(cd) = paste_event.clipboard_data()
        {
            let items = cd.items();
            let mut found_file = false;
            for i in 0..items.length() {
                if let Some(item) = items.get(i)
                    && let Ok(Some(file)) = item.get_as_file()
                {
                    found_file = true;
                    paste_event.prevent_default();
                    add_expanded_pending_file(file);
                    break;
                }
            }
            if found_file {
                return;
            }
            paste_event.prevent_default();
            if let Ok(text) = cd.get_data("text/plain")
                && let Some(target) = paste_event.current_target()
            {
                let el: HtmlElement = target.unchecked_into();
                insert_text_into_editor(&el, &text);
                let serialized = serialize_contenteditable(&el);
                if serialized.trim().is_empty() {
                    let _ = el.set_attribute("data-empty", "true");
                } else {
                    let _ = el.remove_attribute("data-empty");
                }
                draft_message.set(serialized);
            }
        }
    };

    let on_keydown = move |e: web_sys::KeyboardEvent| {
        if e.key() == "Escape" {
            e.prevent_default();
            if is_emoji_open.get() {
                is_emoji_open.set(false);
                is_emoji_expanded.set(false);
            } else {
                modal_context.is_message_editor_open.set(false);
            }
        } else if e.key() == "Enter" && (e.ctrl_key() || e.meta_key()) {
            e.prevent_default();
            do_send();
        }
    };

    let ensure_expanded_caret_at_start = |target: Option<web_sys::EventTarget>| {
        if let Some(target) = target
            && let Ok(el) = target.dyn_into::<HtmlElement>()
            && serialize_contenteditable(&el).is_empty()
            && el.child_nodes().length() == 0
            && let Some(win) = web_sys::window()
            && let Ok(Some(sel)) = win.get_selection()
        {
            let _ = sel.collapse(Some(el.as_ref()));
        }
    };
    let on_expanded_focus =
        move |e: web_sys::FocusEvent| ensure_expanded_caret_at_start(e.target());

    view! {
        div(class="message-editor-overlay", on:click=on_overlay_click) {
            div(class="message-editor-wrapper") {
                form(
                    class="message-editor-window",
                    on:click=on_window_click,
                    on:submit=on_submit,
                ) {
                    div(class="message-editor-header") {
                        div(class="message-editor-title-group") {
                            span(class="message-editor-badge") { "Редактор текста" }
                            span(class="message-editor-target") { (target_title) }
                        }
                    }

                    div(class="message-editor-body") {
                        div(
                            r#ref=expanded_editor_ref,
                            class="expanded-editor-textarea",
                            contenteditable="true",
                            role="textbox",
                            data-placeholder="Напишите сообщение...",
                            on:input=on_expanded_input,
                            on:focus=on_expanded_focus,
                            on:keydown=on_keydown,
                            on:paste=on_expanded_paste,
                        )
                        button(
                            class="editor-collapse-btn",
                            r#type="button",
                            title="Свернуть окно и вернуться к обычному полю",
                            on:click=on_collapse,
                        ) {
                            img(src="/public/arrow2.svg", alt="Свернуть")
                        }
                        button(
                            class=move || if is_emoji_open.get() {
                                "editor-emoji-btn active"
                            } else {
                                "editor-emoji-btn"
                            },
                            r#type="button",
                            title="Выбрать эмодзи",
                            on:click=on_toggle_emoji,
                        ) {
                            svg(
                                viewBox="0 0 24 24",
                                fill="none",
                                stroke="currentColor",
                                stroke-width="2",
                                stroke-linecap="round",
                                stroke-linejoin="round",
                            ) {
                                circle(cx="12", cy="12", r="10")
                                path(d="M8 14s1.5 2 4 2 4-2 4-2")
                                circle(cx="9", cy="9", r="1.2", fill="currentColor", stroke="none")
                                circle(cx="15", cy="9", r="1.2", fill="currentColor", stroke="none")
                            }
                        }
                    }

                    div(class="message-editor-footer") {
                        div(class="message-editor-meta") {
                            span(class=counter_class) { (char_count_text) }
                            span(class="editor-shortcut-hint") { "Ctrl + Enter — отправить · Esc — свернуть" }
                        }
                        div(class="message-editor-actions") {
                            button(
                                class="editor-Send-btn",
                                r#type="submit",
                                title="Отправить сообщение",
                            ) {
                                span { "Отправить" }
                                svg(
                                    xmlns="http://www.w3.org/2000/svg",
                                    width="16",
                                    height="16",
                                    viewBox="0 0 24 24",
                                    fill="none",
                                    stroke="currentColor",
                                    stroke-width="2.2",
                                    stroke-linecap="round",
                                    stroke-linejoin="round",
                                ) {
                                    line(x1="22", y1="2", x2="11", y2="13")
                                    polygon(points="22 2 15 22 11 13 2 9 22 2")
                                }
                            }
                        }
                    }
                }

                (move || if is_emoji_open.get() {
                    view! {
                        div(
                            class="editor-side-emoji-panel",
                            on:click=move |e: web_sys::MouseEvent| e.stop_propagation(),
                        ) {
                            EmojiPicker(
                                is_open=is_emoji_open,
                                is_expanded=is_emoji_expanded,
                                search=emoji_search,
                                category=emoji_category,
                                editor_ref=expanded_editor_ref,
                                has_scrollbar=has_scrollbar,
                                draft_message=draft_message,
                            )
                        }
                    }
                } else {
                    view! {}
                })
            }
        }
    }
}
