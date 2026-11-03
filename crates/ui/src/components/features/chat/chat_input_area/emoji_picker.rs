//! Emoji picker popup with categories, Russian/English search, Twemoji images, and size toggle.

use sycamore::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use super::super::emoji_search::{matches_emoji_query, EMOJI_GROUPS};
use crate::components::{
    common::{
        emoji_to_pack_url, handle_emoji_img_error, insert_emoji_into_editor,
        serialize_contenteditable, UiModalContext, ALL_EMOJI_PACKS, EmojiPack,
    },
    ui::icons::CloseSmallIcon,
};

#[component(inline_props)]
pub(crate) fn EmojiPicker(
    is_open: Signal<bool>,
    is_expanded: Signal<bool>,
    search: Signal<String>,
    category: Signal<String>,
    editor_ref: NodeRef,
    has_scrollbar: Signal<bool>,
    draft_message: Signal<String>,
) -> View {
    let _modal_context = use_context::<UiModalContext>();
    let active_pack = create_signal(EmojiPack::load_saved());

    view! {
        (move || if is_open.get() {
            view! {
                div(
                    class=move || {
                        if is_expanded.get() {
                            "chat-emoji-modal is-expanded"
                        } else {
                            "chat-emoji-modal"
                        }
                    },
                    on:click=move |e: web_sys::MouseEvent| e.stop_propagation(),
                ) {
                    div(class="emoji-modal-header") {
                        span(class="emoji-modal-title") { "😊 Эмодзи" }
                        div(class="emoji-modal-actions") {
                            button(
                                r#type="button",
                                class="emoji-expand-toggle-btn",
                                title=move || if is_expanded.get() {
                                    "Вернуть обычный размер"
                                } else {
                                    "Развернуть окно эмодзи"
                                },
                                on:click=move |e: web_sys::MouseEvent| {
                                    e.stop_propagation();
                                    let next = !is_expanded.get();
                                    is_expanded.set(next);
                                },
                            ) {
                                (move || if is_expanded.get() {
                                    view! {
                                        img(src="/public/arrow2.svg", alt="Свернуть")
                                    }
                                } else {
                                    view! {
                                        img(src="/public/arrow1.svg", alt="Развернуть")
                                    }
                                })
                            }
                            button(
                                r#type="button",
                                class="emoji-close-btn",
                                title="Закрыть окно эмодзи (Esc)",
                                on:click=move |e: web_sys::MouseEvent| {
                                    e.stop_propagation();
                                    is_open.set(false);
                                    is_expanded.set(false);
                                },
                            ) {
                                CloseSmallIcon()
                            }
                        }
                    }
                    div(class="emoji-modal-search") {
                        span(class="emoji-search-icon") {
                            svg(
                                viewBox="0 0 24 24",
                                width="14",
                                height="14",
                                fill="none",
                                stroke="currentColor",
                                stroke-width="2.2",
                                stroke-linecap="round",
                                stroke-linejoin="round",
                            ) {
                                circle(cx="11", cy="11", r="7")
                                line(x1="20", y1="20", x2="16.35", y2="16.35")
                            }
                        }
                        input(
                            r#type="text",
                            placeholder="Поиск эмодзи...",
                            bind:value=search,
                        )
                    }
                    div(class="emoji-pack-selector") {
                        (ALL_EMOJI_PACKS.iter().map(|p| {
                            let pack_item = *p;
                            let name = pack_item.short_name();
                            let icon_preview = match pack_item {
                                EmojiPack::Fluent3D => "💎",
                                EmojiPack::Apple => "🍏",
                                EmojiPack::Noto => "🤖",
                                EmojiPack::Twemoji => "🐦",
                            };
                            view! {
                                button(
                                    r#type="button",
                                    class=move || if active_pack.get() == pack_item {
                                        "emoji-pack-btn active"
                                    } else {
                                        "emoji-pack-btn"
                                    },
                                    title=pack_item.description(),
                                    on:click=move |_| {
                                        active_pack.set(pack_item);
                                        pack_item.save();
                                    },
                                ) {
                                    span(class="emoji-pack-badge") { (icon_preview) }
                                    span { (name) }
                                }
                            }
                        }).collect::<Vec<_>>())
                    }
                    div(class="emoji-category-tabs") {
                        (move || {
                            let pack = active_pack.get();
                            EMOJI_GROUPS
                                .iter()
                                .map(|(cat_id, cat_icon, cat_title, _)| {
                                    let id_str = (*cat_id).to_string();
                                    let id_check = id_str.clone();
                                    let icon_str = (*cat_icon).to_string();
                                    let icon_url = emoji_to_pack_url(cat_icon, pack);
                                    let title_str = (*cat_title).to_string();
                                    view! {
                                        button(
                                            r#type="button",
                                            class=move || if category.get_clone() == id_check {
                                                "emoji-cat-btn active"
                                            } else {
                                                "emoji-cat-btn"
                                            },
                                            title=title_str,
                                            on:click=move |_| category.set(id_str.clone()),
                                        ) {
                                            img(
                                                src=icon_url,
                                                alt=icon_str,
                                                width="18",
                                                height="18",
                                                draggable="false",
                                                on:error=handle_emoji_img_error,
                                            )
                                        }
                                    }
                                })
                                .collect::<Vec<View>>()
                        })
                    }
                    div(class="emoji-modal-body") {
                        (move || {
                            let pack = active_pack.get();
                            let query = search.get_clone().trim().to_lowercase();
                            let active_cat = category.get_clone();

                            let list: Vec<&'static emojis::Emoji> = if !query.is_empty() {
                                emojis::iter()
                                    .filter(|e| e.unicode_version().major() <= 15 && matches_emoji_query(e, &query))
                                    .take(150)
                                    .collect()
                            } else if let Some((_, _, _, grp)) = EMOJI_GROUPS.iter().find(|(id, _, _, _)| *id == active_cat) {
                                grp.emojis().filter(|e| e.unicode_version().major() <= 15).collect()
                            } else {
                                emojis::Group::SmileysAndEmotion.emojis().filter(|e| e.unicode_version().major() <= 15).collect()
                            };

                            if list.is_empty() {
                                view! {
                                    div(class="emoji-empty-state") {
                                        "Эмодзи не найдены"
                                    }
                                }
                            } else {
                                let items: Vec<View> = list
                                    .into_iter()
                                    .map(|e| {
                                        let emoji_char = e.as_str().to_string();
                                        let emoji_click = emoji_char.clone();
                                        let emoji_title = e.name().to_string();
                                        let emoji_title_click = emoji_title.clone();
                                        let emoji_url = emoji_to_pack_url(&emoji_char, pack);
                                        let emoji_url_click = emoji_url.clone();
                                        let pack_id = pack.as_str();
                                        view! {
                                            button(
                                                r#type="button",
                                                class="emoji-grid-item",
                                                title=emoji_title,
                                                on:click=move |_| {
                                                    let to_add = emoji_click.clone();
                                                    let url = emoji_url_click.clone();
                                                    let title = emoji_title_click.clone();
                                                    if let Some(node) = editor_ref.try_get() {
                                                        if let Ok(editor) = node.dyn_into::<HtmlElement>() {
                                                            insert_emoji_into_editor(&editor, &to_add, &url, &title, pack_id);
                                                            let serialized = serialize_contenteditable(&editor);
                                                            if serialized.trim().is_empty() {
                                                                let _ = editor.set_attribute("data-empty", "true");
                                                            } else {
                                                                let _ = editor.remove_attribute("data-empty");
                                                            }
                                                            let overflowed = editor.scroll_height() > 142;
                                                            has_scrollbar.set(overflowed);
                                                            draft_message.set(serialized);
                                                        } else {
                                                            draft_message.update(|msg| msg.push_str(&format!("<e:{}:{to_add}>", pack_id)));
                                                        }
                                                    } else {
                                                        draft_message.update(|msg| msg.push_str(&format!("<e:{}:{to_add}>", pack_id)));
                                                    }
                                                },
                                            ) {
                                                img(
                                                    src=emoji_url,
                                                    alt=emoji_char,
                                                    width="24",
                                                    height="24",
                                                    loading="lazy",
                                                    draggable="false",
                                                    on:error=handle_emoji_img_error,
                                                )
                                            }
                                        }
                                    })
                                    .collect();

                                view! {
                                    div(class="emoji-grid") {
                                        (items)
                                    }
                                }
                            }
                        })
                    }
                }
            }
        } else {
            view! {}
        })
    }
}
