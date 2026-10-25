use gilvave_core::{
    dto::{
        command::{CommandArgs, CommandResponse, CommandResult},
        users::SessionView,
    },
    settings::collect_device_info,
};
use sycamore::{futures::spawn_local_scoped, prelude::*};

use crate::{
    components::ui::icons::{
        CloseSmallIcon, GlobeIcon, LogOutIcon, MonitorIcon, RefreshCwIcon, ShieldCheckIcon,
        SmartphoneIcon, TrashIcon,
    },
    http::api::Api,
    utils::{invoke_command, to_local_datetime},
};
fn format_last_active(dt: time::OffsetDateTime, is_current: bool) -> String {
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

fn format_created_date(dt: time::OffsetDateTime) -> String {
    let local_dt = to_local_datetime(dt);
    format!(
        "{:02}.{:02}.{}",
        local_dt.day(),
        local_dt.month() as u8,
        local_dt.year()
    )
}

async fn fetch_sessions_data(
    sessions_sig: Signal<Vec<SessionView>>,
    initial_load_sig: Signal<bool>,
) {
    // 1. Collect real device info from Tauri desktop or Web browser runtime
    let current_device = match invoke_command(CommandArgs::GetDeviceInfo.to_json()).await {
        CommandResult::Ok(CommandResponse::GetDeviceInfo(info)) => info,
        _ => collect_device_info(),
    };

    // 2. Fetch from backend API
    match Api::get_sessions().await {
        Ok(server_sessions) => {
            tracing::info!(
                "[SessionsTab] Received {} sessions from server",
                server_sessions.len()
            );
            let mut list: Vec<SessionView> = server_sessions
                .into_iter()
                .map(|(mut s, c)| {
                    s.is_current = c;
                    s
                })
                .collect();
            let has_current = list.iter().any(|s| s.is_current);
            if !has_current {
                list.insert(
                    0,
                    SessionView {
                        id: "curr-device".to_string(),
                        device_info: current_device,
                        ip_address: Some("127.0.0.1".to_string()),
                        location: Some("Текущее рабочее место".to_string()),
                        last_used_at: time::OffsetDateTime::now_utc(),
                        created_at: time::OffsetDateTime::now_utc(),
                        is_current: true,
                    },
                );
            }
            sessions_sig.set(list);
        }
        Err(err) => {
            tracing::error!("[SessionsTab] Failed to fetch sessions: {:?}", err);
            // Fallback: show the current local device as the active session if list was empty
            if sessions_sig.with(|v| v.is_empty()) {
                let now = time::OffsetDateTime::now_utc();
                let fallback_list = vec![SessionView {
                    id: "curr-session-local".to_string(),
                    device_info: current_device,
                    ip_address: Some("127.0.0.1 (Локальная сеть)".to_string()),
                    location: Some("Текущее рабочее место".to_string()),
                    last_used_at: now,
                    created_at: now,
                    is_current: true,
                }];
                sessions_sig.set(fallback_list);
            }
        }
    }

    initial_load_sig.set(false);
}

#[component]
pub fn SessionsTab() -> View {
    let sessions = create_signal::<Vec<SessionView>>(Vec::new());
    let is_initial_load = create_signal(true);
    let is_refreshing = create_signal(false);
    let status_msg = create_signal::<Option<(String, bool)>>(None);
    let confirm_terminate_all = create_signal(false);

    // Initial load
    spawn_local_scoped(async move {
        fetch_sessions_data(sessions, is_initial_load).await;
    });

    let on_refresh = move |_| {
        if is_refreshing.get() {
            return;
        }
        spawn_local_scoped(async move {
            is_refreshing.set(true);
            let fetch_fut = fetch_sessions_data(sessions, is_initial_load);
            let timer_fut = gloo_timers::future::sleep(core::time::Duration::from_millis(600));
            futures_util::join!(fetch_fut, timer_fut);
            is_refreshing.set(false);
        });
    };

    let on_terminate_all = move |_| {
        spawn_local_scoped(async move {
            match Api::terminate_all_other_sessions().await {
                Ok(_) => {
                    let mut list = sessions.get_clone();
                    list.retain(|s| s.is_current);
                    sessions.set(list);
                    confirm_terminate_all.set(false);
                    status_msg.set(Some((
                        "Все остальные сессии успешно завершены.".to_string(),
                        false,
                    )));
                }
                Err(err) => {
                    tracing::error!(
                        "[SessionsTab] terminate_all_other_sessions error: {:?}",
                        err
                    );
                    confirm_terminate_all.set(false);
                    status_msg.set(Some((format!("Ошибка завершения сессий: {}", err.1), true)));
                }
            }
        });
    };

    let dismiss_status = move |_| {
        status_msg.set(None);
    };

    let current_session =
        create_memo(move || sessions.get_clone().into_iter().find(|s| s.is_current));

    let other_sessions = create_memo(move || {
        sessions
            .get_clone()
            .into_iter()
            .filter(|s| !s.is_current)
            .collect::<Vec<SessionView>>()
    });

    view! {
        div(class="tab-pane sessions-tab-pane") {
            div(class="sessions-header-row") {
                div(class="sessions-header-titles") {
                    h3(class="tab-title") { "Активные сессии" }
                    p(class="sessions-header-desc") {
                        "Список устройств и приложений, где выполнен вход в вашу учётную запись."
                    }
                }
            }

            // Notification / Status Toast
            (if let Some((msg, is_err)) = status_msg.get_clone() {
                let alert_class = if is_err {
                    "sessions-alert error"
                } else {
                    "sessions-alert success"
                };
                view! {
                    div(class=alert_class) {
                        span(class="alert-text") { (msg) }
                        button(class="alert-close-btn", on:click=dismiss_status) {
                            CloseSmallIcon()
                        }
                    }
                }
            } else {
                view! {}
            })

            // ── Section 1: Current Session ──
            div(class="sessions-section") {
                h4(class="sessions-section-title") { "ТЕКУЩЕЕ УСТРОЙСТВО" }

                (move || {
                    if let Some(curr) = current_session.get_clone() {
                        let client_kind = curr.client_kind();
                        let title = curr.display_name();
                        let client_version = format!(
                            "{} v{}",
                            curr.device_info.client,
                            curr.device_info.client_version
                        );
                        let ip = curr.ip_address.unwrap_or_else(|| "127.0.0.1".to_string());
                        let location = curr.location.unwrap_or_else(|| "Локальная сеть".to_string());
                        let created_str = format_created_date(curr.created_at);

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
                    } else if is_initial_load.get() {
                        view! {
                            div(class="session-card-skeleton") {
                                span { "Загрузка информации о текущем устройстве..." }
                            }
                        }
                    } else {
                        view! {}
                    }
                })
            }

            // ── Section 2: Other Sessions ──
            div(class="sessions-section") {
                div(class="sessions-section-header") {
                    h4(class="sessions-section-title") {
                        "ДРУГИЕ АКТИВНЫЕ СЕССИИ"
                        (move || {
                            let count = other_sessions.with(|v| v.len());
                            if count > 0 {
                                format!(" ({count})")
                            } else {
                                String::new()
                            }
                        })
                    }

                    div(class="sessions-section-actions") {
                        (move || {
                            let has_others = other_sessions.with(|v| !v.is_empty());
                            if has_others && !confirm_terminate_all.get() {
                                view! {
                                    button(
                                        class="terminate-all-action-btn",
                                        on:click=move |_| confirm_terminate_all.set(true),
                                    ) {
                                        LogOutIcon()
                                        span { "Завершить все остальные" }
                                    }
                                }
                            } else {
                                view! {}
                            }
                        })

                        button(
                            class=move || {
                                if is_refreshing.get() {
                                    "sessions-refresh-icon-btn is-loading"
                                } else {
                                    "sessions-refresh-icon-btn"
                                }
                            },
                            disabled=is_refreshing.get(),
                            on:click=on_refresh,
                            title="Обновить список сессий",
                        ) {
                            RefreshCwIcon()
                        }
                    }
                }

                // Confirmation Prompt for Terminate All
                (move || {
                    if confirm_terminate_all.get() {
                        view! {
                            div(class="terminate-confirm-banner") {
                                div(class="confirm-banner-content") {
                                    strong { "Завершить все остальные сессии?" }
                                    p {
                                        "Вы будете разлогинены на всех устройствах, кроме текущего. Для повторного входа потребуется ввести пароль."
                                    }
                                }
                                div(class="confirm-banner-actions") {
                                    button(
                                        class="confirm-cancel-btn",
                                        on:click=move |_| confirm_terminate_all.set(false),
                                    ) {
                                        "Отмена"
                                    }
                                    button(
                                        class="confirm-danger-btn",
                                        on:click=on_terminate_all,
                                    ) {
                                        "Да, завершить все"
                                    }
                                }
                            }
                        }
                    } else {
                        view! {}
                    }
                })

                // Other sessions list or empty state
                (move || {
                    let is_empty = other_sessions.with(|v| v.is_empty());
                    if is_initial_load.get() {
                        view! {
                            div(class="sessions-loading-state") {
                                span { "Загрузка списка сессий..." }
                            }
                        }
                    } else if is_empty {
                        view! {
                            div(class="sessions-empty-card") {
                                div(class="empty-icon-shield") {
                                    ShieldCheckIcon()
                                }
                                h5 { "Других активных сессий нет" }
                                p {
                                    "Вход в ваш аккаунт выполнен только с этого устройства. Никаких подозрительных входов не обнаружено."
                                }
                            }
                        }
                    } else {
                        let other_list = other_sessions.get_clone();
                        let cards: Vec<View> = other_list
                            .into_iter()
                            .map(|session| {
                                let sess_id = session.id.clone();
                                let sess_id_del = session.id.clone();
                                let title = session.display_name();
                                let client_kind = session.client_kind();
                                let client_version = format!(
                                    "{} v{}",
                                    session.device_info.client,
                                    session.device_info.client_version
                                );
                                let ip = session
                                    .ip_address
                                    .unwrap_or_else(|| "Неизвестный IP".to_string());
                                let location = session
                                    .location
                                    .unwrap_or_else(|| "Неизвестное местоположение".to_string());
                                let last_active = format_last_active(
                                    session.last_used_at,
                                    session.is_current,
                                );

                                let on_single_terminate = move |_| {
                                    let target_id = sess_id_del.clone();
                                    spawn_local_scoped(async move {
                                        match Api::terminate_session(&target_id).await {
                                            Ok(_) => {
                                                let mut list = sessions.get_clone();
                                                list.retain(|s| s.id != target_id);
                                                sessions.set(list);
                                                status_msg.set(Some((
                                                    "Сессия завершена.".to_string(),
                                                    false,
                                                )));
                                            }
                                            Err(err) => {
                                                tracing::error!("[SessionsTab] terminate_session error: {:?}", err);
                                                status_msg.set(Some((
                                                    format!("Ошибка завершения сессии: {}", err.1),
                                                    true,
                                                )));
                                            }
                                        }
                                    });
                                };

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
                                                on:click=on_single_terminate,
                                                title="Завершить эту сессию",
                                            ) {
                                                TrashIcon()
                                                span { "Завершить" }
                                            }
                                        }
                                    }
                                }
                            })
                            .collect();

                        view! {
                            div(class="other-sessions-list") {
                                (cards)
                            }
                        }
                    }
                })
            }

            // ── Section 3: Security Advice ──
            div(class="sessions-security-tip-card") {
                div(class="tip-icon") {
                    ShieldCheckIcon()
                }
                div(class="tip-content") {
                    h5 { "Безопасность аккаунта" }
                    p {
                        "Если вы увидели незнакомое устройство в списке, немедленно завершите его сессию и измените пароль во вкладке «Безопасность». Это предотвратит несанкционированный доступ к вашей переписке и серверам."
                    }
                }
            }
        }
    }
}
