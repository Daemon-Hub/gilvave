use futures_util::StreamExt;
use gilvave_core::dto::message::MessageView;
use sycamore::{futures::spawn_local_scoped, prelude::*, web::queue_microtask};
use tauri_sys::event::listen;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlElement, SubmitEvent};

use super::message_item::MessageItem;
use crate::{
    components::common::{ChannelContext, ServerContext, UiModalContext, UserProfileContext},
    gateway::service::WsService,
    utils::{get_local_offset, to_local_datetime},
};

pub fn adjust_textarea_height(el: &HtmlElement, min_h: i32, max_h: i32) -> bool {
    let style = el.style();
    let _ = style.set_property("height", "auto");
    let sh = el.scroll_height();
    let target_h = sh.clamp(min_h, max_h);
    let _ = style.set_property("height", &format!("{target_h}px"));
    let has_scrollbar = sh > max_h;
    let overflow = if has_scrollbar { "auto" } else { "hidden" };
    let _ = style.set_property("overflow-y", overflow);
    has_scrollbar
}

const EMOJI_CATALOG: &[(&str, &str, &[(&str, &str)])] = &[
    (
        "smileys",
        "😀",
        &[
            ("😀", "улыбка радость smile"),
            ("😃", "улыбка смех smile"),
            ("😄", "смех радость happy"),
            ("😁", "ухмылка grin"),
            ("😆", "хаха смех laugh"),
            ("😅", "нервный смех sweat"),
            ("🤣", "рофл смех до слез rofl"),
            ("😂", "слезы радости смех joy"),
            ("🙂", "легкая улыбка slight smile"),
            ("😉", "подмигивание wink"),
            ("😊", "смущение мило blush"),
            ("😇", "ангел нимб innocent"),
            ("🥰", "любовь сердечки in love"),
            ("😍", "влюбленный глаза сердечки heart eyes"),
            ("🤩", "восторг звезды star struck"),
            ("😘", "поцелуй kiss"),
            ("😋", "вкусно ням yum"),
            ("😜", "язык подмигивание crazy"),
            ("🤪", "безумие веселье zany"),
            ("😎", "крутой очки cool"),
            ("🤓", "ботан очки nerd"),
            ("🧐", "монокль анализ monocle"),
            ("🤔", "думаю размышление thinking"),
            ("🫡", "честь салют salute"),
            ("🤫", "тихо секрет shh"),
            ("🫠", "таю неловко melting"),
            ("😏", "ухмылка хитрый smirk"),
            ("😒", "недовольство unamused"),
            ("🙄", "закатил глаза eye roll"),
            ("😮‍💨", "выдох усталость sigh"),
            ("😔", "грусть печаль pensive"),
            ("😴", "сон спать sleep"),
            ("🤯", "взрыв мозга шок mind blown"),
            ("🥳", "праздник вечеринка party"),
            ("🥺", "пожалуйста милые глаза pleading"),
            ("😭", "плач рыдание sob"),
            ("😱", "крик ужас шок scream"),
            ("😡", "злость гнев angry"),
            ("🤬", "ругань злость swear"),
            ("🤡", "клоун clown"),
            ("👻", "призрак бу ghost"),
            ("💀", "череп мертв skull"),
        ],
    ),
    (
        "gestures",
        "👋",
        &[
            ("👋", "привет пока махать wave"),
            ("🤚", "ладонь стоп raised back hand"),
            ("✋", "рука стоп пять high five"),
            ("👌", "ок отлично ok"),
            ("🤌", "италия жест pinched fingers"),
            ("✌️", "мир победа peace victory"),
            ("🤞", "удача скрестить пальцы crossed fingers"),
            ("🫰", "сердечко пальцами hand heart"),
            ("🤟", "рок люблю love you"),
            ("🤘", "рок коза rock"),
            ("🤙", "позвони шака call me"),
            ("👍", "лайк супер класс thumbs up"),
            ("👎", "дизлайк плохо thumbs down"),
            ("👊", "кулак броуфист fist"),
            ("👏", "аплодисменты хлопать clap"),
            ("🙌", "ура руки вверх raised hands"),
            ("🫶", "сердце руками heart hands"),
            ("🤝", "рукопожатие сделка handshake"),
            ("🙏", "спасибо пожалуйста молитва pray"),
            ("💪", "сила бицепс muscle"),
            ("👀", "глаза смотрю eyes"),
            ("🧠", "мозг ум brain"),
        ],
    ),
    (
        "hearts",
        "❤️",
        &[
            ("❤️", "красное сердце любовь red heart"),
            ("🧡", "оранжевое сердце orange heart"),
            ("💛", "желтое сердце yellow heart"),
            ("💚", "зеленое сердце green heart"),
            ("💙", "синее сердце blue heart"),
            ("💜", "фиолетовое сердце purple heart"),
            ("🖤", "черное сердце black heart"),
            ("🤍", "белое сердце white heart"),
            ("💔", "разбитое сердце broken heart"),
            ("❤️‍🔥", "горящее сердце страсть heart on fire"),
            ("💖", "сверкающее сердце sparkling heart"),
            ("💘", "сердце со стрелой cupid"),
            ("💝", "сердце с лентой gift heart"),
            ("💯", "сто процентов идеал 100"),
            ("💢", "злость гнев anger"),
            ("💥", "взрыв бум boom"),
            ("💫", "головокружение звезда dizzy"),
            ("💬", "облачко чат сообщение speech"),
        ],
    ),
    (
        "animals",
        "🐱",
        &[
            ("🐱", "кот кошка cat"),
            ("🐶", "собака пес dog"),
            ("🦊", "лиса хитрая fox"),
            ("🐻", "медведь мишка bear"),
            ("🐼", "панда panda"),
            ("🐨", "коала koala"),
            ("🦁", "лев царь lion"),
            ("🐯", "тигр tiger"),
            ("🐸", "лягушка пепе frog"),
            ("🐵", "обезьяна monkey"),
            ("🐧", "пингвин linux penguin"),
            ("🦉", "сова мудрость owl"),
            ("🦋", "бабочка butterfly"),
            ("🐢", "черепаха turtle"),
            ("🐙", "осьминог octopus"),
            ("🦀", "краб раст rust crab"),
            ("🐳", "кит докер whale"),
            ("🦄", "единорог магия unicorn"),
        ],
    ),
    (
        "food",
        "🍕",
        &[
            ("🍕", "пицца еда pizza"),
            ("🍔", "бургер гамбургер burger"),
            ("🍟", "картошка фри fries"),
            ("🌭", "хотдог hotdog"),
            ("🍿", "попкорн кино popcorn"),
            ("🍩", "пончик сладкое donut"),
            ("🍪", "печенье куки cookie"),
            ("🎂", "торт день рождения cake"),
            ("🍰", "пирожное тортик shortcake"),
            ("🍫", "шоколад chocolate"),
            ("🍓", "клубника ягода strawberry"),
            ("🍒", "вишня черешня cherry"),
            ("🍉", "арбуз лето watermelon"),
            ("☕", "кофе чай утро coffee"),
            ("🧋", "бабл ти чай boba"),
            ("🥤", "газировка напиток soda"),
            ("🍺", "пиво кружка beer"),
            ("🥂", "бокалы праздник cheers"),
        ],
    ),
    (
        "misc",
        "🔥",
        &[
            ("🔥", "огонь топ жарко fire"),
            ("✨", "блестки магия звезды sparkles"),
            ("🌟", "звезда сияние star"),
            ("⚡", "молния быстро энергия zap"),
            ("🌈", "радуга rainbow"),
            ("☀️", "солнце тепло sun"),
            ("🌙", "луна ночь месяц moon"),
            ("🎉", "хлопушка праздник ура tada"),
            ("🎊", "конфетти праздник confetti"),
            ("🎈", "шарик праздник balloon"),
            ("🏆", "кубок победа чемпион trophy"),
            ("🥇", "первое место золото медаль gold"),
            ("🎮", "геймпад игры джойстик game"),
            ("🎧", "наушники музыка headphones"),
            ("🎵", "музыка нота music"),
            ("🚀", "ракета старт космос rocket"),
            ("💻", "ноутбук код работа laptop"),
            ("💎", "алмаз бриллиант gem"),
            ("🔒", "замок безопасность lock"),
            ("✅", "галочка готово да check"),
            ("❌", "крестик отмена нет cross"),
        ],
    ),
];

