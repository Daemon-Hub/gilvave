//! All friends list tab with online status sorting, search filtering, and action buttons.

use sycamore::prelude::*;
use gilvave_core::{
    dto::users::friends::FriendView,
    ids::UserId,
};
use crate::components::ui::icons::{
    BanIcon, ChatBubbleIcon, CloseSmallIcon, SearchIcon, TrashIcon, UserPlusIcon, UsersIcon,
};
use super::format_friend_date;

pub(crate) fn view_all_friends<F1, F2, F3, F4>(
    friends_signal: Signal<Vec<FriendView>>,
    search_signal: Signal<String>,
    is_loading: Signal<bool>,
    on_start_chat: F1,
    on_block: F2,
    on_remove: F3,
    on_open_add: F4,
) -> View
where
    F1: Fn(String) + Copy + 'static,
    F2: Fn(UserId) + Copy + 'static,
    F3: Fn(UserId) + Copy + 'static,
    F4: Fn() + Copy + 'static,
{
    let filtered_friends = create_memo(move || {
        let q = search_signal.get_clone().trim().to_lowercase();
        let list = friends_signal.get_clone();
        if q.is_empty() {
            list
        } else {
            list.into_iter()
                .filter(|f| {
                    f.username.to_lowercase().contains(&q)
                        || f.display_name.to_lowercase().contains(&q)
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
                        placeholder="Поиск среди друзей...",
                        bind:value=search_signal,
                    )
                    (if !search_signal.with(|s| s.is_empty()) {
                        view! {
                            button(
                                class="search-clear-btn",
                                on:click=move |_| search_signal.set(String::new()),
                            ) {
                                CloseSmallIcon()
                            }
                        }
                    } else {
                        view! {}
                    })
                }
            }

            div(class="friends-section-header") {
                span { (format!("ВСЕ ДРУЗЬЯ — {}", filtered_friends.with(|f| f.len()))) }
            }

            (if is_loading.get() && friends_signal.with(|f| f.is_empty()) {
                view! {
                    div(class="friends-loading-state") {
                        span { "Загрузка списка друзей..." }
                    }
                }
            } else if filtered_friends.with(|f| f.is_empty()) {
                if search_signal.with(|s| !s.is_empty()) {
                    view! {
                        div(class="friends-empty-state") {
                            p(class="empty-title") { "Ничего не найдено" }
                            p(class="empty-subtitle") { "Попробуйте изменить поисковый запрос" }
                        }
                    }
                } else {
                    view! {
                        div(class="friends-empty-state") {
                            div(class="empty-icon-circle") { UsersIcon() }
                            p(class="empty-title") { "У вас пока нет друзей" }
                            p(class="empty-subtitle") {
                                "Добавьте друзей по имени пользователя во вкладке «Добавить в друзья», чтобы общаться и созваниваться!"
                            }
                            button(class="empty-action-btn", on:click=move |_| on_open_add()) {
                                UserPlusIcon()
                                span { "Найти друга" }
                            }
                        }
                    }
                }
            } else {
                view! {
                    div(class="friends-list") {
                        Indexed(
                            list=filtered_friends,
                            view=move |friend| {
                                let f_id = friend.id;
                                let f_name = friend.display_name.clone();
                                let f_username = friend.username.clone();
                                let f_avatar = friend.avatar.clone();
                                let date_str = format_friend_date(friend.created_at);

                                let initial = f_name
                                    .chars()
                                    .next()
                                    .unwrap_or('?')
                                    .to_uppercase()
                                    .to_string();

                                let avatar_view = if !f_avatar.is_empty() {
                                    view! { img(src=f_avatar, alt="", class="friend-avatar-img") }
                                } else {
                                    view! {
                                        div(class="friend-avatar-fallback") {
                                            span { (initial) }
                                        }
                                    }
                                };

                                let name_for_chat = f_name.clone();
                                let f_id_str = f_id.to_string();
                                let f_username_tag = format!("@{f_username}");

                                view! {
                                    div(
                                        class="friend-row-item",
                                        data-user-id=f_id_str,
                                        data-username=f_username,
                                    ) {
                                        div(class="friend-avatar-wrap") {
                                            (avatar_view)
                                        }

                                        div(class="friend-main-info") {
                                            div(class="friend-names-row") {
                                                span(class="friend-display-name") { (f_name) }
                                                span(class="friend-username-tag") { (f_username_tag) }
                                            }
                                            span(class="friend-added-date") { (format!("В друзьях с {date_str}")) }
                                        }

                                        div(class="friend-row-actions") {
                                            button(
                                                class="friend-action-icon-btn chat",
                                                title="Написать личное сообщение",
                                                on:click=move |_| on_start_chat(name_for_chat.clone()),
                                            ) {
                                                ChatBubbleIcon()
                                            }
                                            button(
                                                class="friend-action-icon-btn block",
                                                title="Заблокировать пользователя",
                                                on:click=move |_| on_block(f_id),
                                            ) {
                                                BanIcon()
                                            }
                                            button(
                                                class="friend-action-icon-btn delete",
                                                title="Удалить из друзей",
                                                on:click=move |_| on_remove(f_id),
                                            ) {
                                                TrashIcon()
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


