//! Session card rendering and formatting for active login sessions.

use std::rc::Rc;
use gilvave_core::dto::users::SessionView;
use sycamore::prelude::*;
use crate::{
    components::ui::icons::{GlobeIcon, MonitorIcon, SmartphoneIcon, TrashIcon},
    utils::to_local_datetime,
};

pub fn format_last_active(dt: time::OffsetDateTime, is_current: bool) -> String {
    if is_current {
        return "В сети прямо сейчас".to_string();
    }
    let now = time::OffsetDateTime::now_utc();
    let diff = now - dt;
    if diff.is_negative() || diff.whole_seconds() < 60 {
        return "Только что".to_string();
    }
    let minutes = diff.whole_minutes();
    if minutes < 60 {
        return format!("{minutes} мин. назад");
    }
    let hours = diff.whole_hours();
    if hours < 24 {
        return format!("{hours} ч. назад");
    }
    let days = diff.whole_days();
    if days == 1 {
        let local_dt = to_local_datetime(dt);
        return format!("Вчера в {:02}:{:02}", local_dt.hour(), local_dt.minute());
    }
    if days < 7 {
        return format!("{days} дн. назад");
    }
    let local_dt = to_local_datetime(dt);
    format!(
        "{:02}.{:02}.{}",
        local_dt.day(),
        local_dt.month() as u8,
        local_dt.year()
    )
}

pub fn format_created_date(dt: time::OffsetDateTime) -> String {
    let local_dt = to_local_datetime(dt);
    format!(
        "{:02}.{:02}.{}",
        local_dt.day(),
        local_dt.month() as u8,
        local_dt.year()
    )
}

#[component(inline_props)]
pub fn CurrentSessionCard(session: SessionView) -> View {
    let client_kind = session.client_kind();
    let title = session.display_name();
    let client_version = format!(
        "{} v{}",
        session.device_info.client, session.device_info.client_version
    );
    let ip = session.ip_address.unwrap_or_else(|| "127.0.0.1".to_string());
    let location = session.location.unwrap_or_else(|| "Локальная сеть".to_string());
    let created_str = format_created_date(session.created_at);

    view! {
        div(class="session-card current-session-card") {
            div(class="session-card-left") {
                div(class=format!("session-device-icon-bubble current {client_kind}")) {
                    (match client_kind {
                        "desktop" => view! { MonitorIcon() },
                        "mobile" => view! { SmartphoneIcon() },
                        _ => view! { GlobeIcon() },
                    })
                }
            }

            div(class="session-card-center") {
                div(class="session-title-row") {
                    span(class="session-device-title") { (title) }
                    span(class="session-client-badge current") { (client_version) }
                }

                div(class="session-meta-row") {
                    span(class="session-meta-item location") {
                        "📍 " (location)
                    }
                    span(class="session-meta-divider") { "•" }
                    span(class="session-meta-item ip") {
                        (ip)
                    }
                    span(class="session-meta-divider") { "•" }
                    span(class="session-meta-item created") {
                        "Вход: " (created_str)
                    }
                }
            }

            div(class="session-card-right") {
                span(class="session-online-pill") {
                    span(class="online-pulse-dot")
                    span { "Это устройство" }
                }
            }
        }
    }
}

#[component(inline_props)]
pub fn OtherSessionCard(session: SessionView, on_terminate: Rc<dyn Fn()>) -> View {
    let sess_id = session.id.clone();
    let title = session.display_name();
    let client_kind = session.client_kind();
    let client_version = format!(
        "{} v{}",
        session.device_info.client, session.device_info.client_version
    );
    let ip = session.ip_address.unwrap_or_else(|| "Неизвестный IP".to_string());
    let location = session.location.unwrap_or_else(|| "Неизвестное местоположение".to_string());
    let last_active = format_last_active(session.last_used_at, session.is_current);

    let on_terminate_click = on_terminate.clone();

    view! {
        div(class="session-card other-session-card", id=sess_id) {
            div(class="session-card-left") {
                div(class=format!("session-device-icon-bubble {client_kind}")) {
                    (match client_kind {
                        "desktop" => view! { MonitorIcon() },
                        "mobile" => view! { SmartphoneIcon() },
                        _ => view! { GlobeIcon() },
                    })
                }
            }

            div(class="session-card-center") {
                div(class="session-title-row") {
                    span(class="session-device-title") { (title) }
                    span(class="session-client-badge") { (client_version) }
                }

                div(class="session-meta-row") {
                    span(class="session-meta-item location") {
                        "📍 " (location)
                    }
                    span(class="session-meta-divider") { "•" }
                    span(class="session-meta-item ip") {
                        (ip)
                    }
                    span(class="session-meta-divider") { "•" }
                    span(class="session-meta-item last-active") {
                        "Был(а) в сети: " (last_active)
                    }
                }
            }

            div(class="session-card-right") {
                button(
                    class="terminate-single-btn",
                    on:click=move |_| on_terminate_click(),
                    title="Завершить эту сессию",
                ) {
                    TrashIcon()
                    span { "Завершить" }
                }
            }
        }
    }
}
