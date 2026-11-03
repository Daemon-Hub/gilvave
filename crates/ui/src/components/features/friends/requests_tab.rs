//! Pending incoming and outgoing friend requests tab.

use sycamore::prelude::*;
use gilvave_core::{
    dto::users::friends::{FriendRequestsList, FriendRequestView},
    ids::UserId,
};
use crate::components::ui::icons::{CheckIcon, CloseSmallIcon, UsersIcon};
use super::format_friend_date;

pub(crate) fn view_requests<F1, F2>(
    requests_signal: Signal<FriendRequestsList>,
    is_loading: Signal<bool>,
    on_accept: F1,
    on_decline: F2,
) -> View
where
    F1: Fn(UserId) + Copy + 'static,
    F2: Fn(UserId) + Copy + 'static,
{
    let incoming = create_memo(move || requests_signal.with(|r| r.incoming.clone()));
    let outgoing = create_memo(move || requests_signal.with(|r| r.outgoing.clone()));

    let has_any = create_memo(move || {
        requests_signal.with(|r| !r.incoming.is_empty() || !r.outgoing.is_empty())
    });

    view! {
        div(class="friends-tab-content") {
            (if is_loading.get() && !has_any.get() {
                view! {
                    div(class="friends-loading-state") {
                        span { "Загрузка запросов в друзья..." }
                    }
                }
            } else if !has_any.get() {
                view! {
                    div(class="friends-empty-state") {
                        div(class="empty-icon-circle") { UsersIcon() }
                        p(class="empty-title") { "Нет ожидающих запросов" }
                        p(class="empty-subtitle") {
                            "Здесь будут отображаться новые входящие и отправленные вами приглашения в друзья."
                        }
                    }
                }
            } else {
                view! {
                    div(class="requests-container") {
                        // ─── Incoming Requests Section ─────────────────────────
                        (if !incoming.with(|i| i.is_empty()) {
                            let in_count = incoming.with(|i| i.len());
                            view! {
                                div(class="friends-section-header") {
                                    span { (format!("ВХОДЯЩИЕ ЗАПРОСЫ — {in_count}")) }
                                }
                                div(class="friends-list") {
                                    Indexed(
                                        list=incoming,
                                        view=move |req: FriendRequestView| {
                                            let user = req.user;
                                            let u_id = user.id;
                                            let u_name = user.display_name;
                                            let u_user = user.username;
                                            let u_avatar = user.avatar;
                                            let date_str = format_friend_date(req.created_at);

                                            let initial = u_name
                                                .chars()
                                                .next()
                                                .unwrap_or('?')
                                                .to_uppercase()
                                                .to_string();

                                            let avatar_view = if !u_avatar.is_empty() {
                                                view! { img(src=u_avatar, alt="", class="friend-avatar-img") }
                                            } else {
                                                view! {
                                                    div(class="friend-avatar-fallback") {
                                                        span { (initial) }
                                                    }
                                                }
                                            };

                                            view! {
                                                div(class="friend-row-item") {
                                                    div(class="friend-avatar-wrap") {
                                                        (avatar_view)
                                                    }

                                                    div(class="friend-main-info") {
                                                        div(class="friend-names-row") {
                                                            span(class="friend-display-name") { (u_name) }
                                                            span(class="friend-username-tag") { (format!("@{u_user}")) }
                                                        }
                                                        span(class="friend-added-date") { (format!("Запрос получен {date_str}")) }
                                                    }

                                                    div(class="friend-row-actions") {
                                                        button(
                                                            class="request-btn accept",
                                                            title="Принять запрос в друзья",
                                                            on:click=move |_| on_accept(u_id),
                                                        ) {
                                                            CheckIcon()
                                                            span { "Принять" }
                                                        }
                                                        button(
                                                            class="request-btn decline",
                                                            title="Отклонить запрос",
                                                            on:click=move |_| on_decline(u_id),
                                                        ) {
                                                            CloseSmallIcon()
                                                            span { "Отклонить" }
                                                        }
                                                    }
                                                }
                                            }
                                        },
                                    )
                                }
                            }
                        } else {
                            view! {}
                        })

                        // ─── Outgoing Requests Section ─────────────────────────
                        (if !outgoing.with(|o| o.is_empty()) {
                            let out_count = outgoing.with(|o| o.len());
                            view! {
                                div(class="friends-section-header") {
                                    span { (format!("ОТПРАВЛЕННЫЕ ЗАПРОСЫ — {out_count}")) }
                                }
                                div(class="friends-list") {
                                    Indexed(
                                        list=outgoing,
                                        view=move |req: FriendRequestView| {
                                            let user = req.user;
                                            let u_id = user.id;
                                            let u_name = user.display_name;
                                            let u_user = user.username;
                                            let u_avatar = user.avatar;
                                            let date_str = format_friend_date(req.created_at);

                                            let initial = u_name
                                                .chars()
                                                .next()
                                                .unwrap_or('?')
                                                .to_uppercase()
                                                .to_string();

                                            let avatar_view = if !u_avatar.is_empty() {
                                                view! { img(src=u_avatar, alt="", class="friend-avatar-img") }
                                            } else {
                                                view! {
                                                    div(class="friend-avatar-fallback") {
                                                        span { (initial) }
                                                    }
                                                }
                                            };

                                            view! {
                                                div(class="friend-row-item") {
                                                    div(class="friend-avatar-wrap") {
                                                        (avatar_view)
                                                    }

                                                    div(class="friend-main-info") {
                                                        div(class="friend-names-row") {
                                                            span(class="friend-display-name") { (u_name) }
                                                            span(class="friend-username-tag") { (format!("@{u_user}")) }
                                                        }
                                                        span(class="friend-added-date") { (format!("Отправлен {date_str}")) }
                                                    }

                                                    div(class="friend-row-actions") {
                                                        button(
                                                            class="request-btn decline",
                                                            title="Отменить отправленный запрос",
                                                            on:click=move |_| on_decline(u_id),
                                                        ) {
                                                            CloseSmallIcon()
                                                            span { "Отменить" }
                                                        }
                                                    }
                                                }
                                            }
                                        },
                                    )
                                }
                            }
                        } else {
                            view! {}
                        })
                    }
                }
            })
        }
    }
}


