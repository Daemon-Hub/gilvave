//! Chat scroll management, date dividers, and unread watermark persistence.

use gilvave_core::{
    dto::messages::MessageView,
    ids::{ChannelId, MessageId},
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

use crate::utils::to_local_datetime;

pub fn format_date_divider(date: time::Date) -> String {
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

pub fn get_channel_last_read(channel_id: ChannelId) -> Option<(Option<MessageId>, Option<i64>)> {
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

pub fn determine_unread_marker_fn(
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

#[allow(dead_code)]
pub fn scroll_to_bottom(container: &web_sys::HtmlElement) {
    #[cfg(target_arch = "wasm32")]
    container.set_scroll_top(container.scroll_height());
    #[cfg(not(target_arch = "wasm32"))]
    let _ = container;
}
