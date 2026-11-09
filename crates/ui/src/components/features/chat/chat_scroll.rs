//! Chat scroll management, date dividers, viewport anchoring, and unread watermark persistence.

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

/// Получение последней сохраненной отметки прочитанного сообщения (ID сообщения и timestamp)
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

/// Сохранение отметки последнего прочитанного сообщения
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

/// Сохранение состояния скролла (находился ли пользователь в самом низу или читал историю)
pub fn save_channel_scroll_state(
    channel_id: ChannelId,
    is_bottom: bool,
    anchor_id: Option<MessageId>,
    offset: i32,
) {
    #[cfg(target_arch = "wasm32")]
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let at_bottom_key = format!("gilvave_scroll_at_bottom_{channel_id}");
            let anchor_id_key = format!("gilvave_scroll_anchor_id_{channel_id}");
            let anchor_offset_key = format!("gilvave_scroll_anchor_offset_{channel_id}");

            let _ = storage.set_item(&at_bottom_key, if is_bottom { "true" } else { "false" });
            if let Some(a_id) = anchor_id {
                let _ = storage.set_item(&anchor_id_key, &a_id.to_string());
                let _ = storage.set_item(&anchor_offset_key, &offset.to_string());
            } else if is_bottom {
                let _ = storage.remove_item(&anchor_id_key);
                let _ = storage.remove_item(&anchor_offset_key);
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (channel_id, is_bottom, anchor_id, offset);
    }
}

/// Получение состояния скролла канала: (находился_ли_внизу, id_якорного_сообщения, смещение_в_пикселях)
pub fn get_channel_scroll_state(channel_id: ChannelId) -> (bool, Option<MessageId>, i32) {
    #[cfg(target_arch = "wasm32")]
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let at_bottom_key = format!("gilvave_scroll_at_bottom_{channel_id}");
            let anchor_id_key = format!("gilvave_scroll_anchor_id_{channel_id}");
            let anchor_offset_key = format!("gilvave_scroll_anchor_offset_{channel_id}");

            let is_bottom = storage
                .get_item(&at_bottom_key)
                .ok()
                .flatten()
                .map(|s| s != "false")
                .unwrap_or(true);

            let anchor_id = storage
                .get_item(&anchor_id_key)
                .ok()
                .flatten()
                .and_then(|s| MessageId::try_from(s.as_str()).ok());

            let offset = storage
                .get_item(&anchor_offset_key)
                .ok()
                .flatten()
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(0);

            return (is_bottom, anchor_id, offset);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = channel_id;
    (true, None, 0)
}

