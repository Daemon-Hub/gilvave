//! Blocked users management tab with unblock actions.

use sycamore::prelude::*;
use gilvave_core::{
    dto::users::friends::BlockedUserView,
    ids::UserId,
};
use crate::components::ui::icons::{BanIcon, SearchIcon};

pub(crate) fn view_blocked<F>(
    blocked_signal: Signal<Vec<BlockedUserView>>,
    search_signal: Signal<String>,
    is_loading: Signal<bool>,
    on_unblock: F,
) -> View
where
    F: Fn(UserId) + Copy + 'static,
{
    let filtered_blocked = create_memo(move || {
        let q = search_signal.get_clone().trim().to_lowercase();
        let list = blocked_signal.get_clone();
        if q.is_empty() {
            list
        } else {
            list.into_iter()
                .filter(|b| {
                    b.username.to_lowercase().contains(&q)
                        || b.display_name.to_lowercase().contains(&q)
                })
                .collect()
        }
    });

    view! {
        div(class="friends-tab-content") {
            div(class="friends-search-bar") {
                div(class="friends-search-input-wrap") {
                    SearchIcon()
                    input(
                        r#type="text",
                        placeholder="Поиск среди заблокированных...",
                        bind:value=search_signal,
                    )
                }
            }

            div(class="friends-section-header") {
                span { (format!("ЗАБЛОКИРОВАННЫЕ — {}", filtered_blocked.with(|b| b.len()))) }
            }

            (if is_loading.get() && blocked_signal.with(|b| b.is_empty()) {
                view! {
                    div(class="friends-loading-state") {
                        span { "Загрузка списка заблокированных..." }
                    }
                }
            } else if filtered_blocked.with(|b| b.is_empty()) {
                view! {
                    div(class="friends-empty-state") {
                        div(class="empty-icon-circle") { BanIcon() }
                        p(class="empty-title") { "Список блокировок пуст" }
                        p(class="empty-subtitle") { "У вас нет заблокированных пользователей." }
                    }
                }
            } else {
                view! {
                    div(class="friends-list") {
                        Indexed(
                            list=filtered_blocked,
                            view=move |b| {
                                let b_id = b.id;
                                let b_name = b.display_name;
                                let b_username = b.username;
                                let b_avatar = b.avatar;

                                let initial = b_name
                                    .chars()
                                    .next()
                                    .unwrap_or('?')
                                    .to_uppercase()
                                    .to_string();

                                let avatar_view = if !b_avatar.is_empty() {
                                    view! { img(src=b_avatar, alt="", class="friend-avatar-img") }
                                } else {
                                    view! {
                                        div(class="friend-avatar-fallback") {
                                            span { (initial) }
                                        }
                                    }
                                };

                                let b_id_str = b_id.to_string();
                                let b_username_tag = format!("@{b_username}");

                                view! {
                                    div(
                                        class="friend-row-item",
                                        data-user-id=b_id_str,
                                        data-username=b_username,
                                    ) {
                                        div(class="friend-avatar-wrap") {
                                            (avatar_view)
                                        }

                                        div(class="friend-main-info") {
                                            div(class="friend-names-row") {
                                                span(class="friend-display-name") { (b_name) }
                                                span(class="friend-username-tag") { (b_username_tag) }
                                            }
                                            span(class="friend-blocked-tag") { "Заблокирован" }
                                        }

                                        div(class="friend-row-actions") {
                                            button(
                                                class="unblock-btn",
                                                on:click=move |_| on_unblock(b_id),
                                            ) {
                                                span { "Разблокировать" }
                                            }
                                        }
                                    }
                                }
                            },
                        )
                    }
                }
            })
        }
    }
}


