use futures_util::StreamExt;
use sycamore::{futures::spawn_local_scoped, prelude::*, web::queue_microtask};
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlElement, SubmitEvent};

use super::message_item::MessageItem;
use crate::{
    cache::CacheService,
    components::{
        common::{
            ChannelContext, ScreenWrapper, ServerContext, UiModalContext, UserProfileContext,
            emoji_to_twemoji_url, focus_editor_at_end, insert_emoji_into_editor,
            insert_text_into_editor, serialize_contenteditable, sync_editor_content,
        },
        ui::icons::CloseSmallIcon,
    },
    gateway::{
        handler::{GatewayEvent, subscribe_gateway_events},
        service::WsService,
    },
    utils::{get_local_offset, to_local_datetime},
};
use gilvave_core::{
    dto::messages::MessageView,
    ids::{ChannelId, MessageId},
};

const EMOJI_GROUPS: &[(&str, &str, &str, emojis::Group)] = &[
    (
        "smileys",
        "😀",
        "Смайлики и эмоции",
        emojis::Group::SmileysAndEmotion,
    ),
    ("people", "👋", "Люди и жесты", emojis::Group::PeopleAndBody),
    (
        "animals",
        "🐱",
        "Животные и природа",
        emojis::Group::AnimalsAndNature,
    ),
    ("food", "🍔", "Еда и напитки", emojis::Group::FoodAndDrink),
    (
        "travel",
        "✈️",
        "Путешествия и места",
        emojis::Group::TravelAndPlaces,
    ),
    (
        "activities",
        "⚽",
        "Активности и спорт",
        emojis::Group::Activities,
    ),
    ("objects", "💡", "Предметы", emojis::Group::Objects),
    ("symbols", "❤️", "Символы", emojis::Group::Symbols),
    ("flags", "🚩", "Флаги", emojis::Group::Flags),
];