#[derive(Clone, PartialEq)]
struct EnrichedMessage {
    message: MessageView,
    avatar_url: String,
    is_first: bool,
    is_last: bool,
    date_divider: Option<String>,
}

fn format_date_divider(date: time::Date) -> String {
    let month_str = match date.month() {
        time::Month::January => "января",
        time::Month::February => "февраля",
        time::Month::March => "марта",
        time::Month::April => "апреля",
        time::Month::May => "мая",
        time::Month::June => "июня",
        time::Month::July => "июля",
        time::Month::August => "августа",
        time::Month::September => "сентября",
        time::Month::October => "октября",
        time::Month::November => "ноября",
        time::Month::December => "декабря",
    };

    let weekday_str = match date.weekday() {
        time::Weekday::Monday => "понедельник",
        time::Weekday::Tuesday => "вторник",
        time::Weekday::Wednesday => "среда",
        time::Weekday::Thursday => "четверг",
        time::Weekday::Friday => "пятница",
        time::Weekday::Saturday => "суббота",
        time::Weekday::Sunday => "воскресенье",
    };

    let now = to_local_datetime(time::OffsetDateTime::now_utc());
    let today = now.date();
    let yesterday = (now - time::Duration::days(1)).date();

    if date == today {
        format!("Сегодня, {} {}", date.day(), month_str)
    } else if date == yesterday {
        format!("Вчера, {} {}", date.day(), month_str)
    } else if date.year() == today.year() {
        format!("{} {}, {}", date.day(), month_str, weekday_str)
    } else {
        format!("{} {} {} г.", date.day(), month_str, date.year())
    }
}

