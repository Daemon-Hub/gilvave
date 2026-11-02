use sycamore::{futures::spawn_local_scoped, prelude::*};

use gilvave_core::{
    dto::users::friends::{
        BlockedUserView, FriendRequestView, FriendRequestsList, FriendView, SendFriendRequest,
    },
    ids::UserId,
};

use crate::{
    components::{
        common::{HomeTab, UiModalContext},
        ui::icons::{
            BanIcon, ChatBubbleIcon, CheckIcon, CloseSmallIcon, RefreshCwIcon, SearchIcon,
            TrashIcon, UserPlusIcon, UsersIcon,
        },
    },
    http::api::Api,
    utils::to_local_datetime,
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

    let active_tab = create_signal(FriendsTab::All);
    let friends = create_signal::<Vec<FriendView>>(Vec::new());
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
                    action_message
                        .set(Some((false, "Пользователь удалён из списка друзей".to_string())));
                    friends.update(|list| list.retain(|f| f.id != friend_id));
                }
                Err(err) => {
                    action_message
                        .set(Some((true, format!("Ошибка удаления из друзей: {}", err.1))));
                }
            }
        });
    };

    let on_block_friend = move |user_id: UserId| {
        spawn_local_scoped(async move {
            match Api::block_user(user_id).await {
                Ok(()) => {
                    action_message
                        .set(Some((false, "Пользователь успешно заблокирован".to_string())));
                    friends.update(|list| list.retain(|f| f.id != user_id));
                    if let Ok(b) = Api::get_blocked_users().await {
                        blocked.set(b);
                    }
                }
                Err(err) => {
                    action_message
                        .set(Some((true, format!("Ошибка блокировки пользователя: {}", err.1))));
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
                    action_message
                        .set(Some((true, format!("Ошибка принятия запроса: {}", err.1))));
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
                    action_message
                        .set(Some((false, "Пользователь успешно разблокирован".to_string())));
                    blocked.update(|list| list.retain(|b| b.id != user_id));
                }
                Err(err) => {
                    action_message
                        .set(Some((true, format!("Ошибка разблокировки: {}", err.1))));
                }
            }
        });
    };

    let on_send_request_submit = move || {
        let raw = add_friend_input.get_clone();
        let trimmed = raw.trim();
        let name = trimmed.strip_prefix('@').unwrap_or(trimmed).trim().to_string();
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
                    add_friend_result
                        .set(Some((true, format!("Не удалось отправить запрос: {}", err.1))));
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
                        class="friends-refresh-btn",
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

fn view_all_friends<F1, F2, F3, F4>(
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

fn view_requests<F1, F2>(
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

fn view_blocked<F>(
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

fn view_add_friend<F>(
    input_signal: Signal<String>,
    result_signal: Signal<Option<(bool, String)>>,
    on_submit: F,
) -> View
where
    F: Fn() + Copy + 'static,
{
    let handle_keydown = move |e: web_sys::KeyboardEvent| {
        if e.key() == "Enter" {
            on_submit();
        }
    };

    view! {
        div(class="friends-tab-content add-friend-pane") {
            div(class="add-friend-hero-box") {
                h3(class="add-friend-title") { "ДОБАВИТЬ В ДРУЗЬЯ" }
                p(class="add-friend-desc") {
                    "Вы можете отправить запрос дружбы, указав имя пользователя (username) нужного человека."
                }

                div(class="add-friend-input-box") {
                    input(
                        r#type="text",
                        placeholder="Введите username (например: alex_smirnov)...",
                        bind:value=input_signal,
                        on:keydown=handle_keydown,
                    )
                    button(
                        class="add-friend-submit-btn",
                        on:click=move |_| on_submit(),
                    ) {
                        span { "Отправить запрос" }
                    }
                }

                (if let Some((is_err, msg)) = result_signal.get_clone() {
                    let alert_class = if is_err {
                        "add-friend-status-box error"
                    } else {
                        "add-friend-status-box success"
                    };
                    view! {
                        div(class=alert_class) {
                            span { (msg) }
                        }
                    }
                } else {
                    view! {}
                })
            }

            div(class="add-friend-guide-card") {
                div(class="guide-header") {
                    UsersIcon()
                    h4 { "Как работает система друзей в Gilvave?" }
                }
                ul(class="guide-steps") {
                    li {
                        span(class="step-num") { "1" }
                        span(class="step-text") { "Узнайте уникальное имя пользователя (username) вашего друга." }
                    }
                    li {
                        span(class="step-num") { "2" }
                        span(class="step-text") { "Введите его в поле выше и нажмите «Отправить запрос»." }
                    }
                    li {
                        span(class="step-num") { "3" }
                        span(class="step-text") { "Когда пользователь примет ваше приглашение, он сразу появится во вкладке «Все друзья»!" }
                    }
                }
            }
        }
    }
}

fn format_friend_date(dt: time::OffsetDateTime) -> String {
    let local = to_local_datetime(dt);
    let month_str = match local.month() {
        time::Month::January => "янв",
        time::Month::February => "фев",
        time::Month::March => "мар",
        time::Month::April => "апр",
        time::Month::May => "мая",
        time::Month::June => "июн",
        time::Month::July => "июл",
        time::Month::August => "авг",
        time::Month::September => "сен",
        time::Month::October => "окт",
        time::Month::November => "ноя",
        time::Month::December => "дек",
    };
    format!("{} {} {} г.", local.day(), month_str, local.year())
}