fn map_ru_query_to_en(ru: &str) -> &'static [&'static str] {
    let lower = ru.trim().to_lowercase();
    match lower.as_str() {
        s if s.starts_with("улыб")
            || s.starts_with("смех")
            || s.starts_with("радос")
            || s == "хаха"
            || s == "весел" =>
        {
            &["smile", "grin", "laugh", "happy", "joy"]
        }
        s if s.starts_with("смущ") || s.starts_with("мил") => &["blush", "shy"],
        s if s.starts_with("люб") || s.starts_with("серд") => &["heart", "love"],
        s if s.starts_with("поцел") => &["kiss"],
        s if s.starts_with("глаз") => &["eye", "eyes"],
        s if s.starts_with("слез") || s.starts_with("плач") || s.starts_with("рыд") => {
            &["cry", "sob", "tear"]
        }
        s if s.starts_with("груст") || s.starts_with("печал") || s.starts_with("тоск") => {
            &["sad", "pensive", "disappointed"]
        }
        s if s.starts_with("зло") || s.starts_with("гнев") || s.starts_with("ярос") => {
            &["angry", "rage", "mad"]
        }
        s if s.starts_with("шок") || s.starts_with("ужас") || s.starts_with("страх") => {
            &["scream", "fear", "shock"]
        }
        s if s.starts_with("сон") || s.starts_with("спат") || s.starts_with("устал") => {
            &["sleep", "yawn", "tired", "zzz"]
        }
        s if s.starts_with("дум") || s.starts_with("мысл") => &["thinking", "thought"],
        s if s.starts_with("крут") || s.starts_with("очки") => &["cool", "sunglasses"],
        s if s.starts_with("праздн") || s.starts_with("вечерин") || s.starts_with("ура") => {
            &["party", "tada", "celebration"]
        }
        s if s.starts_with("ботан") || s.starts_with("умн") => &["nerd"],
        s if s.starts_with("клоун") => &["clown"],
        s if s.starts_with("призрак") || s.starts_with("привид") => &["ghost"],
        s if s.starts_with("череп") || s.starts_with("смерт") => &["skull"],
        s if s.starts_with("огонь")
            || s.starts_with("плам")
            || s.starts_with("жар")
            || s == "топ" =>
        {
            &["fire", "flame"]
        }
        s if s.starts_with("звезд") || s.starts_with("блеск") || s.starts_with("сиян") => {
            &["star", "sparkle"]
        }
        s if s.starts_with("молни") || s.starts_with("гром") || s.starts_with("ток") => {
            &["zap", "lightning"]
        }
        s if s.starts_with("взрыв") || s.starts_with("бум") => {
            &["boom", "explosion", "collision"]
        }

        s if s.starts_with("рук")
            || s.starts_with("ладон")
            || s.starts_with("привет")
            || s.starts_with("пок") =>
        {
            &["hand", "wave", "raised"]
        }
        s if s.starts_with("лайк") || s.starts_with("класс") || s.starts_with("супер") => {
            &["thumbs up"]
        }
        s if s.starts_with("дизлайк") || s.starts_with("плох") => &["thumbs down"],
        s if s.starts_with("палец") || s.starts_with("пальц") => &["finger", "point"],
        s if s.starts_with("кулак") => &["fist"],
        s if s.starts_with("хлоп") || s.starts_with("аплодис") => &["clap"],
        s if s.starts_with("молитв") || s.starts_with("спасиб") || s.starts_with("пожалуйст") => {
            &["pray"]
        }
        s if s.starts_with("сил") || s.starts_with("бицепс") || s.starts_with("мышц") => {
            &["muscle"]
        }
        s if s.starts_with("ок") => &["ok"],
        s if s.starts_with("мир") || s.starts_with("побед") => &["peace", "victory"],

        s if s.starts_with("кот") || s.starts_with("кош") => &["cat"],
        s if s.starts_with("собак") || s.starts_with("пес") || s.starts_with("щен") => {
            &["dog"]
        }
        s if s.starts_with("лис") => &["fox"],
        s if s.starts_with("медвед") || s.starts_with("мишк") => &["bear"],
        s if s.starts_with("панд") => &["panda"],
        s if s.starts_with("лев") => &["lion"],
        s if s.starts_with("тигр") => &["tiger"],
        s if s.starts_with("обезьян") => &["monkey"],
        s if s.starts_with("лягушк") || s.starts_with("жаб") => &["frog"],
        s if s.starts_with("птиц") => &["bird"],
        s if s.starts_with("пингвин") => &["penguin"],
        s if s.starts_with("сов") => &["owl"],
        s if s.starts_with("бабочк") => &["butterfly"],
        s if s.starts_with("рыб") => &["fish"],
        s if s.starts_with("кит") => &["whale"],
        s if s.starts_with("краб") => &["crab"],
        s if s.starts_with("осьминог") => &["octopus"],

        s if s.starts_with("ед") || s.starts_with("кушат") => &["food"],
        s if s.starts_with("пицц") => &["pizza"],
        s if s.starts_with("бургер") => &["burger", "hamburger"],
        s if s.starts_with("картошк") || s.starts_with("фри") => &["fries"],
        s if s.starts_with("хотдог") => &["hotdog"],
        s if s.starts_with("торт") || s.starts_with("пирож") => &["cake"],
        s if s.starts_with("пончик") => &["donut"],
        s if s.starts_with("шоколад") => &["chocolate"],
        s if s.starts_with("печень") => &["cookie"],
        s if s.starts_with("кофе") => &["coffee"],
        s if s.starts_with("чай") => &["tea"],
        s if s.starts_with("пив") => &["beer"],
        s if s.starts_with("вин") || s.starts_with("бокал") => &["wine", "cheers"],
        s if s.starts_with("яблок") => &["apple"],
        s if s.starts_with("банан") => &["banana"],
        s if s.starts_with("клубник") => &["strawberry"],
        s if s.starts_with("арбуз") => &["watermelon"],

        s if s.starts_with("машин") || s.starts_with("авто") => &["car", "automobile"],
        s if s.starts_with("самолет") => &["airplane"],
        s if s.starts_with("поезд") => &["train"],
        s if s.starts_with("ракет") => &["rocket"],
        s if s.starts_with("дом") => &["house", "building"],
        s if s.starts_with("комп") || s.starts_with("ноут") => &["laptop", "computer"],
        s if s.starts_with("телефон") => &["phone", "mobile"],
        s if s.starts_with("наушник") => &["headphone"],
        s if s.starts_with("музык") || s.starts_with("нот") => &["music", "note"],
        s if s.starts_with("игр") || s.starts_with("гейм") => &["game", "video game"],
        s if s.starts_with("деньг") || s.starts_with("рубл") || s.starts_with("доллар") => {
            &["money", "dollar"]
        }
        s if s.starts_with("подар") => &["gift", "present"],
        s if s.starts_with("замок") => &["lock"],
        s if s.starts_with("ключ") => &["key"],
        s if s.starts_with("галочк") => &["check"],
        s if s.starts_with("крестик") => &["cross"],
        s if s.starts_with("флаг") => &["flag"],
        s if s.starts_with("росси") => &["russia"],
        _ => &[],
    }
}

fn matches_emoji_query(emoji: &emojis::Emoji, query: &str) -> bool {
    if emoji.as_str().contains(query) {
        return true;
    }
    let name_lower = emoji.name().to_lowercase();
    if name_lower.contains(query) {
        return true;
    }
    let sc_lower = emoji.shortcode().map(|sc| sc.to_lowercase());
    if let Some(ref sc) = sc_lower
        && sc.contains(query)
    {
        return true;
    }
    let en_keywords = map_ru_query_to_en(query);
    for kw in en_keywords {
        if name_lower.contains(kw) {
            return true;
        }
        if let Some(ref sc) = sc_lower
            && sc.contains(kw)
        {
            return true;
        }
    }
    false
}

#[derive(Clone, PartialEq)]
struct EnrichedMessage {
    message: MessageView,
    avatar_url: String,
    is_first: bool,
    is_last: bool,
    date_divider: Option<String>,
    is_unread_divider: bool,
}