fn send_channel_message(channel_context: ChannelContext, draft_message: Signal<String>) {
    let Some(channel_id) = channel_context.current.with(|c| c.as_ref().map(|ch| ch.id)) else {
        return;
    };

    let sanitized = match draft_message.with(|msg| gilvave_core::validation::validate_message(msg)) {
        Ok(s) => s,
        Err(e) => {
            web_sys::console::warn_1(&format!("[MESSAGES_AREA] on_submit ignored: {e}").into());
            return;
        }
    };

    spawn_local_scoped(async move {
        web_sys::console::log_1(&format!("[MESSAGES_AREA] invoking MessageCreate for {channel_id}").into());
        let res = WsService::message_create(channel_id, sanitized).await;
        web_sys::console::log_1(&format!("[MESSAGES_AREA] MessageCreate res: {res:?}").into());
    });
    draft_message.set(String::new());
}

#[component(inline_props)]
pub fn ChatInputArea<F>(on_send: F) -> View
where
    F: Fn() + Copy + 'static,
{
    let modal_context = use_context::<UiModalContext>();
    let draft_message = modal_context.draft_message;
    let textarea_ref = create_node_ref();

    let has_scrollbar = create_signal(false);
    let is_attach_menu_open = create_signal(false);
    let is_emoji_modal_open = create_signal(false);
    let emoji_search = create_signal(String::new());
    let emoji_category = create_signal("all".to_string());

    create_effect(move || {
        draft_message.track();
        let is_empty = draft_message.with(|m| m.is_empty());
        if is_empty {
            has_scrollbar.set(false);
        }
        if let Some(node) = textarea_ref.try_get() {
            let el: HtmlElement = node.unchecked_into();
            queue_microtask(move || {
                let overflowed = adjust_textarea_height(&el, 42, 142);
                has_scrollbar.set(overflowed);
            });
        }
    });

    let on_submit = move |event: SubmitEvent| {
        event.prevent_default();
        is_attach_menu_open.set(false);
        is_emoji_modal_open.set(false);
        on_send();
    };

    let on_input = move |event: Event| {
        if let Some(target) = event.current_target() {
            let el: HtmlElement = target.unchecked_into();
            let overflowed = adjust_textarea_height(&el, 42, 142);
            has_scrollbar.set(overflowed);
        }
    };

    let on_keydown = move |event: web_sys::KeyboardEvent| {
        if event.key() == "Enter" && !event.shift_key() {
            event.prevent_default();
            is_attach_menu_open.set(false);
            is_emoji_modal_open.set(false);
            on_send();
        }
    };

    let on_open_editor = move |_| {
        is_attach_menu_open.set(false);
        is_emoji_modal_open.set(false);
        modal_context.is_message_editor_open.set(true);
    };

    let on_toggle_attach = move |e: web_sys::MouseEvent| {
        e.stop_propagation();
        let next = !is_attach_menu_open.get();
        is_attach_menu_open.set(next);
        if next {
            is_emoji_modal_open.set(false);
        }
    };

    let on_toggle_emoji = move |e: web_sys::MouseEvent| {
        e.stop_propagation();
        let next = !is_emoji_modal_open.get();
        is_emoji_modal_open.set(next);
        if next {
            is_attach_menu_open.set(false);
        }
    };

    let on_file_selected = move |e: Event| {
        if let Some(target) = e.target() {
            if let Ok(input) = target.dyn_into::<web_sys::HtmlInputElement>() {
                input.set_value("");
            }
        }
        is_attach_menu_open.set(false);
    };

    view! {
        form(class="input-area", on:submit=on_submit) {
            div(class="chat-input-wrapper") {
                // Click-outside backdrop for attachment menu or emoji modal
                (move || if is_attach_menu_open.get() || is_emoji_modal_open.get() {
                    view! {
                        div(
                            class="chat-popover-backdrop",
                            on:click=move |_| {
                                is_attach_menu_open.set(false);
                                is_emoji_modal_open.set(false);
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

                // Hidden file inputs for the attachment menu
                input(
                    id="chat-attach-media-input",
                    r#type="file",
                    accept="image/*,video/*",
                    multiple=true,
                    class="chat-hidden-file-input",
                    on:change=on_file_selected,
                )
                input(
                    id="chat-attach-file-input",
                    r#type="file",
                    multiple=true,
                    class="chat-hidden-file-input",
                    on:change=on_file_selected,
                )

                // Attachment popup menu (bottom-left above paperclip button)
                (move || if is_attach_menu_open.get() {
                    view! {
                        div(
                            class="chat-attach-menu",
                            on:click=move |e: web_sys::MouseEvent| e.stop_propagation(),
                        ) {
                            label(
                                r#for="chat-attach-media-input",
                                class="chat-attach-menu-item",
                            ) {
                                span(class="attach-item-icon media") {
                                    svg(viewBox="0 0 24 24") {
                                        rect(x="3", y="3", width="18", height="18", rx="3", ry="3")
                                        circle(cx="8.5", cy="8.5", r="1.5")
                                        polyline(points="21 15 16 10 5 21")
                                    }
                                }
                                span(class="attach-item-label") { "Фото или видео" }
                            }
                            label(
                                r#for="chat-attach-file-input",
                                class="chat-attach-menu-item",
                            ) {
                                span(class="attach-item-icon file") {
                                    svg(viewBox="0 0 24 24") {
                                        path(d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z")
                                        polyline(points="14 2 14 8 20 8")
                                    }
                                }
                                span(class="attach-item-label") { "Файл" }
                            }
                        }
                    }
                } else {
                    view! {}
                })

                // Emoji picker modal window (bottom-right above emoji button)
                (move || if is_emoji_modal_open.get() {
                    view! {
                        div(
                            class="chat-emoji-modal",
                            on:click=move |e: web_sys::MouseEvent| e.stop_propagation(),
                        ) {
                            div(class="emoji-modal-header") {
                                span(class="emoji-modal-title") { "😊 Эмодзи" }
                                button(
                                    r#type="button",
                                    class="modal-close-icon-btn",
                                    title="Закрыть",
                                    on:click=move |_| is_emoji_modal_open.set(false),
                                ) { "✕" }
                            }
                            div(class="emoji-modal-search") {
                                span(class="emoji-search-icon") {
                                    svg(viewBox="0 0 24 24") {
                                        circle(cx="11", cy="11", r="7")
                                        line(x1="20", y1="20", x2="16.35", y2="16.35")
                                    }
                                }
                                input(
                                    r#type="text",
                                    placeholder="Поиск эмодзи...",
                                    bind:value=emoji_search,
                                )
                            }
                            div(class="emoji-category-tabs") {
                                button(
                                    r#type="button",
                                    class=move || if emoji_category.get_clone() == "all" {
                                        "emoji-cat-btn active"
                                    } else {
                                        "emoji-cat-btn"
                                    },
                                    title="Все эмодзи",
                                    on:click=move |_| emoji_category.set("all".to_string()),
                                ) { "Все" }
                                (EMOJI_CATALOG
                                    .iter()
                                    .map(|(cat_id, cat_icon, _)| {
                                        let id_str = (*cat_id).to_string();
                                        let id_check = id_str.clone();
                                        let icon_str = (*cat_icon).to_string();
                                        view! {
                                            button(
                                                r#type="button",
                                                class=move || if emoji_category.get_clone() == id_check {
                                                    "emoji-cat-btn active"
                                                } else {
                                                    "emoji-cat-btn"
                                                },
                                                on:click=move |_| emoji_category.set(id_str.clone()),
                                            ) { (icon_str) }
                                        }
                                    })
                                    .collect::<Vec<View>>())
                            }
                            div(class="emoji-modal-body") {
                                (move || {
                                    let query = emoji_search.get_clone().trim().to_lowercase();
                                    let active_cat = emoji_category.get_clone();
                                    let mut items: Vec<View> = Vec::new();

                                    for (cat_id, _, list) in EMOJI_CATALOG {
                                        if active_cat != "all" && active_cat != *cat_id {
                                            continue;
                                        }
                                        for (ch, keywords) in *list {
                                            if !query.is_empty()
                                                && !keywords.contains(&query)
                                                && !ch.contains(&query)
                                            {
                                                continue;
                                            }
                                            let emoji_str = (*ch).to_string();
                                            let emoji_click = emoji_str.clone();
                                            items.push(view! {
                                                button(
                                                    r#type="button",
                                                    class="emoji-grid-item",
                                                    on:click=move |_| {
                                                        let to_add = emoji_click.clone();
                                                        draft_message.update(|msg| msg.push_str(&to_add));
                                                    },
                                                ) { (emoji_str) }
                                            });
                                        }
                                    }

                                    if items.is_empty() {
                                        view! {
                                            div(class="emoji-empty-state") {
                                                "Эмодзи не найдены"
                                            }
                                        }
                                    } else {
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

                textarea(
                    r#ref=textarea_ref,
                    class="chat-input",
                    rows="1",
                    placeholder="Напишите сообщение...",
                    bind:value=draft_message,
                    on:input=on_input,
                    on:keydown=on_keydown,
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
            button(class="send-btn", r#type="submit", title="Отправить") {
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

#[component()]
pub fn MessagesArea() -> View {
    let channel_context = use_context::<ChannelContext>();
    let server_context = use_context::<ServerContext>();
    let user_profile = use_context::<UserProfileContext>();
    let modal_context = use_context::<UiModalContext>();
    let channel_name = create_memo(move || {
        channel_context
            .current
            .with(|c| c.as_ref().map_or_else(|| "<UNKNOWN>".into(), |ch| ch.name.clone()))
    });

    let draft_message = modal_context.draft_message;
    let container = create_node_ref();
    let on_scroll = move |event: Event| {
        let el: HtmlElement = match event.current_target() {
            Some(t) => t.unchecked_into(),
            None => return,
        };

        const SCROLL_THRESHOLD: i32 = 1;

        let top = el.scroll_top();
        let sh = el.scroll_height();
        let ch = el.client_height();

        let is_top = top <= 0;
        let is_bottom = (sh - ch - top).abs() <= SCROLL_THRESHOLD;

        if is_top {
            console_log!("top message");
            spawn_local_scoped(async move {
                let Some(ch_id) = channel_context.current.with(|c| c.as_ref().map(|ch| ch.id)) else {
                    return;
                };
                let Some(first_dt) = channel_context.messages.with(|m| m.first().map(|msg| msg.created_at)) else {
                    return;
                };
                let _ = WsService::channel_history_before(ch_id, first_dt).await;
            });
        }
        if is_bottom {
            console_log!("bottom message");
            spawn_local_scoped(async move {
                let Some(ch_id) = channel_context.current.with(|c| c.as_ref().map(|ch| ch.id)) else {
                    return;
                };
                let Some(last_dt) = channel_context.messages.with(|m| m.last().map(|msg| msg.created_at)) else {
                    return;
                };
                let _ = WsService::channel_history_after(ch_id, last_dt).await;
            });
        }
    };

    create_effect(move || {
        channel_context.current.track();
        channel_context.messages.set(vec![]);
    });

    spawn_local_scoped(async move {
        web_sys::console::log_1(&"[MESSAGES_AREA] listening for message_new...".into());
        let mut events = match listen::<MessageView>("message_new").await {
            Ok(s) => {
                web_sys::console::log_1(&"[MESSAGES_AREA] successfully obtained message_new stream".into());
                s
            }
            Err(e) => {
                web_sys::console::error_1(&format!("[MESSAGES_AREA] failed to listen message_new: {e:?}").into());
                return;
            }
        };
        while let Some(event) = events.next().await {
            web_sys::console::log_1(&format!("[MESSAGES_AREA] received message_new event: {:?}", event.payload).into());
            let Some(node) = container.try_get() else {
                continue;
            };
            let el = node.unchecked_into::<HtmlElement>();
            let old_scroll_top = el.scroll_top();
            let old_scroll_height = el.scroll_height();
            let old_client_height = el.client_height();

            const SCROLL_THRESHOLD: i32 = 1;
            let is_bottom =
                (old_scroll_height - old_client_height - old_scroll_top).abs() <= SCROLL_THRESHOLD;

            channel_context
                .messages
                .update(|list| list.push(event.payload));

            queue_microtask(move || {
                if is_bottom {
                    let delta = el.scroll_height() - old_scroll_height;
                    el.set_scroll_top(old_scroll_top + delta);
                }
            });
        }
    });
    spawn_local_scoped(async move {
        web_sys::console::log_1(&"[MESSAGES_AREA] listening for channel_history_before...".into());
        let mut events = match listen::<Vec<MessageView>>("channel_history_before").await {
            Ok(s) => {
                web_sys::console::log_1(&"[MESSAGES_AREA] successfully obtained channel_history_before stream".into());
                s
            }
            Err(e) => {
                web_sys::console::error_1(&format!("[MESSAGES_AREA] failed to listen channel_history_before: {e:?}").into());
                return;
            }
        };
        while let Some(event) = events.next().await {
            web_sys::console::log_1(&format!("[MESSAGES_AREA] received channel_history_before with {} items", event.payload.len()).into());
            let Some(node) = container.try_get() else {
                continue;
            };
            let el = node.unchecked_into::<HtmlElement>();
            let old_scroll_top = el.scroll_top();
            let old_scroll_height = el.scroll_height();

            channel_context.messages.update(|list| {
                for msg in event.payload.into_iter() {
                    list.insert(0, msg);
                }
            });

            // Ждём, пока Sycamore реально вольёт узлы в DOM.
            // queue_microtask достаточно, т.к. эффекты Sycamore выполняются
            // в микротасках. Для надёжности можно использовать RAF.
            queue_microtask(move || {
                let delta = el.scroll_height() - old_scroll_height;
                el.set_scroll_top(old_scroll_top + delta);
            });
        }
    });
    spawn_local_scoped(async move {
        let mut events = listen::<Vec<MessageView>>("channel_history_after")
            .await
            .unwrap();
        while let Some(mut event) = events.next().await {
            channel_context
                .messages
                .update(|list| list.append(event.payload.as_mut()));
        }
    });

    let on_back = move |_| {
        channel_context.current.set(None);
    };

    let enriched_messages = create_memo(move || {
        channel_context.messages.with(|msgs| {
            server_context.members.with(|members| {
                user_profile.username.with(|my_username| {
                    user_profile.avatar.with(|my_avatar| {
                        let len = msgs.len();
                        let local_offset = get_local_offset();
                        let local_dts: Vec<time::OffsetDateTime> = msgs
                            .iter()
                            .map(|m| m.created_at.to_offset(local_offset))
                            .collect();

                        msgs.iter()
                            .enumerate()
                            .map(|(i, m)| {
                                let m_local_dt = local_dts[i];
                                let m_date = m_local_dt.date();
                                let is_new_day = if i == 0 {
                                    true
                                } else {
                                    local_dts[i - 1].date() != m_date
                                };
                                let date_divider = if is_new_day {
                                    Some(format_date_divider(m_date))
                                } else {
                                    None
                                };

                                let is_first = if i == 0 {
                                    true
                                } else {
                                    let prev_dt = local_dts[i - 1];
                                    let prev_m = &msgs[i - 1];
                                    prev_m.author_name != m.author_name
                                        || (m.author_id.is_some()
                                            && prev_m.author_id != m.author_id)
                                        || prev_dt.date() != m_date
                                        || (m_local_dt - prev_dt) > time::Duration::minutes(5)
                                };

                                let is_last = if i + 1 == len {
                                    true
                                } else {
                                    let next_dt = local_dts[i + 1];
                                    let next_m = &msgs[i + 1];
                                    next_m.author_name != m.author_name
                                        || (m.author_id.is_some()
                                            && next_m.author_id != m.author_id)
                                        || next_dt.date() != m_date
                                        || (next_dt - m_local_dt) > time::Duration::minutes(5)
                                };

                                let avatar = if !my_avatar.is_empty()
                                    && m.author_name == *my_username
                                {
                                    my_avatar.clone()
                                } else {
                                    members
                                        .iter()
                                        .find(|mem| {
                                            mem.username == m.author_name
                                                || (m.author_id.is_some()
                                                    && Some(mem.user_id) == m.author_id)
                                        })
                                        .map(|mem| mem.avatar.clone())
                                        .unwrap_or_else(|| {
                                            if m.author_name == *my_username {
                                                my_avatar.clone()
                                            } else {
                                                String::new()
                                            }
                                        })
                                };

                                EnrichedMessage {
                                    message: m.clone(),
                                    avatar_url: avatar,
                                    is_first,
                                    is_last,
                                    date_divider,
                                }
                            })
                            .collect::<Vec<_>>()
                    })
                })
            })
        })
    });

    view! {
        div(class="messages-area") {
            div(class="chat-header") {
                button(class="chat-back-btn", on:click=on_back) {
                    svg(
                        xmlns="http://www.w3.org/2000/svg",
                        width="18",
                        height="18",
                        viewBox="0 0 24 24",
                        fill="none",
                        stroke="currentColor",
                        stroke-width="2.5",
                        stroke-linecap="round",
                        stroke-linejoin="round",
                    ) {
                        polyline(points="15 18 9 12 15 6")
                    }
                    span { "Каналы" }
                }
                div(class="chat-header-title") {
                    span(class="channel-hash") { "#" }
                    span(class="channel-name") { (channel_name) }
                }
            }

            div(class="messages-list", r#ref=container, on:scroll=on_scroll) {
                WelcomeMessage(channel_name=channel_name)

                Keyed(
                    list=enriched_messages,
                    key=|em| (em.message.id, em.is_first, em.is_last, em.date_divider.clone(), em.avatar_url.clone()),
                    view=|em| {
                        let date_view = if let Some(date_str) = em.date_divider {
                            view! {
                                div(class="chat-date-divider") {
                                    span(class="date-badge") { (date_str) }
                                }
                            }
                        } else {
                            view! {}
                        };

                        view! {
                            (date_view)
                            MessageItem(
                                message_view=em.message,
                                avatar_url=em.avatar_url,
                                is_first_in_group=em.is_first,
                                is_last_in_group=em.is_last,
                            )
                        }
                    },
                )
            }

            ChatInputArea(on_send=move || send_channel_message(channel_context, draft_message))
        }
    }
}

#[component(inline_props)]
fn WelcomeMessage(channel_name: ReadSignal<String>) -> View {
    view! {
        div(class="welcome-message") {
            p { (format!("Добро пожаловать на канал {channel_name}!")) }
        }
    }
}

#[component]
pub fn ExpandedMessageEditorModal() -> View {
    let modal_context = use_context::<UiModalContext>();
    let channel_context = use_context::<ChannelContext>();
    let server_context = use_context::<ServerContext>();

    let draft_message = modal_context.draft_message;

    let target_title = create_memo(move || {
        let in_server = server_context.current.with(|s| s.is_some());
        if in_server {
            channel_context.current.with(|c| {
                c.as_ref()
                    .map_or_else(|| "Редактор сообщения".to_string(), |ch| format!("#{}", ch.name))
            })
        } else {
            modal_context.selected_dm_name.with(|dm| {
                dm.as_ref()
                    .map_or_else(|| "Редактор сообщения".to_string(), |name| name.clone())
            })
        }
    });

    let char_count_text = create_memo(move || {
        let count = draft_message.with(|msg| msg.chars().count());
        format!("{count} / {}", gilvave_core::validation::MAX_MESSAGE_CHARS)
    });

    let is_over_limit = create_memo(move || {
        draft_message.with(|msg| msg.chars().count() > gilvave_core::validation::MAX_MESSAGE_CHARS)
    });

    let counter_class = move || {
        if is_over_limit.get() {
            "editor-char-counter over-limit"
        } else {
            "editor-char-counter"
        }
    };

    let on_collapse = move |_| {
        modal_context.is_message_editor_open.set(false);
    };

    let on_overlay_click = move |_| {
        modal_context.is_message_editor_open.set(false);
    };

    let on_window_click = move |e: web_sys::MouseEvent| {
        e.stop_propagation();
    };

    let do_send = move || {
        let in_server = server_context.current.with(|s| s.is_some());
        if in_server {
            let is_valid = draft_message
                .with(|msg| gilvave_core::validation::validate_message(msg).is_ok());
            if !is_valid {
                return;
            }
            send_channel_message(channel_context, draft_message);
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

    let on_keydown = move |e: web_sys::KeyboardEvent| {
        if e.key() == "Escape" {
            e.prevent_default();
            modal_context.is_message_editor_open.set(false);
        } else if e.key() == "Enter" && (e.ctrl_key() || e.meta_key()) {
            e.prevent_default();
            do_send();
        }
    };

    view! {
        div(class="message-editor-overlay", on:click=on_overlay_click) {
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
                    textarea(
                        class="expanded-editor-textarea",
                        placeholder="Напишите сообщение...",
                        bind:value=draft_message,
                        on:keydown=on_keydown,
                    )
                    button(
                        class="editor-collapse-btn",
                        r#type="button",
                        title="Свернуть окно и вернуться к обычному полю",
                        on:click=on_collapse,
                    ) {
                        img(src="/public/arrow2.svg", alt="Свернуть")
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
        }
    }
}

