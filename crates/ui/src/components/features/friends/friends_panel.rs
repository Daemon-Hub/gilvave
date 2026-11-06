use super::{
    add_friend_tab::view_add_friend, all_friends_tab::view_all_friends, blocked_tab::view_blocked,
    requests_tab::view_requests,
};
use sycamore::{futures::spawn_local_scoped, prelude::*};

use gilvave_core::{
    dto::users::friends::{BlockedUserView, FriendRequestsList, SendFriendRequest},
    ids::UserId,
};

use crate::{
    components::{
        common::{HomeTab, UiModalContext, UserProfileContext},
        ui::icons::{CloseSmallIcon, RefreshCwIcon, UserPlusIcon, UsersIcon},
    },
    http::api::Api,
};

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum FriendsTab {
    #[default]
    All,
    Requests,
    Blocked,
    AddFriend,
}

#[component]
pub fn FriendsPanel() -> View {
    let modal_context = use_context::<UiModalContext>();
    let user_profile = use_context::<UserProfileContext>();

    let active_tab = create_signal(FriendsTab::All);
    let friends = user_profile.friends;
    let requests = create_signal::<FriendRequestsList>(FriendRequestsList::default());
    let blocked = create_signal::<Vec<BlockedUserView>>(Vec::new());

    let search_query = create_signal::<String>(String::new());
    let is_loading = create_signal::<bool>(true);

    let add_friend_input = create_signal::<String>(String::new());
    let add_friend_result = create_signal::<Option<(bool, String)>>(None);
    let action_message = create_signal::<Option<(bool, String)>>(None);

    let load_all_data = move || {
        is_loading.set(true);
        spawn_local_scoped(async move {
            if let Ok(f) = Api::get_friends().await {
                friends.set(f);
            }
            if let Ok(r) = Api::get_friend_requests().await {
                requests.set(r);
            }
            if active_tab.get_untracked() == FriendsTab::Blocked
                && let Ok(b) = Api::get_blocked_users().await
            {
                blocked.set(b);
            }
            is_loading.set(false);
        });
    };

    // Initial fetch on component mount
    load_all_data();

    let on_refresh_click = move |_| {
        load_all_data();
    };

    let on_start_chat = move |friend_name: String| {
        modal_context.selected_dm_name.set(Some(friend_name));
        modal_context.home_tab.set(HomeTab::Chats);
    };

    let on_remove_friend = move |friend_id: UserId| {
        spawn_local_scoped(async move {
            match Api::remove_friend(friend_id).await {
                Ok(()) => {
                    action_message.set(Some((
                        false,
                        "Пользователь удалён из списка друзей".to_string(),
                    )));
                    friends.update(|list| list.retain(|f| f.id != friend_id));
                }
                Err(err) => {
                    action_message.set(Some((
                        true,
                        format!("Ошибка удаления из друзей: {}", err.1),
                    )));
                }
            }
        });
    };

    let on_block_friend = move |user_id: UserId| {
        spawn_local_scoped(async move {
            match Api::block_user(user_id).await {
                Ok(()) => {
                    action_message.set(Some((
                        false,
                        "Пользователь успешно заблокирован".to_string(),
                    )));
                    friends.update(|list| list.retain(|f| f.id != user_id));
                    if let Ok(b) = Api::get_blocked_users().await {
                        blocked.set(b);
                    }
                }
                Err(err) => {
                    action_message.set(Some((
                        true,
                        format!("Ошибка блокировки пользователя: {}", err.1),
                    )));
                }
            }
        });
    };

    let on_accept_request = move |sender_id: UserId| {
        spawn_local_scoped(async move {
            match Api::accept_friend_request(sender_id).await {
                Ok(()) => {
                    action_message.set(Some((false, "Запрос в друзья принят!".to_string())));
                    if let Ok(f) = Api::get_friends().await {
                        friends.set(f);
                    }
                    if let Ok(r) = Api::get_friend_requests().await {
                        requests.set(r);
                    }
                }
                Err(err) => {
                    action_message.set(Some((true, format!("Ошибка принятия запроса: {}", err.1))));
                }
            }
        });
    };

    let on_decline_request = move |user_id: UserId| {
        spawn_local_scoped(async move {
            match Api::decline_friend_request(user_id).await {
                Ok(()) => {
                    action_message.set(Some((false, "Запрос отклонён/отменён".to_string())));
                    if let Ok(r) = Api::get_friend_requests().await {
                        requests.set(r);
                    }
                }
                Err(err) => {
                    action_message
                        .set(Some((true, format!("Ошибка обработки запроса: {}", err.1))));
                }
            }
        });
    };

    let on_unblock_user = move |user_id: UserId| {
        spawn_local_scoped(async move {
            match Api::unblock_user(user_id).await {
                Ok(()) => {
                    action_message.set(Some((
                        false,
                        "Пользователь успешно разблокирован".to_string(),
                    )));
                    blocked.update(|list| list.retain(|b| b.id != user_id));
                }
                Err(err) => {
                    action_message.set(Some((true, format!("Ошибка разблокировки: {}", err.1))));
                }
            }
        });
    };

    let on_send_request_submit = move || {
        let raw = add_friend_input.get_clone();
        let trimmed = raw.trim();
        let name = trimmed
            .strip_prefix('@')
            .unwrap_or(trimmed)
            .trim()
            .to_string();
        if name.is_empty() {
            return;
        }
        add_friend_result.set(None);
        spawn_local_scoped(async move {
            let req = SendFriendRequest {
                user_id: None,
                username: Some(name),
            };
            match Api::send_friend_request(&req).await {
                Ok(res) => {
                    add_friend_result.set(Some((false, res.message)));
                    add_friend_input.set(String::new());
                    if let Ok(r) = Api::get_friend_requests().await {
                        requests.set(r);
                    }
                    if let Ok(f) = Api::get_friends().await {
                        friends.set(f);
                    }
                }
                Err(err) => {
                    add_friend_result.set(Some((
                        true,
                        format!("Не удалось отправить запрос: {}", err.1),
                    )));
                }
            }
        });
    };

    let total_friends = create_memo(move || friends.with(|list| list.len()));
    let incoming_count = create_memo(move || requests.with(|r| r.incoming.len()));
    let total_blocked = create_memo(move || blocked.with(|list| list.len()));

    view! {
        div(class="friends-panel") {
            // ─── Top Friends Subheader & Tabs ──────────────────────────
            div(class="friends-header") {
                div(class="friends-header-title") {
                    span(class="friends-header-icon") { UsersIcon() }
                    span(class="friends-header-label") { "Друзья" }
                }

                div(class="friends-header-divider")

                div(class="friends-tabs-strip") {
                    button(
                        class=move || {
                            if active_tab.get() == FriendsTab::All {
                                "friends-tab-btn active"
                            } else {
                                "friends-tab-btn"
                            }
                        },
                        on:click=move |_| active_tab.set(FriendsTab::All),
                    ) {
                        span { "Все друзья" }
                        (if total_friends.get() > 0 {
                            let count_str = total_friends.get().to_string();
                            view! { span(class="tab-count-badge") { (count_str) } }
                        } else {
                            view! {}
                        })
                    }

                    button(
                        class=move || {
                            if active_tab.get() == FriendsTab::Requests {
                                "friends-tab-btn active"
                            } else {
                                "friends-tab-btn"
                            }
                        },
                        on:click=move |_| active_tab.set(FriendsTab::Requests),
                    ) {
                        span { "Ожидание" }
                        (if incoming_count.get() > 0 {
                            let count_str = incoming_count.get().to_string();
                            view! { span(class="tab-count-badge highlight") { (count_str) } }
                        } else {
                            view! {}
                        })
                    }

                    button(
                        class=move || {
                            if active_tab.get() == FriendsTab::Blocked {
                                "friends-tab-btn active"
                            } else {
                                "friends-tab-btn"
                            }
                        },
                        on:click=move |_| {
                            active_tab.set(FriendsTab::Blocked);
                            spawn_local_scoped(async move {
                                if let Ok(b) = Api::get_blocked_users().await {
                                    blocked.set(b);
                                }
                            });
                        },
                    ) {
                        span { "Заблокированные" }
                        (if total_blocked.get() > 0 {
                            let count_str = total_blocked.get().to_string();
                            view! { span(class="tab-count-badge") { (count_str) } }
                        } else {
                            view! {}
                        })
                    }

                    button(
                        class=move || {
                            if active_tab.get() == FriendsTab::AddFriend {
                                "friends-tab-btn add-btn active"
                            } else {
                                "friends-tab-btn add-btn"
                            }
                        },
                        on:click=move |_| active_tab.set(FriendsTab::AddFriend),
                    ) {
                        UserPlusIcon()
                        span { "Добавить в друзья" }
                    }
                }

                div(class="friends-header-actions") {
                    button(
                        class=move || {
                            if is_loading.get() {
                                "friends-refresh-btn is-loading"
                            } else {
                                "friends-refresh-btn"
                            }
                        },
                        disabled=move || is_loading.get(),
                        title="Обновить список друзей",
                        on:click=on_refresh_click,
                    ) {
                        RefreshCwIcon()
                    }
                }
            }

            // ─── Toast Action Notification ─────────────────────────────
            (if let Some((is_err, msg)) = action_message.get_clone() {
                let alert_class = if is_err {
                    "friends-action-toast error"
                } else {
                    "friends-action-toast success"
                };
                view! {
                    div(class=alert_class) {
                        span { (msg) }
                        button(
                            class="toast-close-btn",
                            on:click=move |_| action_message.set(None),
                        ) {
                            CloseSmallIcon()
                        }
                    }
                }
            } else {
                view! {}
            })

            // ─── Tab Content Views ──────────────────────────────────────
            (match active_tab.get() {
                FriendsTab::All => view_all_friends(
                    friends,
                    search_query,
                    is_loading,
                    on_start_chat,
                    on_block_friend,
                    on_remove_friend,
                    move || active_tab.set(FriendsTab::AddFriend),
                ),
                FriendsTab::Requests => view_requests(
                    requests,
                    is_loading,
                    on_accept_request,
                    on_decline_request,
                ),
                FriendsTab::Blocked => view_blocked(
                    blocked,
                    search_query,
                    is_loading,
                    on_unblock_user,
                ),
                FriendsTab::AddFriend => view_add_friend(
                    add_friend_input,
                    add_friend_result,
                    on_send_request_submit,
                ),
            })
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Sub-views
// ─────────────────────────────────────────────────────────────────────────────
