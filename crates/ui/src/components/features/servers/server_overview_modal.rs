use std::collections::HashSet;
use sycamore::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

use gilvave_core::ids::ServerId;

use crate::components::{
    common::{
        CreateServerContext, CreateServerOrigin, ModalView, ServerContext, UiModalContext,
        UserProfileContext,
    },
    ui::icons::{CloseSmallIcon, GlobeIcon, GridIcon, PlusIcon, SearchIcon, UserIcon},
};

use super::{open_join_modal, server_actions::select_server};

#[allow(dead_code)]
const OWNED_SERVERS_KEY: &str = "gilvave_owned_servers";

#[cfg(target_arch = "wasm32")]
pub fn get_owned_server_ids() -> HashSet<ServerId> {
    let mut set = HashSet::new();
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        if let Ok(Some(json)) = storage.get_item(OWNED_SERVERS_KEY) {
            if let Ok(ids) = serde_json::from_str::<Vec<ServerId>>(&json) {
                set.extend(ids);
            }
        }
    }
    set
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn get_owned_server_ids() -> HashSet<ServerId> {
    HashSet::new()
}

#[cfg(target_arch = "wasm32")]
pub fn add_owned_server_id(id: ServerId) {
    let mut ids = get_owned_server_ids();
    if ids.insert(id) {
        save_owned_server_ids(&ids);
        if let Some(window) = web_sys::window() {
            if let Ok(ev) = web_sys::Event::new("gilvave_owned_servers_changed") {
                let _ = window.dispatch_event(&ev);
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn add_owned_server_id(_id: ServerId) {}

#[cfg(target_arch = "wasm32")]
pub fn toggle_owned_server_id(id: ServerId) -> bool {
    let mut ids = get_owned_server_ids();
    let is_owned = if ids.contains(&id) {
        ids.remove(&id);
        false
    } else {
        ids.insert(id);
        true
    };
    save_owned_server_ids(&ids);
    if let Some(window) = web_sys::window() {
        if let Ok(ev) = web_sys::Event::new("gilvave_owned_servers_changed") {
            let _ = window.dispatch_event(&ev);
        }
    }
    is_owned
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn toggle_owned_server_id(_id: ServerId) -> bool {
    false
}

#[cfg(target_arch = "wasm32")]
fn save_owned_server_ids(ids: &HashSet<ServerId>) {
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        let vec: Vec<ServerId> = ids.iter().copied().collect();
        if let Ok(json) = serde_json::to_string(&vec) {
            let _ = storage.set_item(OWNED_SERVERS_KEY, &json);
        }
    }
}

pub use crate::components::common::OverviewTab;

fn get_server_initials(name: &str) -> String {
    let words: Vec<&str> = name.split_whitespace().collect();
    if words.len() >= 2 {
        let first = words[0].chars().next().unwrap_or('?');
        let second = words[1].chars().next().unwrap_or(' ');
        if second != ' ' {
            format!("{first}{second}").to_uppercase()
        } else {
            first.to_uppercase().to_string()
        }
    } else {
        name.chars()
            .next()
            .unwrap_or('?')
            .to_uppercase()
            .to_string()
    }
}

fn get_server_grad_class(name: &str) -> &'static str {
    let sum: usize = name.chars().map(|c| c as usize).sum();
    match (name.len() + sum) % 6 {
        0 => "overview-grad-0",
        1 => "overview-grad-1",
        2 => "overview-grad-2",
        3 => "overview-grad-3",
        4 => "overview-grad-4",
        _ => "overview-grad-5",
    }
}

#[component]
pub fn ServerOverviewModal() -> View {
    let modal_context = use_context::<UiModalContext>();
    let server_context = use_context::<ServerContext>();
    let user_profile = try_use_context::<UserProfileContext>();
    let create_server_context = try_use_context::<CreateServerContext>();

    let active_tab = modal_context.server_overview_tab;
    let search_query = create_signal(String::new());
    let owned_ids_signal = create_signal(get_owned_server_ids());

    let close = move |e: web_sys::MouseEvent| {
        e.stop_propagation();
        modal_context.is_server_overview_open.set(false);
    };

    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            let on_key = wasm_bindgen::closure::Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(
                move |e: web_sys::KeyboardEvent| {
                    if e.key() == "Escape" && modal_context.is_server_overview_open.get() {
                        modal_context.is_server_overview_open.set(false);
                    }
                },
            );
            let _ =
                window.add_event_listener_with_callback("keydown", on_key.as_ref().unchecked_ref());

            let on_owned_change = wasm_bindgen::closure::Closure::<dyn Fn(web_sys::Event)>::new(
                move |_: web_sys::Event| {
                    owned_ids_signal.set(get_owned_server_ids());
                },
            );
            let _ = window.add_event_listener_with_callback(
                "gilvave_owned_servers_changed",
                on_owned_change.as_ref().unchecked_ref(),
            );

            let window_clone = window.clone();
            on_cleanup(move || {
                let _ = window_clone.remove_event_listener_with_callback(
                    "keydown",
                    on_key.as_ref().unchecked_ref(),
                );
                let _ = window_clone.remove_event_listener_with_callback(
                    "gilvave_owned_servers_changed",
                    on_owned_change.as_ref().unchecked_ref(),
                );
            });
        }
    }

    // Auto-sync owned status from current active server if owner matches
    create_effect(move || {
        let is_owned_current = server_context.current.with(|curr_opt| {
            if let (Some(curr), Some(up)) = (curr_opt.as_ref(), user_profile) {
                up.user_id.with(|uid| *uid == Some(curr.owner_id))
            } else {
                false
            }
        });

        if is_owned_current {
            let curr_id = server_context.current.with(|c| c.as_ref().map(|s| s.id));
            if let Some(id) = curr_id {
                let mut current_set = owned_ids_signal.get_clone();
                if current_set.insert(id) {
                    #[cfg(target_arch = "wasm32")]
                    save_owned_server_ids(&current_set);
                    owned_ids_signal.set(current_set);
                }
            }
        }
    });

    let all_servers = server_context.list;

    let tab_servers = create_memo(move || {
        let tab = active_tab.get();
        let servers = all_servers.get_clone();
        servers
            .into_iter()
            .filter(|s| match tab {
                OverviewTab::Saved => !s.is_owner,
                OverviewTab::My => s.is_owner,
            })
            .collect::<Vec<_>>()
    });

    let saved_count =
        create_memo(move || all_servers.with(|list| list.iter().filter(|s| !s.is_owner).count()));

    let my_count =
        create_memo(move || all_servers.with(|list| list.iter().filter(|s| s.is_owner).count()));

    let filtered_servers = create_memo(move || {
        let q = search_query.get_clone().trim().to_lowercase();
        let servers = tab_servers.get_clone();
        if q.is_empty() {
            servers
        } else {
            servers
                .into_iter()
                .filter(|s| s.name.to_lowercase().contains(&q))
                .collect()
        }
    });

    let total_all_servers_count = create_memo(move || all_servers.with(|list| list.len()));

    let count_text = create_memo(move || {
        let total = total_all_servers_count.get();
        let q = search_query.get_clone();
        if q.trim().is_empty() {
            format!("{total}")
        } else {
            let filtered = filtered_servers.with(|l| l.len());
            format!("{filtered} из {total}")
        }
    });

    // 1. Top header button: opens general selection modal (ModalView::Home)
    let open_general_add_modal = move |e: web_sys::MouseEvent| {
        e.stop_propagation();
        if let Some(ctx) = create_server_context {
            ctx.from_dashboard.set(false);
            ctx.origin.set(CreateServerOrigin::OverviewHome);
            ctx.server_name.set(String::new());
            ctx.modal_view.set(ModalView::Home);
            ctx.is_modal_open.set(true);
        }
        modal_context.is_server_overview_open.set(false);
    };

    // 2. Direct create action for "Мои серверы" tab
    let open_create_modal = move |e: web_sys::MouseEvent| {
        e.stop_propagation();
        if let Some(ctx) = create_server_context {
            ctx.from_dashboard.set(false);
            ctx.origin.set(CreateServerOrigin::OverviewTab);
            ctx.server_name.set(String::new());
            ctx.modal_view.set(ModalView::Create);
            ctx.is_modal_open.set(true);
        }
        modal_context.server_overview_tab.set(OverviewTab::My);
        modal_context.is_server_overview_open.set(false);
    };

    // 3. Direct join action for "Сохранённые серверы" tab
    let open_join_modal_action = move |e: web_sys::MouseEvent| {
        e.stop_propagation();
        if let Some(ctx) = create_server_context {
            ctx.from_dashboard.set(false);
            ctx.origin.set(CreateServerOrigin::OverviewTab);
            open_join_modal(ctx);
        }
        modal_context.server_overview_tab.set(OverviewTab::Saved);
        modal_context.is_server_overview_open.set(false);
    };

    // 4. Contextual add tile inside grid: checks active_tab BEFORE unmounting
    let open_contextual_add_modal = move |e: web_sys::MouseEvent| {
        e.stop_propagation();
        let current_tab = active_tab.get();
        if let Some(ctx) = create_server_context {
            ctx.from_dashboard.set(false);
            ctx.origin.set(CreateServerOrigin::OverviewTab);
            match current_tab {
                OverviewTab::My => {
                    ctx.server_name.set(String::new());
                    ctx.modal_view.set(ModalView::Create);
                    ctx.is_modal_open.set(true);
                }
                OverviewTab::Saved => {
                    open_join_modal(ctx);
                }
            }
        }
        modal_context.is_server_overview_open.set(false);
    };

    view! {
        div(class="server-overview-overlay", on:click=close) {
            div(
                class="server-overview-modal",
                on:click=move |e: web_sys::MouseEvent| e.stop_propagation(),
            ) {
                // Header
                div(class="server-overview-header") {
                    div(class="server-overview-header-title") {
                        span(class="server-overview-header-icon") { GridIcon() }
                        h2 { "Серверы" }
                        span(class="server-overview-badge") { (count_text.get_clone()) }
                    }

                    div(class="server-overview-search") {
                        span(class="server-overview-search-icon") { SearchIcon() }
                        input(
                            r#type="text",
                            class="server-overview-search-input",
                            placeholder="Поиск серверов...",
                            bind:value=search_query,
                        )
                        (if !search_query.with(|q| q.is_empty()) {
                            view! {
                                button(
                                    class="server-overview-search-clear",
                                    on:click=move |_| search_query.set(String::new()),
                                    title="Очистить",
                                ) {
                                    CloseSmallIcon()
                                }
                            }
                        } else {
                            view! {}
                        })
                    }

                    div(class="server-overview-header-actions") {
                        button(
                            class="server-overview-add-btn",
                            on:click=open_general_add_modal,
                            title="Добавить или создать сервер",
                        ) {
                            PlusIcon()
                            span { "Добавить" }
                        }
                        button(
                            class="server-overview-close-btn",
                            on:click=close,
                            title="Закрыть (Esc)",
                        ) {
                            CloseSmallIcon()
                        }
                    }
                }

                // 2 Tabs Navigation Bar: Сохраненные серверы & Мои серверы
                div(class="server-overview-tabs-bar") {
                    button(
                        class=format!(
                            "server-overview-tab-btn{}",
                            if active_tab.get() == OverviewTab::Saved { " active" } else { "" }
                        ),
                        on:click=move |_| active_tab.set(OverviewTab::Saved),
                    ) {
                        GlobeIcon()
                        span { "Сохранённые серверы" }
                        span(class="server-overview-tab-badge") { (format!("{}", saved_count.get())) }
                    }
                    button(
                        class=format!(
                            "server-overview-tab-btn{}",
                            if active_tab.get() == OverviewTab::My { " active" } else { "" }
                        ),
                        on:click=move |_| active_tab.set(OverviewTab::My),
                    ) {
                        UserIcon()
                        span { "Мои серверы" }
                        span(class="server-overview-tab-badge") { (format!("{}", my_count.get())) }
                    }
                }

                // Body
                div(class="server-overview-body") {
                    (if filtered_servers.with(|l| l.is_empty()) {
                        let q = search_query.get_clone();
                        let tab = active_tab.get();
                        if q.trim().is_empty() {
                            match tab {
                                OverviewTab::My => view! {
                                    div(class="server-overview-empty") {
                                        div(class="server-overview-empty-icon") { UserIcon() }
                                        h3 { "У вас пока нет своих серверов" }
                                        p { "Создайте собственный сервер, чтобы общаться с друзьями и настраивать свои каналы." }
                                        button(
                                            class="server-overview-empty-btn",
                                            on:click=open_create_modal,
                                        ) {
                                            PlusIcon()
                                            span { "Создать сервер" }
                                        }
                                    }
                                },
                                OverviewTab::Saved => view! {
                                    div(class="server-overview-empty") {
                                        div(class="server-overview-empty-icon") { GlobeIcon() }
                                        h3 { "У вас пока нет сохранённых серверов" }
                                        p { "Присоединяйтесь к публичным серверам или примите приглашение от сообщества." }
                                        button(
                                            class="server-overview-empty-btn",
                                            on:click=open_join_modal_action,
                                        ) {
                                            PlusIcon()
                                            span { "Найти серверы" }
                                        }
                                    }
                                },
                            }
                        } else {
                            view! {
                                div(class="server-overview-empty") {
                                    div(class="server-overview-empty-icon") { SearchIcon() }
                                    h3 { "Ничего не найдено" }
                                    p { (format!("По запросу «{}» серверов не найдено.", q)) }
                                    button(
                                        class="server-overview-empty-btn",
                                        on:click=move |_| search_query.set(String::new()),
                                    ) {
                                        "Сбросить поиск"
                                    }
                                }
                            }
                        }
                    } else {
                        view! {
                            div(class="server-overview-grid") {
                                Indexed(
                                    list=filtered_servers,
                                    view=move |server| {
                                        let s_id = server.id;
                                        let s_name = server.name;
                                        let s_alt = s_name.clone();
                                        let s_icon = server.icon_url;
                                        let is_active = server_context.current.with(|c| c.as_ref().map(|s| s.id) == Some(s_id));
                                        let initials = get_server_initials(&s_name);
                                        let has_icon = !s_icon.is_empty();

                                        // Item 1 fix: NEVER apply gradient class if custom icon image exists
                                        let icon_box_class = if has_icon {
                                            "server-overview-icon-box has-icon".to_string()
                                        } else {
                                            let grad_cls = get_server_grad_class(&s_name);
                                            format!("server-overview-icon-box {grad_cls}")
                                        };

                                        let on_item_click = move |_| {
                                            select_server(s_id);
                                            modal_context.selected_dm_name.set(None);
                                            modal_context.is_server_overview_open.set(false);
                                        };

                                        // Selected server highlighted with standard outline on active card
                                        let card_class = format!("server-overview-item{}", if is_active { " active" } else { "" });

                                        view! {
                                            div(
                                                class=card_class,
                                                on:click=on_item_click,
                                                data-server-id=s_id.to_string(),
                                            ) {
                                                div(class=icon_box_class) {
                                                    (if has_icon {
                                                        view! {
                                                            img(src=s_icon, alt=s_alt, class="server-overview-icon-img")
                                                        }
                                                    } else {
                                                        view! {
                                                            span(class="server-overview-initials") { (initials) }
                                                        }
                                                    })
                                                }
                                                span(class="server-overview-name") { (s_name) }
                                            }
                                        }
                                    },
                                )

                                // Add Server Tile at the end
                                div(
                                    class="server-overview-item add-tile",
                                    on:click=open_contextual_add_modal,
                                ) {
                                    div(class="server-overview-icon-box add-box") {
                                        PlusIcon()
                                    }
                                    span(class="server-overview-name") { "Добавить" }
                                }
                            }
                        }
                    })
                }
            }
        }
    }
}