/// Определение верхнего видимого сообщения в области видимости контейнера
#[cfg(target_arch = "wasm32")]
pub fn find_top_visible_message(container: &web_sys::HtmlElement) -> (Option<MessageId>, i32) {
    let c_rect = container.get_bounding_client_rect();
    let threshold = c_rect.top() + 10.0;

    let Ok(node_list) = container.query_selector_all(".message[data-message-id]") else {
        return (None, 0);
    };

    let len = node_list.length();
    for i in 0..len {
        let Some(node) = node_list.get(i) else { continue };
        let Ok(el) = node.dyn_into::<web_sys::HtmlElement>() else { continue };
        let el_rect = el.get_bounding_client_rect();

        if el_rect.bottom() > threshold {
            if let Some(id_str) = el.get_attribute("data-message-id") {
                if let Ok(m_id) = MessageId::try_from(id_str.as_str()) {
                    let offset = (c_rect.top() - el_rect.top()).max(0.0) as i32;
                    return (Some(m_id), offset);
                }
            }
        }
    }
    (None, 0)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn find_top_visible_message(_container: &web_sys::HtmlElement) -> (Option<MessageId>, i32) {
    (None, 0)
}

/// Вычисление первого непрочитанного сообщения на основе сохраненного водяного знака
pub fn determine_unread_marker_fn(
    saved: Option<(Option<MessageId>, Option<i64>)>,
    msgs: &[MessageView],
) -> Option<MessageId> {
    let (saved_id, saved_ts) = saved?;
    if msgs.is_empty() {
        return None;
    }
    // 1. Поиск по сохраненному ID сообщения
    if let Some(s_id) = saved_id
        && let Some(idx) = msgs.iter().position(|m| m.id == s_id)
    {
        if idx + 1 < msgs.len() {
            return Some(msgs[idx + 1].id);
        } else {
            return None; // Пользователь видел последнее сообщение
        }
    }
    // 2. Поиск по timestamp если точный ID старше или отсутствует в текущей выборке
    if let Some(ts) = saved_ts {
        let first_after = msgs.iter().find(|m| m.created_at.unix_timestamp() > ts)?;
        return Some(first_after.id);
    }
    None
}

/// Целевая позиция скролла при входе в канал
#[derive(Clone, Debug, PartialEq)]
pub enum ChannelScrollAnchor {
    /// Самый низ (последние сообщения)
    Bottom,
    /// Плашка «Непрочитанные сообщения»
    UnreadMarker(MessageId),
    /// Конкретное сообщение, на котором пользователь остановился при чтении истории
    Message {
        message_id: MessageId,
        offset: i32,
    },
}

/// Чистая логика определения целевой позиции скролла
pub fn determine_target_scroll_pure(
    scroll_state: (bool, Option<MessageId>, i32),
    saved_last_read: Option<(Option<MessageId>, Option<i64>)>,
    messages: &[MessageView],
) -> (ChannelScrollAnchor, Option<MessageId>) {
    let unread_marker = determine_unread_marker_fn(saved_last_read, messages);
    let (is_bottom, anchor_id, offset) = scroll_state;

    // Сценарий 3: Пользователь пролистал вверх для чтения истории и вышел
    // Возвращаем в то же самое место независимо от прихода новых сообщений внизу
    if !is_bottom
        && let Some(a_id) = anchor_id
        && messages.iter().any(|m| m.id == a_id)
    {
        return (
            ChannelScrollAnchor::Message {
                message_id: a_id,
                offset,
            },
            unread_marker,
        );
    }

    // Сценарий 2: Пользователь был в самом низу, но пришли новые сообщения
    // Позиционируем на плашку «Непрочитанные сообщения»
    if let Some(unread_id) = unread_marker {
        return (
            ChannelScrollAnchor::UnreadMarker(unread_id),
            Some(unread_id),
        );
    }

    // Сценарий 1: Пользователь видел последние сообщения (был внизу и новых нет)
    // Либо первый заход в канал без истории
    (ChannelScrollAnchor::Bottom, None)
}

/// Определение целевой позиции скролла для канала
pub fn determine_channel_target_scroll(
    channel_id: ChannelId,
    saved_last_read: Option<(Option<MessageId>, Option<i64>)>,
    messages: &[MessageView],
) -> (ChannelScrollAnchor, Option<MessageId>) {
    let state = get_channel_scroll_state(channel_id);
    determine_target_scroll_pure(state, saved_last_read, messages)
}

/// Непосредственное применение позиции скролла к HTML-контейнеру
#[cfg(target_arch = "wasm32")]
pub fn apply_channel_scroll_anchor(
    container: &web_sys::HtmlElement,
    anchor: &ChannelScrollAnchor,
) -> bool {
    let c_rect = container.get_bounding_client_rect();
    let current_scroll_top = container.scroll_top();

    match anchor {
        ChannelScrollAnchor::Bottom => {
            let sh = container.scroll_height();
            let ch = container.client_height();
            let target = (sh - ch).max(0);
            container.set_scroll_top(target);
            true
        }
        ChannelScrollAnchor::UnreadMarker(unread_id) => {
            let selector = format!("[data-unread-marker='{unread_id}']");
            if let Ok(Some(marker_el)) = container.query_selector(&selector) {
                let m_rect = marker_el.get_bounding_client_rect();
                let diff = m_rect.top() - c_rect.top();
                let target = (current_scroll_top as f64 + diff - 50.0).max(0.0);
                container.set_scroll_top(target.round() as i32);
                true
            } else {
                false
            }
        }
        ChannelScrollAnchor::Message { message_id, offset } => {
            let selector = format!(".message[data-message-id='{message_id}']");
            if let Ok(Some(msg_el)) = container.query_selector(&selector) {
                let el_rect = msg_el.get_bounding_client_rect();
                let diff = el_rect.top() - c_rect.top();
                let target = (current_scroll_top as f64 + diff + (*offset as f64)).max(0.0);
                container.set_scroll_top(target.round() as i32);
                true
            } else {
                false
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn apply_channel_scroll_anchor(
    _container: &web_sys::HtmlElement,
    _anchor: &ChannelScrollAnchor,
) -> bool {
    true
}

/// Надежное восстановление скролла с повторными проверками в кадрах анимации браузера
#[cfg(target_arch = "wasm32")]
pub fn execute_initial_scroll_restoration(
    container: web_sys::HtmlElement,
    anchor: ChannelScrollAnchor,
) {
    apply_channel_scroll_anchor(&container, &anchor);

    if let Some(window) = web_sys::window() {
        let c1 = container.clone();
        let a1 = anchor.clone();
        let cb = wasm_bindgen::closure::Closure::once_into_js(move || {
            let applied2 = apply_channel_scroll_anchor(&c1, &a1);

            // Дополнительная проверка на втором кадре при динамической загрузке вложений/карточек
            if !applied2 || matches!(a1, ChannelScrollAnchor::Bottom) {
                if let Some(w) = web_sys::window() {
                    let c2 = c1.clone();
                    let a2 = a1.clone();
                    let cb2 = wasm_bindgen::closure::Closure::once_into_js(move || {
                        apply_channel_scroll_anchor(&c2, &a2);
                    });
                    let _ = w.request_animation_frame(cb2.as_ref().unchecked_ref());
                }
            }
        });
        let _ = window.request_animation_frame(cb.as_ref().unchecked_ref());
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn execute_initial_scroll_restoration(
    _container: web_sys::HtmlElement,
    _anchor: ChannelScrollAnchor,
) {}

#[allow(dead_code)]
pub fn scroll_to_bottom(container: &web_sys::HtmlElement) {
    #[cfg(target_arch = "wasm32")]
    container.set_scroll_top(container.scroll_height());
    #[cfg(not(target_arch = "wasm32"))]
    let _ = container;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_message(id: MessageId, ts: i64) -> MessageView {
        MessageView {
            id,
            channel_id: ChannelId::default(),
            author_id: None,
            author_name: "test_user".to_string(),
            content: "hello".to_string(),
            edited: false,
            reply_to_id: None,
            forwarded_from: None,
            created_at: time::OffsetDateTime::from_unix_timestamp(ts).unwrap(),
            is_pending: false,
            is_failed: false,
        }
    }

    #[test]
    fn test_scenario_1_user_at_bottom_no_unreads_stays_at_bottom() {
        let m1 = mock_message(MessageId::default(), 100);
        let m2 = mock_message(MessageId::default(), 200);
        let msgs = vec![m1.clone(), m2.clone()];

        // Saved read watermark = m2 (latest message)
        let saved_read = Some((Some(m2.id), Some(200)));
        // Saved scroll state = at_bottom true
        let scroll_state = (true, None, 0);

        let (target, unread) = determine_target_scroll_pure(scroll_state, saved_read, &msgs);
        assert_eq!(target, ChannelScrollAnchor::Bottom);
        assert_eq!(unread, None);
    }

    #[test]
    fn test_scenario_2_user_at_bottom_new_messages_arrived_anchors_to_unread() {
        let m1 = mock_message(MessageId::default(), 100);
        let m2 = mock_message(MessageId::default(), 200);
        let m3 = mock_message(MessageId::default(), 300);
        let msgs = vec![m1.clone(), m2.clone(), m3.clone()];

        // Saved read watermark = m1 (user read up to m1)
        let saved_read = Some((Some(m1.id), Some(100)));
        // Saved scroll state = was at bottom before m2 and m3 arrived
        let scroll_state = (true, None, 0);

        let (target, unread) = determine_target_scroll_pure(scroll_state, saved_read, &msgs);
        assert_eq!(target, ChannelScrollAnchor::UnreadMarker(m2.id));
        assert_eq!(unread, Some(m2.id));
    }

    #[test]
    fn test_scenario_3_user_scrolled_up_returns_to_exact_message() {
        let m1 = mock_message(MessageId::default(), 100);
        let m2 = mock_message(MessageId::default(), 200);
        let m3 = mock_message(MessageId::default(), 300);
        let msgs = vec![m1.clone(), m2.clone(), m3.clone()];

        // User scrolled up to m1 with 25px offset
        let scroll_state = (false, Some(m1.id), 25);
        // Even if new messages m2, m3 arrived
        let saved_read = Some((Some(m1.id), Some(100)));

        let (target, unread) = determine_target_scroll_pure(scroll_state, saved_read, &msgs);
        assert_eq!(
            target,
            ChannelScrollAnchor::Message {
                message_id: m1.id,
                offset: 25,
            }
        );
        assert_eq!(unread, Some(m2.id));
    }

    #[test]
    fn test_initial_visit_empty_history_defaults_to_bottom() {
        let msgs: Vec<MessageView> = vec![];
        let scroll_state = (true, None, 0);
        let saved_read = None;

        let (target, unread) = determine_target_scroll_pure(scroll_state, saved_read, &msgs);
        assert_eq!(target, ChannelScrollAnchor::Bottom);
        assert_eq!(unread, None);
    }
}