#[inline]
fn hash_str(s: &str) -> u64 {
    let mut hasher = std::hash::DefaultHasher::new();
    std::hash::Hash::hash(s, &mut hasher);
    std::hash::Hasher::finish(&hasher)
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

fn get_channel_last_read(channel_id: ChannelId) -> Option<(Option<MessageId>, Option<i64>)> {
    #[cfg(target_arch = "wasm32")]
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let id_key = format!("gilvave_last_read_id_{channel_id}");
            let ts_key = format!("gilvave_last_read_ts_{channel_id}");
            let msg_id = storage
                .get_item(&id_key)
                .ok()
                .flatten()
                .and_then(|s| MessageId::try_from(s.as_str()).ok());
            let ts = storage
                .get_item(&ts_key)
                .ok()
                .flatten()
                .and_then(|s| s.parse::<i64>().ok());
            if msg_id.is_some() || ts.is_some() {
                return Some((msg_id, ts));
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = channel_id;
    None
}

pub fn save_channel_last_read(channel_id: ChannelId, message_id: MessageId, ts: i64) {
    #[cfg(target_arch = "wasm32")]
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let id_key = format!("gilvave_last_read_id_{channel_id}");
            let ts_key = format!("gilvave_last_read_ts_{channel_id}");
            let _ = storage.set_item(&id_key, &message_id.to_string());
            let _ = storage.set_item(&ts_key, &ts.to_string());
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (channel_id, message_id, ts);
    }
}

fn determine_unread_marker_fn(
    saved: Option<(Option<MessageId>, Option<i64>)>,
    msgs: &[MessageView],
) -> Option<MessageId> {
    let (saved_id, saved_ts) = saved?;
    if msgs.is_empty() {
        return None;
    }
    // 1. Try finding message by ID
    if let Some(s_id) = saved_id
        && let Some(idx) = msgs.iter().position(|m| m.id == s_id)
    {
        if idx + 1 < msgs.len() {
            return Some(msgs[idx + 1].id);
        } else {
            return None; // Caught up to the end!
        }
    }
    // 2. Try by timestamp if message ID was older or not found
    if let Some(ts) = saved_ts {
        let first_after = msgs.iter().find(|m| m.created_at.unix_timestamp() > ts)?;
        return Some(first_after.id);
    }
    None
}

pub fn scroll_to_initial_chat_position(
    container: &web_sys::HtmlElement,
    unread_id: Option<MessageId>,
) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(target_id) = unread_id {
            let selector = format!("[data-unread-marker='{target_id}']");
            if let Ok(Some(marker_el)) = container.query_selector(&selector) {
                if let Ok(html_marker) = marker_el.dyn_into::<web_sys::HtmlElement>() {
                    let marker_top = html_marker.offset_top();
                    container.set_scroll_top((marker_top - 60).max(0));
                    return;
                }
            }
            // Retry on next animation frame if Sycamore DOM node is mounting
            let container_clone = container.clone();
            if let Some(window) = web_sys::window() {
                let cb = wasm_bindgen::closure::Closure::once_into_js(move || {
                    if let Ok(Some(marker_el)) = container_clone.query_selector(&selector) {
                        if let Ok(html_marker) = marker_el.dyn_into::<web_sys::HtmlElement>() {
                            let marker_top = html_marker.offset_top();
                            container_clone.set_scroll_top((marker_top - 60).max(0));
                        }
                    }
                });
                let _ = window.request_animation_frame(cb.unchecked_ref());
            }
            return;
        }
        container.set_scroll_top(container.scroll_height());
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = (container, unread_id);
}

