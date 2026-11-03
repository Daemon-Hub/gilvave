//! Active user sessions tab with device details, termination actions, and security advice.

mod security_tip;
mod session_card;

pub use security_tip::{EmptySessionsCard, SecurityTipCard, TerminateConfirmBanner};
pub use session_card::{CurrentSessionCard, OtherSessionCard};

use std::rc::Rc;
use gilvave_core::dto::users::SessionView;
use sycamore::{futures::spawn_local_scoped, prelude::*};

use crate::{
    components::ui::icons::{CloseSmallIcon, LogOutIcon, RefreshCwIcon},
    http::api::Api,
};

async fn fetch_sessions_data(
    sessions_sig: Signal<Vec<SessionView>>,
    initial_load_sig: Signal<bool>,
) {
    match Api::get_sessions().await {
        Ok(server_sessions) => {
            tracing::info!(
                "[SessionsTab] Received {} sessions from server",
                server_sessions.len()
            );
            let list: Vec<SessionView> = server_sessions
                .into_iter()
                .map(|(mut s, c)| {
                    s.is_current = c;
                    s
                })
                .collect();
            sessions_sig.set(list);
        }
        Err(err) => {
            tracing::error!("[SessionsTab] Failed to fetch sessions: {:?}", err);
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
                        view! {
                            CurrentSessionCard(session=curr)
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
                            TerminateConfirmBanner(
                                on_confirm=on_terminate_all,
                                on_cancel=move |_| confirm_terminate_all.set(false),
                            )
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
                            EmptySessionsCard()
                        }
                    } else {
                        let other_list = other_sessions.get_clone();
                        let cards: Vec<View> = other_list
                            .into_iter()
                            .map(|session| {
                                let sess_id_del = session.id.clone();
                                let on_single_terminate: Rc<dyn Fn()> = Rc::new(move || {
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
                                });

                                view! {
                                    OtherSessionCard(
                                        session=session,
                                        on_terminate=on_single_terminate,
                                    )
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
            SecurityTipCard()
        }
    }
}
