//! Chat input area with rich-text contenteditable, emoji picker, attachments, and actions.

mod action_islands;
mod attach_menu;
mod emoji_picker;
mod send_message;

pub(crate) use send_message::send_channel_message;

use action_islands::ActionIslands;
use attach_menu::AttachMenu;
use emoji_picker::EmojiPicker;

use sycamore::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlElement, SubmitEvent};

use crate::components::common::{
    focus_editor_at_end, insert_text_into_editor, serialize_contenteditable,
    sync_editor_content, ChannelContext, ScreenWrapper, UiModalContext,
};

#[component(inline_props)]
pub fn ChatInputArea<F>(on_send: F) -> View
where
    F: Fn() + Copy + 'static,
{
    let screen_wrapper = use_context::<ScreenWrapper>();
    let channel_context = use_context::<ChannelContext>();
    let modal_context = use_context::<UiModalContext>();
    let draft_message = modal_context.draft_message;
    let editor_ref = create_node_ref();

    let has_scrollbar = create_signal(false);
    let is_attach_menu_open = create_signal(false);
    let is_emoji_modal_open = create_signal(false);
    let is_emoji_expanded = create_signal(false);
    let emoji_search = create_signal(String::new());
    let emoji_category = create_signal("smileys".to_string());

    // Auto-close popovers when leaving Home screen
    create_effect(move || {
        if !screen_wrapper.is_home() {
            is_attach_menu_open.set(false);
            is_emoji_modal_open.set(false);
            is_emoji_expanded.set(false);
        }
    });

    // Auto-close popovers when active channel changes
    let last_channel_id =
        create_signal(channel_context.current.with(|c| c.as_ref().map(|ch| ch.id)));
    create_effect(move || {
        let cur_id = channel_context.current.with(|c| c.as_ref().map(|ch| ch.id));
        if cur_id != last_channel_id.get_untracked() {
            last_channel_id.set(cur_id);
            is_attach_menu_open.set(false);
            is_emoji_modal_open.set(false);
            is_emoji_expanded.set(false);
        }
    });

    // Global Escape key listener when popover or editing is active
    create_effect(move || {
        let is_editing = modal_context.editing_message.with(|e| e.is_some());
        if is_emoji_modal_open.get() || is_attach_menu_open.get() || is_editing {
            #[cfg(target_arch = "wasm32")]
            if let Some(window) = web_sys::window() {
                let modal_ctx = modal_context;
                let on_key = wasm_bindgen::closure::Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(
                    move |e: web_sys::KeyboardEvent| {
                        if e.key() == "Escape" {
                            if is_attach_menu_open.get() {
                                is_attach_menu_open.set(false);
                                return;
                            }
                            if is_emoji_modal_open.get() {
                                is_emoji_modal_open.set(false);
                                is_emoji_expanded.set(false);
                                return;
                            }
                            if modal_ctx.editing_message.with(|ed| ed.is_some()) {
                                modal_ctx.editing_message.set(None);
                                modal_ctx.draft_message.set(String::new());
                            }
                        }
                    },
                );
                let _ = window
                    .add_event_listener_with_callback("keydown", on_key.as_ref().unchecked_ref());
                let window_clone = window.clone();
                on_cleanup(move || {
                    let _ = window_clone.remove_event_listener_with_callback(
                        "keydown",
                        on_key.as_ref().unchecked_ref(),
                    );
                });
            }
        }
    });

    create_effect(move || {
        let msg = draft_message.get_clone();
        if let Some(node) = editor_ref.try_get()
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
                let overflowed = editor.scroll_height() > 142;
                has_scrollbar.set(overflowed);
            }
        }
    });

    // Automatically focus the input when replying_to or editing_message is set
    create_effect(move || {
        let is_replying = modal_context.replying_to.with(|r| r.is_some());
        let is_editing = modal_context.editing_message.with(|e| e.is_some());
        if (is_replying || is_editing)
            && let Some(node) = editor_ref.try_get()
            && let Ok(editor) = node.dyn_into::<HtmlElement>()
        {
            focus_editor_at_end(&editor);
        }
    });

    let on_submit = move |event: SubmitEvent| {
        event.prevent_default();
        is_attach_menu_open.set(false);
        is_emoji_modal_open.set(false);
        is_emoji_expanded.set(false);
        if let Some(node) = editor_ref.try_get()
            && let Ok(editor) = node.dyn_into::<HtmlElement>()
        {
            editor.set_inner_html("");
            let _ = editor.set_attribute("data-empty", "true");
        }
        on_send();
    };

    let on_input = move |event: Event| {
        if let Some(target) = event.current_target() {
            let el: HtmlElement = target.unchecked_into();
            let serialized = serialize_contenteditable(&el);
            if serialized.trim().is_empty() {
                let _ = el.set_attribute("data-empty", "true");
            } else {
                let _ = el.remove_attribute("data-empty");
            }
            let overflowed = el.scroll_height() > 142;
            has_scrollbar.set(overflowed);
            draft_message.set(serialized);
        }
    };

    let on_paste = move |event: web_sys::Event| {
        if let Ok(paste_event) = event.dyn_into::<web_sys::ClipboardEvent>() {
            paste_event.prevent_default();
            if let Some(cd) = paste_event.clipboard_data()
                && let Ok(text) = cd.get_data("text/plain")
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
                let overflowed = el.scroll_height() > 142;
                has_scrollbar.set(overflowed);
                draft_message.set(serialized);
            }
        }
    };

    let on_keydown = move |event: web_sys::KeyboardEvent| {
        if event.key() == "Escape" {
            if is_emoji_modal_open.get() {
                is_emoji_modal_open.set(false);
                is_emoji_expanded.set(false);
                return;
            }
            if is_attach_menu_open.get() {
                is_attach_menu_open.set(false);
                return;
            }
            if modal_context.editing_message.with(|e| e.is_some()) {
                modal_context.editing_message.set(None);
                modal_context.draft_message.set(String::new());
                return;
            }
            if modal_context.replying_to.with(|r| r.is_some()) {
                modal_context.replying_to.set(None);
                return;
            }
        }
        if event.key() == "Enter" {
            if event.shift_key() {
                // Allow shift+enter for multiline newline
            } else {
                event.prevent_default();
                if event.is_composing() || event.key_code() == 229 {
                    return;
                }
                is_attach_menu_open.set(false);
                is_emoji_modal_open.set(false);
                is_emoji_expanded.set(false);
                if let Some(node) = editor_ref.try_get()
                    && let Ok(editor) = node.dyn_into::<HtmlElement>()
                {
                    editor.set_inner_html("");
                    let _ = editor.set_attribute("data-empty", "true");
                }
                on_send();
            }
        }
    };

    let on_open_editor = move |_| {
        is_attach_menu_open.set(false);
        is_emoji_modal_open.set(false);
        is_emoji_expanded.set(false);
        modal_context.is_message_editor_open.set(true);
    };

    let on_toggle_attach = move |e: web_sys::MouseEvent| {
        e.stop_propagation();
        let next = !is_attach_menu_open.get();
        is_attach_menu_open.set(next);
        if next {
            is_emoji_modal_open.set(false);
            is_emoji_expanded.set(false);
        }
    };

    let on_toggle_emoji = move |e: web_sys::MouseEvent| {
        e.stop_propagation();
        let next = !is_emoji_modal_open.get();
        is_emoji_modal_open.set(next);
        if next {
            is_attach_menu_open.set(false);
        } else {
            is_emoji_expanded.set(false);
        }
    };

    let on_file_selected = move |e: Event| {
        if let Some(target) = e.target()
            && let Ok(input) = target.dyn_into::<web_sys::HtmlInputElement>()
        {
            input.set_value("");
        }
        is_attach_menu_open.set(false);
    };

    let ensure_caret_at_start = |target: Option<web_sys::EventTarget>| {
        if let Some(target) = target
            && let Ok(el) = target.dyn_into::<HtmlElement>()
            && el.inner_text().trim().is_empty()
            && let Some(win) = web_sys::window()
            && let Ok(Some(sel)) = win.get_selection()
        {
            let _ = sel.collapse(Some(el.as_ref()));
        }
    };
    let on_input_focus = move |e: web_sys::FocusEvent| ensure_caret_at_start(e.target());
    let on_input_click = move |e: web_sys::MouseEvent| ensure_caret_at_start(e.target());

    view! {
        div(class="chat-bottom-dock") {
            ActionIslands()
            form(class="input-area", on:submit=on_submit) {
                div(class="chat-input-wrapper") {
                    // Click-outside backdrop for attachment menu or emoji modal
                    (move || if is_attach_menu_open.get() || is_emoji_modal_open.get() {
                        view! {
                            div(
                                class=move || {
                                    if is_emoji_expanded.get() {
                                        "chat-popover-backdrop is-dimmed"
                                    } else {
                                        "chat-popover-backdrop"
                                    }
                                },
                                on:click=move |_| {
                                    is_attach_menu_open.set(false);
                                    is_emoji_modal_open.set(false);
                                    is_emoji_expanded.set(false);
                                },
                                on:touchstart=move |_| {
                                    is_attach_menu_open.set(false);
                                    is_emoji_modal_open.set(false);
                                    is_emoji_expanded.set(false);
                                },
                            )
                        }
                    } else {
                        view! {}
                    })

                    // Floating character counter above input (after 4096 chars)
                    div(
                        class=move || {
                            let count = draft_message.with(|msg| msg.chars().count());
                            if count > gilvave_core::validation::MAX_MESSAGE_CHARS {
                                "chat-char-counter-float visible over-limit"
                            } else if count >= gilvave_core::validation::MESSAGE_COUNTER_VISIBLE_CHARS {
                                "chat-char-counter-float visible"
                            } else {
                                "chat-char-counter-float"
                            }
                        },
                    ) {
                        (move || {
                            let count = draft_message.with(|msg| msg.chars().count());
                            if count >= gilvave_core::validation::MESSAGE_COUNTER_VISIBLE_CHARS {
                                format!("{count} / {}", gilvave_core::validation::MAX_MESSAGE_CHARS)
                            } else {
                                String::new()
                            }
                        })
                    }

                    AttachMenu(is_open=is_attach_menu_open, on_file_selected=on_file_selected)

                    EmojiPicker(
                        is_open=is_emoji_modal_open,
                        is_expanded=is_emoji_expanded,
                        search=emoji_search,
                        category=emoji_category,
                        editor_ref=editor_ref,
                        has_scrollbar=has_scrollbar,
                        draft_message=draft_message,
                    )

                    // Paperclip button pinned at bottom-left inside input
                    button(
                        class=move || if is_attach_menu_open.get() {
                            "chat-attach-btn active"
                        } else {
                            "chat-attach-btn"
                        },
                        r#type="button",
                        title="Прикрепить фото, видео или файл",
                        on:click=on_toggle_attach,
                    ) {
                        svg(viewBox="0 0 24 24") {
                            path(d="m21.44 11.05-9.19 9.19a6 6 0 0 1-8.49-8.49l8.57-8.57A4 4 0 1 1 18 8.84l-8.59 8.57a2 2 0 0 1-2.83-2.83l8.49-8.48")
                        }
                    }

                    div(
                        r#ref=editor_ref,
                        class="chat-input",
                        contenteditable="true",
                        role="textbox",
                        data-placeholder="Напишите сообщение...",
                        on:input=on_input,
                        on:focus=on_input_focus,
                        on:click=on_input_click,
                        on:keydown=on_keydown,
                        on:paste=on_paste,
                    )

                    // Expand editor button at top-right inside input (visible only from 7th line or when scrollbar appears)
                    button(
                        class=move || {
                            let line_count = draft_message.with(|msg| msg.split('\n').count());
                            if has_scrollbar.get() || line_count >= 7 {
                                "chat-expand-btn visible"
                            } else {
                                "chat-expand-btn"
                            }
                        },
                        r#type="button",
                        title="Развернуть полноразмерный редактор текста",
                        on:click=on_open_editor,
                    ) {
                        img(src="/public/arrow1.svg", alt="Развернуть")
                    }

                    // Emoji button pinned at bottom-right inside input (always visible)
                    button(
                        class=move || if is_emoji_modal_open.get() {
                            "chat-emoji-btn active"
                        } else {
                            "chat-emoji-btn"
                        },
                        r#type="button",
                        title="Выбрать эмодзи",
                        on:click=on_toggle_emoji,
                    ) {
                        svg(viewBox="0 0 24 24") {
                            circle(cx="12", cy="12", r="10")
                            path(d="M8 14s1.5 2 4 2 4-2 4-2")
                            line(x1="9", y1="9", x2="9.01", y2="9")
                            line(x1="15", y1="9", x2="15.01", y2="9")
                        }
                    }
                }
                button(
                    class="send-btn",
                    r#type="submit",
                    title="Отправить",
                    on:click=move |_| {
                        is_attach_menu_open.set(false);
                        is_emoji_modal_open.set(false);
                        is_emoji_expanded.set(false);
                    },
                ) {
                    svg(
                        xmlns="http://www.w3.org/2000/svg",
                        width="18",
                        height="18",
                        viewBox="0 0 24 24",
                        fill="none",
                        stroke="currentColor",
                        stroke-width="2",
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
}