fn send_channel_message(
    channel_context: ChannelContext,
    modal_context: UiModalContext,
    draft_message: Signal<String>,
    _user_profile: UserProfileContext,
    container: Option<NodeRef>,
) {
    let Some(channel_id) = channel_context.current.with(|c| c.as_ref().map(|ch| ch.id)) else {
        return;
    };

    let sanitized = match draft_message.with(|msg| gilvave_core::validation::validate_message(msg))
    {
        Ok(s) => s,
        Err(e) => {
            web_sys::console::warn_1(&format!("[MESSAGES_AREA] on_submit ignored: {e}").into());
            return;
        }
    };

    let editing = modal_context.editing_message.get_clone();
    let replying = modal_context.replying_to.get_clone();

    if let Some(edit_msg) = editing {
        spawn_local_scoped(async move {
            web_sys::console::log_1(
                &format!("[MESSAGES_AREA] invoking MessageUpdate for {}", edit_msg.id).into(),
            );
            let res = WsService::message_update(edit_msg.channel_id, edit_msg.id, sanitized).await;
            web_sys::console::log_1(&format!("[MESSAGES_AREA] MessageUpdate res: {res:?}").into());
        });
        modal_context.editing_message.set(None);
    } else {
        let reply_to = replying.as_ref().map(|r| r.id);
        modal_context.replying_to.set(None);

        save_channel_last_read(
            channel_id,
            MessageId::default(),
            time::OffsetDateTime::now_utc().unix_timestamp(),
        );

        spawn_local_scoped(async move {
            web_sys::console::log_1(
                &format!(
                    "[MESSAGES_AREA] invoking MessageCreate for {channel_id} with reply_to={reply_to:?}"
                )
                .into(),
            );
            let res = WsService::message_create(channel_id, sanitized, reply_to, None).await;
            web_sys::console::log_1(&format!("[MESSAGES_AREA] MessageCreate res: {res:?}").into());
        });

        if let Some(node) = container.as_ref().and_then(|c| c.try_get()) {
            let el = node.unchecked_into::<HtmlElement>();
            el.set_scroll_top(el.scroll_height());
        }
    }

    draft_message.set(String::new());
}

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

    // Global Escape key listener when popover is open
    create_effect(move || {
        if is_emoji_modal_open.get() || is_attach_menu_open.get() {
            #[cfg(target_arch = "wasm32")]
            if let Some(window) = web_sys::window() {
                let on_key = wasm_bindgen::closure::Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(
                    move |e: web_sys::KeyboardEvent| {
                        if e.key() == "Escape" {
                            is_attach_menu_open.set(false);
                            is_emoji_modal_open.set(false);
                            is_emoji_expanded.set(false);
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
        use wasm_bindgen::JsCast;
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

    view! {
        form(class="input-area", on:submit=on_submit) {
            (move || {
                if let Some(reply_msg) = modal_context.replying_to.get_clone() {
                    let author = reply_msg.author_name;
                    let snippet = if reply_msg.content.chars().count() > 60 {
                        let s: String = reply_msg.content.chars().take(60).collect();
                        format!("{s}…")
                    } else {
                        reply_msg.content
                    };
                    view! {
                        div(class="chat-action-banner chat-reply-banner") {
                            div(class="banner-spine")
                            div(class="banner-info") {
                                span(class="banner-title") { (format!("Ответ пользователю @{author}")) }
                                span(class="banner-snippet") { (snippet) }
                            }
                            button(
                                r#type="button",
                                class="banner-close-btn",
                                title="Отменить ответ (Esc)",
                                on:click=move |_| modal_context.replying_to.set(None),
                            ) {
                                CloseSmallIcon()
                            }
                        }
                    }
                } else if let Some(edit_msg) = modal_context.editing_message.get_clone() {
                    let snippet = if edit_msg.content.chars().count() > 60 {
                        let s: String = edit_msg.content.chars().take(60).collect();
                        format!("{s}…")
                    } else {
                        edit_msg.content
                    };
                    view! {
                        div(class="chat-action-banner chat-edit-banner") {
                            div(class="banner-spine")
                            div(class="banner-info") {
                                span(class="banner-title") { "Редактирование сообщения" }
                                span(class="banner-snippet") { (snippet) }
                            }
                            span(class="banner-esc-hint") { "Esc — отмена" }
                            button(
                                r#type="button",
                                class="banner-close-btn",
                                title="Отменить редактирование (Esc)",
                                on:click=move |_| {
                                    modal_context.editing_message.set(None);
                                    modal_context.draft_message.set(String::new());
                                },
                            ) {
                                CloseSmallIcon()
                            }
                        }
                    }
                } else {
                    view! {}
                }
            })
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
                            class=move || {
                                if is_emoji_expanded.get() {
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
                                        title=move || if is_emoji_expanded.get() {
                                            "Вернуть обычный размер"
                                        } else {
                                            "Развернуть окно эмодзи"
                                        },
                                        on:click=move |e: web_sys::MouseEvent| {
                                            e.stop_propagation();
                                            let next = !is_emoji_expanded.get();
                                            is_emoji_expanded.set(next);
                                        },
                                    ) {
                                        (move || if is_emoji_expanded.get() {
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
                                            is_emoji_modal_open.set(false);
                                            is_emoji_expanded.set(false);
                                        },
                                    ) {
                                        CloseSmallIcon()
                                    }
                                }
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
                                (EMOJI_GROUPS
                                    .iter()
                                    .map(|(cat_id, cat_icon, cat_title, _)| {
                                        let id_str = (*cat_id).to_string();
                                        let id_check = id_str.clone();
                                        let icon_str = (*cat_icon).to_string();
                                        let icon_url = emoji_to_twemoji_url(cat_icon);
                                        let title_str = (*cat_title).to_string();
                                        view! {
                                            button(
                                                r#type="button",
                                                class=move || if emoji_category.get_clone() == id_check {
                                                    "emoji-cat-btn active"
                                                } else {
                                                    "emoji-cat-btn"
                                                },
                                                title=title_str,
                                                on:click=move |_| emoji_category.set(id_str.clone()),
                                            ) {
                                                img(
                                                    src=icon_url,
                                                    alt=icon_str,
                                                    draggable="false",
                                                )
                                            }
                                        }
                                    })
                                    .collect::<Vec<View>>())
                            }
                            div(class="emoji-modal-body") {
                                (move || {
                                    let query = emoji_search.get_clone().trim().to_lowercase();
                                    let active_cat = emoji_category.get_clone();

                                    let list: Vec<&'static emojis::Emoji> = if !query.is_empty() {
                                        emojis::iter()
                                            .filter(|e| matches_emoji_query(e, &query))
                                            .take(150)
                                            .collect()
                                    } else if let Some((_, _, _, grp)) = EMOJI_GROUPS.iter().find(|(id, _, _, _)| *id == active_cat) {
                                        grp.emojis().collect()
                                    } else {
                                        emojis::Group::SmileysAndEmotion.emojis().collect()
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
                                                let emoji_url = emoji_to_twemoji_url(&emoji_char);
                                                let emoji_url_click = emoji_url.clone();
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
                                                                    insert_emoji_into_editor(&editor, &to_add, &url, &title);
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
                                                                    draft_message.update(|msg| msg.push_str(&to_add));
                                                                }
                                                            } else {
                                                                draft_message.update(|msg| msg.push_str(&to_add));
                                                            }
                                                        },
                                                    ) {
                                                        img(
                                                            src=emoji_url,
                                                            alt=emoji_char,
                                                            loading="lazy",
                                                            draggable="false",
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

#[component()]
pub fn MessagesArea() -> View {
    let channel_context = use_context::<ChannelContext>();
    let server_context = use_context::<ServerContext>();
    let user_profile = use_context::<UserProfileContext>();
    let modal_context = use_context::<UiModalContext>();
    let channel_name = create_memo(move || {
        channel_context.current.with(|c| {
            c.as_ref()
                .map_or_else(|| "<UNKNOWN>".into(), |ch| ch.name.clone())
        })
    });

    let draft_message = modal_context.draft_message;
    let container = create_node_ref();

    let unread_marker_id = create_signal::<Option<MessageId>>(None);
    let initial_scroll_performed = create_signal::<bool>(false);
    let saved_read_marker = create_signal::<Option<(Option<MessageId>, Option<i64>)>>(None);
    let prev_channel_id = create_signal::<Option<ChannelId>>(None);

    let on_scroll = move |event: Event| {
        let el: HtmlElement = match event.current_target() {
            Some(t) => t.unchecked_into(),
            None => return,
        };

        const SCROLL_THRESHOLD: i32 = 40;

        let top = el.scroll_top();
        let sh = el.scroll_height();
        let ch = el.client_height();

        let is_top = top <= 0;
        let is_bottom = (sh - ch - top).abs() <= SCROLL_THRESHOLD;

        if is_bottom
            && let Some(ch_id) = channel_context.current.with(|c| c.as_ref().map(|ch| ch.id))
            && let Some(last_msg) = channel_context.messages.with(|m| m.last().cloned())
        {
            save_channel_last_read(
                ch_id,
                last_msg.id,
                last_msg.created_at.unix_timestamp(),
            );
        }

        if is_top {
            console_log!("top message");
            spawn_local_scoped(async move {
                let Some(ch_id) = channel_context.current.with(|c| c.as_ref().map(|ch| ch.id))
                else {
                    return;
                };
                let Some(first_dt) = channel_context
                    .messages
                    .with(|m| m.first().map(|msg| msg.created_at))
                else {
                    return;
                };
                let _ = WsService::channel_history_before(ch_id, first_dt).await;
            });
        }
        if is_bottom {
            console_log!("bottom message");
            spawn_local_scoped(async move {
                let Some(ch_id) = channel_context.current.with(|c| c.as_ref().map(|ch| ch.id))
                else {
                    return;
                };
                let Some(last_dt) = channel_context
                    .messages
                    .with(|m| m.last().map(|msg| msg.created_at))
                else {
                    return;
                };
                let _ = WsService::channel_history_after(ch_id, last_dt).await;
            });
        }
    };

    create_effect(move || {
        let cur = channel_context.current.get_clone();

        // 1. If we were viewing a previous channel at bottom or fit on screen, record last read
        if let Some(prev_id) = prev_channel_id.get_untracked()
            && cur.as_ref().map(|c| c.id) != Some(prev_id)
            && let Some(node) = container.try_get()
        {
            let el: HtmlElement = node.unchecked_into();
            let is_bottom = (el.scroll_height() - el.client_height() - el.scroll_top()).abs() <= 80;
            let fits_screen = el.scroll_height() <= el.client_height() + 20;
            if (is_bottom || fits_screen)
                && unread_marker_id.get_untracked().is_none()
                && let Some(last_msg) = channel_context.messages.with_untracked(|m| m.last().cloned())
            {
                save_channel_last_read(prev_id, last_msg.id, last_msg.created_at.unix_timestamp());
            }
        }
        prev_channel_id.set(cur.as_ref().map(|c| c.id));

        channel_context.messages.set(vec![]);
        initial_scroll_performed.set(false);
        unread_marker_id.set(None);
        let saved = cur.as_ref().and_then(|ch| get_channel_last_read(ch.id));
        saved_read_marker.set(saved);

        if let Some(ch) = cur {
            spawn_local_scoped(async move {
                let cached = CacheService::get_cached_messages(ch.id, 100, None).await;
                if !cached.is_empty() {
                    channel_context.messages.update(|list| {
                        if list.is_empty() {
                            *list = cached.clone();
                        } else {
                            for msg in &cached {
                                if !list.iter().any(|m| m.id == msg.id) {
                                    list.push(msg.clone());
                                }
                            }
                            list.sort_by_key(|m| m.created_at);
                        }
                    });

                    if !initial_scroll_performed.get() {
                        let (marker, effective_saved) = if saved.is_none() {
                            // Baseline for new channel with no prior watermark
                            if let Some(last_msg) = cached.last() {
                                save_channel_last_read(ch.id, last_msg.id, last_msg.created_at.unix_timestamp());
                            }
                            (None, cached.last().map(|m| (Some(m.id), Some(m.created_at.unix_timestamp()))))
                        } else {
                            (determine_unread_marker_fn(saved, &cached), saved)
                        };
                        saved_read_marker.set(effective_saved);
                        unread_marker_id.set(marker);

                        if let Some(node) = container.try_get() {
                            let el: HtmlElement = node.unchecked_into();
                            scroll_to_initial_chat_position(&el, marker);
                            initial_scroll_performed.set(true);
                        }
                    }
                }
            });
        }
    });

    spawn_local_scoped(async move {
        web_sys::console::log_1(&"[MESSAGES_AREA] subscribing to gateway events...".into());
        let mut events = subscribe_gateway_events();
        while let Some(event) = events.next().await {
            match event {
                GatewayEvent::MessageNew(msg) => {
                    web_sys::console::log_1(
                        &format!("[MESSAGES_AREA] received MessageNew: {:?}", msg).into(),
                    );
                    let cur_channel_id =
                        channel_context.current.with(|c| c.as_ref().map(|ch| ch.id));
                    if cur_channel_id != Some(msg.channel_id) {
                        continue;
                    }

                    let Some(node) = container.try_get() else {
                        continue;
                    };
                    let el = node.unchecked_into::<HtmlElement>();
                    let old_scroll_top = el.scroll_top();
                    let old_scroll_height = el.scroll_height();
                    let old_client_height = el.client_height();

                    const SCROLL_THRESHOLD: i32 = 80;
                    let is_bottom = (old_scroll_height - old_client_height - old_scroll_top).abs()
                        <= SCROLL_THRESHOLD;
                    let is_own = user_profile
                        .username
                        .with(|u| !u.is_empty() && u == &msg.author_name)
                        || (msg.author_id.is_some() && msg.author_id == user_profile.user_id.get());

                    const MAX_LOADED_MESSAGES: usize = 300;
                    let msg_id = msg.id;
                    let msg_ch_id = msg.channel_id;
                    let msg_ts = msg.created_at.unix_timestamp();
                    let msg_to_cache = msg.clone();
                    spawn_local_scoped(async move {
                        CacheService::save_cached_message(msg_to_cache).await;
                    });

                    channel_context.messages.update(|list| {
                        // Prevent duplicate if already received
                        if list.iter().any(|m| m.id == msg_id) {
                            return;
                        }

                        // Remove any stray pending messages
                        list.retain(|m| !m.is_pending);
                        list.push(msg);
                        if list.len() > MAX_LOADED_MESSAGES {
                            list.remove(0);
                        }
                        list.sort_by_key(|m| m.created_at);
                    });

                    queue_microtask(move || {
                        if is_own || (is_bottom && unread_marker_id.get().is_none()) {
                            el.set_scroll_top(el.scroll_height());
                            if let Some(window) = web_sys::window() {
                                let el_clone = el.clone();
                                let cb = wasm_bindgen::closure::Closure::once_into_js(move || {
                                    el_clone.set_scroll_top(el_clone.scroll_height());
                                });
                                let _ = window.request_animation_frame(cb.unchecked_ref());
                            }
                            save_channel_last_read(msg_ch_id, msg_id, msg_ts);
                        } else {
                            if unread_marker_id.get().is_none() {
                                unread_marker_id.set(Some(msg_id));
                            }
                            let delta = el.scroll_height() - old_scroll_height;
                            if delta > 0 {
                                el.set_scroll_top(old_scroll_top + delta);
                            }
                        }
                    });
                }
                GatewayEvent::ChannelHistoryBefore(messages) => {
                    web_sys::console::log_1(
                        &format!(
                            "[MESSAGES_AREA] received ChannelHistoryBefore with {} items",
                            messages.len()
                        )
                        .into(),
                    );
                    let Some(node) = container.try_get() else {
                        continue;
                    };
                    let el = node.unchecked_into::<HtmlElement>();
                    let old_scroll_top = el.scroll_top();
                    let old_scroll_height = el.scroll_height();
                    let old_client_height = el.client_height();
                    let was_empty = channel_context.messages.with(|m| m.is_empty());
                    let was_at_bottom =
                        (old_scroll_height - old_client_height - old_scroll_top).abs() <= 80;
                    let has_initial_scroll = initial_scroll_performed.get();

                    let cur_ch_id = channel_context.current.with(|c| c.as_ref().map(|ch| ch.id));
                    if let Some(ch_id) = cur_ch_id {
                        let msgs_clone = messages.clone();
                        spawn_local_scoped(async move {
                            CacheService::sync_cached_messages(ch_id, msgs_clone).await;
                        });
                    }

                    const MAX_LOADED_MESSAGES: usize = 300;
                    channel_context.messages.update(|list| {
                        list.retain(|m| !m.is_pending);

                        if !messages.is_empty() {
                            let min_time = messages
                                .iter()
                                .map(|m| m.created_at)
                                .min()
                                .expect("non-empty");
                            let max_time = messages
                                .iter()
                                .map(|m| m.created_at)
                                .max()
                                .expect("non-empty");
                            let server_set: std::collections::HashSet<_> =
                                messages.iter().map(|m| m.id).collect();

                            // Reconcile: delete messages absent from server in this time window
                            list.retain(|m| {
                                if m.created_at >= min_time && m.created_at <= max_time {
                                    server_set.contains(&m.id)
                                } else {
                                    true
                                }
                            });

                            // Reconcile: update edited messages and insert new ones
                            for msg in messages {
                                if let Some(existing) = list.iter_mut().find(|m| m.id == msg.id) {
                                    existing.content = msg.content;
                                    existing.edited = msg.edited;
                                    existing.forwarded_from = msg.forwarded_from;
                                    existing.reply_to_id = msg.reply_to_id;
                                } else {
                                    list.push(msg);
                                }
                            }
                        }

                        list.sort_by_key(|m| m.created_at);
                        if list.len() > MAX_LOADED_MESSAGES {
                            let excess = list.len() - MAX_LOADED_MESSAGES;
                            list.drain(0..excess);
                        }
                    });

                    let all_msgs = channel_context.messages.get_clone();
                    let current_marker = unread_marker_id.get();

                    let marker = if current_marker.is_some() {
                        current_marker
                    } else {
                        let m = determine_unread_marker_fn(saved_read_marker.get(), &all_msgs);
                        if m.is_some() {
                            unread_marker_id.set(m);
                        } else if saved_read_marker.get().is_none()
                            && let Some(last_msg) = all_msgs.last()
                            && let Some(ch_id) = cur_ch_id
                        {
                            save_channel_last_read(ch_id, last_msg.id, last_msg.created_at.unix_timestamp());
                            saved_read_marker.set(Some((Some(last_msg.id), Some(last_msg.created_at.unix_timestamp()))));
                        }
                        m
                    };

                    if !has_initial_scroll {
                        let el_clone = el.clone();
                        scroll_to_initial_chat_position(&el_clone, marker);
                        initial_scroll_performed.set(true);
                    } else if marker.is_some() && current_marker.is_none() {
                        let el_clone = el.clone();
                        scroll_to_initial_chat_position(&el_clone, marker);
                    } else {
                        queue_microtask(move || {
                            if was_empty || (was_at_bottom && marker.is_none()) {
                                el.set_scroll_top(el.scroll_height());
                                if let Some(window) = web_sys::window() {
                                    let el_clone = el.clone();
                                    let cb = wasm_bindgen::closure::Closure::once_into_js(move || {
                                        el_clone.set_scroll_top(el_clone.scroll_height());
                                    });
                                    let _ = window.request_animation_frame(cb.unchecked_ref());
                                }
                            } else {
                                let delta = el.scroll_height() - old_scroll_height;
                                if delta > 0 {
                                    el.set_scroll_top(old_scroll_top + delta);
                                }
                            }
                        });
                    }
                }
                GatewayEvent::ChannelHistoryAfter(messages) => {
                    web_sys::console::log_1(
                        &format!(
                            "[MESSAGES_AREA] received ChannelHistoryAfter with {} items",
                            messages.len()
                        )
                        .into(),
                    );
                    let cur_ch_id = channel_context.current.with(|c| c.as_ref().map(|ch| ch.id));
                    if let Some(ch_id) = cur_ch_id {
                        let msgs_clone = messages.clone();
                        spawn_local_scoped(async move {
                            CacheService::sync_cached_messages(ch_id, msgs_clone).await;
                        });
                    }

                    const MAX_LOADED_MESSAGES: usize = 300;
                    channel_context.messages.update(|list| {
                        list.retain(|m| !m.is_pending);

                        if !messages.is_empty() {
                            let min_time = messages
                                .iter()
                                .map(|m| m.created_at)
                                .min()
                                .expect("non-empty");
                            let max_time = messages
                                .iter()
                                .map(|m| m.created_at)
                                .max()
                                .expect("non-empty");
                            let server_set: std::collections::HashSet<_> =
                                messages.iter().map(|m| m.id).collect();

                            list.retain(|m| {
                                if m.created_at >= min_time && m.created_at <= max_time {
                                    server_set.contains(&m.id)
                                } else {
                                    true
                                }
                            });

                            for msg in messages {
                                if let Some(existing) = list.iter_mut().find(|m| m.id == msg.id) {
                                    existing.content = msg.content;
                                    existing.edited = msg.edited;
                                    existing.forwarded_from = msg.forwarded_from;
                                    existing.reply_to_id = msg.reply_to_id;
                                } else {
                                    list.push(msg);
                                }
                            }
                        }

                        list.sort_by_key(|m| m.created_at);
                        if list.len() > MAX_LOADED_MESSAGES {
                            let excess = list.len() - MAX_LOADED_MESSAGES;
                            list.drain(0..excess);
                        }
                    });
                }
                GatewayEvent::MessageUpdate(updated) => {
                    web_sys::console::log_1(
                        &format!("[MESSAGES_AREA] received MessageUpdate: {:?}", updated).into(),
                    );
                    let cur_channel_id =
                        channel_context.current.with(|c| c.as_ref().map(|ch| ch.id));
                    if cur_channel_id != Some(updated.channel_id) {
                        continue;
                    }

                    let upd_id = updated.id;
                    let upd_content = updated.content.clone();
                    let upd_edited = updated.edited;
                    spawn_local_scoped(async move {
                        CacheService::update_cached_message(upd_id, upd_content, upd_edited).await;
                    });

                    channel_context.messages.update(|list| {
                        if let Some(target) = list.iter_mut().find(|m| m.id == updated.id) {
                            target.content = updated.content;
                            target.edited = true;
                        }
                    });
                }
                GatewayEvent::MessageDelete {
                    channel_id,
                    message_id,
                } => {
                    web_sys::console::log_1(
                        &format!("[MESSAGES_AREA] received MessageDelete: message_id={message_id}")
                            .into(),
                    );
                    let cur_channel_id =
                        channel_context.current.with(|c| c.as_ref().map(|ch| ch.id));
                    if cur_channel_id != Some(channel_id) {
                        continue;
                    }

                    let del_id = message_id;
                    spawn_local_scoped(async move {
                        CacheService::delete_cached_message(del_id).await;
                    });

                    channel_context.messages.update(|list| {
                        list.retain(|m| m.id != message_id);
                    });
                    if modal_context
                        .replying_to
                        .with(|r| r.as_ref().map(|m| m.id) == Some(message_id))
                    {
                        modal_context.replying_to.set(None);
                    }
                    if modal_context
                        .editing_message
                        .with(|e| e.as_ref().map(|m| m.id) == Some(message_id))
                    {
                        modal_context.editing_message.set(None);
                        modal_context.draft_message.set(String::new());
                    }
                }
                _ => {}
            }
        }
    });

    let on_back = move |_| {
        if let Some(ch) = channel_context.current.get_clone()
            && let Some(node) = container.try_get()
        {
            let el: HtmlElement = node.unchecked_into();
            let is_bottom = (el.scroll_height() - el.client_height() - el.scroll_top()).abs() <= 80;
            let fits_screen = el.scroll_height() <= el.client_height() + 20;
            if (is_bottom || fits_screen)
                && unread_marker_id.get_untracked().is_none()
                && let Some(last_msg) = channel_context.messages.with_untracked(|m| m.last().cloned())
            {
                save_channel_last_read(ch.id, last_msg.id, last_msg.created_at.unix_timestamp());
            }
        }
        channel_context.current.set(None);
    };

    let enriched_messages = create_memo(move || {
        let cur_unread_marker = unread_marker_id.get();
        channel_context.messages.with(|msgs| {
            server_context.members.with(|members| {
                user_profile.username.with(|my_username| {
                    user_profile.avatar.with(|my_avatar| {
                        let len = msgs.len();
                        let local_offset = get_local_offset();
                        let mut local_dts: Vec<time::OffsetDateTime> = Vec::with_capacity(len);
                        for m in msgs.iter() {
                            local_dts.push(m.created_at.to_offset(local_offset));
                        }

                        let mut avatar_by_username =
                            std::collections::HashMap::with_capacity(members.len());
                        let mut avatar_by_id =
                            std::collections::HashMap::with_capacity(members.len());
                        for mem in members.iter() {
                            if !mem.avatar.is_empty() {
                                avatar_by_username
                                    .insert(mem.username.as_str(), mem.avatar.as_str());
                                avatar_by_id.insert(mem.user_id, mem.avatar.as_str());
                            }
                        }

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

                                let avatar =
                                    if !my_avatar.is_empty() && m.author_name == *my_username {
                                        my_avatar.clone()
                                    } else if let Some(author_id) = m.author_id
                                        && let Some(&av) = avatar_by_id.get(&author_id)
                                    {
                                        av.to_string()
                                    } else if let Some(&av) =
                                        avatar_by_username.get(m.author_name.as_str())
                                    {
                                        av.to_string()
                                    } else if m.author_name == *my_username {
                                        my_avatar.clone()
                                    } else {
                                        String::new()
                                    };

                                let is_unread_divider = if let Some(marker_id) = cur_unread_marker {
                                    m.id == marker_id
                                } else {
                                    false
                                };

                                EnrichedMessage {
                                    message: m.clone(),
                                    avatar_url: avatar,
                                    is_first,
                                    is_last,
                                    date_divider,
                                    is_unread_divider,
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
                    key=|em| (
                        em.message.id,
                        hash_str(&em.message.content),
                        em.message.edited,
                        em.message.is_pending,
                        em.message.reply_to_id,
                        em.is_first,
                        em.is_last,
                        em.date_divider.as_deref().map(hash_str),
                        em.is_unread_divider,
                        hash_str(&em.avatar_url),
                    ),
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

                        let unread_view = if em.is_unread_divider {
                            view! {
                                div(
                                    class="chat-unread-divider",
                                    data-unread-marker=em.message.id.to_string(),
                                ) {
                                    span(class="unread-badge") { "Непрочитанные сообщения" }
                                }
                            }
                        } else {
                            view! {}
                        };

                        view! {
                            (date_view)
                            (unread_view)
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

            ChatInputArea(on_send=move || send_channel_message(channel_context, modal_context, draft_message, user_profile, Some(container)))
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
    let user_profile = use_context::<UserProfileContext>();

    let draft_message = modal_context.draft_message;

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
            let is_valid =
                draft_message.with(|msg| gilvave_core::validation::validate_message(msg).is_ok());
            if !is_valid {
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

    let expanded_editor_ref = create_node_ref();

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

    let on_expanded_paste = move |event: web_sys::Event| {
        use wasm_bindgen::JsCast;
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
                draft_message.set(serialized);
            }
        }
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
                    div(
                        r#ref=expanded_editor_ref,
                        class="expanded-editor-textarea",
                        contenteditable="true",
                        role="textbox",
                                                data-placeholder="Напишите сообщение...",
                        on:input=on_expanded_input,
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
