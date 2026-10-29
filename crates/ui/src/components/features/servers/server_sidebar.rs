use gilvave_core::{
    dto::servers::{Server, ServerCreateInfo},
    ids::{ServerId, UserId},
};
use sycamore::prelude::*;

use crate::components::common::classes;
use crate::components::{
    common::{CreateServerContext, CreateServerOrigin, ModalView, ServerContext, UiModalContext},
    ui::icons::{ArrowLeftIcon, GlobeIcon, LockIcon, ServerIcon},
};

use super::{
    join_server_modal::JoinServerModal,
    server_actions::{create_server, select_server},
};

pub fn fallback_hardcoded_servers() -> Vec<Server> {
    use gilvave_core::uuid::Uuid;
    let epoch = time::OffsetDateTime::UNIX_EPOCH;
    vec![
        Server {
            id: ServerId(Uuid::from_u128(0x550e8400_e29b_41d4_a716_446655440001)),
            name: "Rust Developers".to_string(),
            icon_url: String::new(),
            created_at: epoch,
            description: "Сообщество разработчиков на Rust. Обсуждаем код, делимся проектами и помогаем новичкам.".to_string(),
            members_count: 1247,
            cover: "https://images.unsplash.com/photo-1515879218367-8466d910auj4?w=600".to_string(),
            is_public: true,
            owner_id: UserId::default(),
        },
        Server {
            id: ServerId(Uuid::from_u128(0x550e8400_e29b_41d4_a716_446655440002)),
            name: "Гейм-дев".to_string(),
            icon_url: String::new(),
            created_at: epoch,
            description: "Разработка игр на всех движках. Unity, Unreal, Godot — всё обсуждаем здесь.".to_string(),
            members_count: 834,
            cover: "https://images.unsplash.com/photo-1511512578047-dfb367046420?w=600".to_string(),
            is_public: true,
            owner_id: UserId::default(),
        },
        Server {
            id: ServerId(Uuid::from_u128(0x550e8400_e29b_41d4_a716_446655440003)),
            name: "Музыка".to_string(),
            icon_url: String::new(),
            created_at: epoch,
            description: "Делимся музыкой, обсуждаем альбомы и находим единомышленников по вкусам.".to_string(),
            members_count: 2103,
            cover: "https://images.unsplash.com/photo-1511379938547-c1f69419868d?w=600".to_string(),
            is_public: true,
            owner_id: UserId::default(),
        },
        Server {
            id: ServerId(Uuid::from_u128(0x550e8400_e29b_41d4_a716_446655440004)),
            name: "Аниме клуб".to_string(),
            icon_url: String::new(),
            created_at: epoch,
            description: "Обсуждаем аниме, мангу и всё связанное с японской культурой. Новости, обзоры, рекомендации.".to_string(),
            members_count: 3562,
            cover: "https://images.unsplash.com/photo-1578632767115-351597cf2477?w=600".to_string(),
            is_public: true,
            owner_id: UserId::default(),
        },
        Server {
            id: ServerId(Uuid::from_u128(0x550e8400_e29b_41d4_a716_446655440005)),
            name: "Memes & Chill".to_string(),
            icon_url: String::new(),
            created_at: epoch,
            description: "Лучшие мемы, тёплая атмосфера и хорошее настроение каждый день.".to_string(),
            members_count: 5891,
            cover: "https://images.unsplash.com/photo-1533738363-b7f9aef128ce?w=600".to_string(),
            is_public: true,
            owner_id: UserId::default(),
        },
    ]
}

pub fn open_join_modal(context: CreateServerContext) {
    context.modal_view.set(ModalView::Join);
    context.is_modal_open.set(true);
    context.expanded_id.set(None);
    context.public_servers_page.set(1);
    context.has_more_public_servers.set(false);
    context.public_servers.set(vec![]);
}

#[component(inline_props)]
pub fn ServerSidebar() -> View {
    let context = use_context::<CreateServerContext>();
    let server_context = use_context::<ServerContext>();

    let on_plus_click = move |e: web_sys::MouseEvent| {
        e.stop_propagation();
        let context = use_context::<CreateServerContext>();
        context.from_dashboard.set(false);
        context.origin.set(CreateServerOrigin::Sidebar);
        context.modal_view.set(ModalView::Home);
        context.is_modal_open.set(true);
    };

    let close = move |e: web_sys::MouseEvent| {
        e.stop_propagation();
        let context = use_context::<CreateServerContext>();
        let origin = context.origin.get();
        context.is_modal_open.set(false);
        context.modal_view.set(ModalView::Home);
        context.server_name.set(String::new());
        context.public_servers.set(vec![]);
        if origin == CreateServerOrigin::OverviewHome
            && let Some(m_ctx) = try_use_context::<UiModalContext>()
        {
            m_ctx.is_server_overview_open.set(true);
        }
    };

    let open_create = move |e: web_sys::MouseEvent| {
        e.stop_propagation();
        let context = use_context::<CreateServerContext>();
        context.modal_view.set(ModalView::Create);
    };

    let open_join = move |e: web_sys::MouseEvent| {
        e.stop_propagation();
        let context = use_context::<CreateServerContext>();
        context.from_dashboard.set(false);
        open_join_modal(context);
    };

    let handle_create = move |e: web_sys::MouseEvent| {
        e.stop_propagation();
        let context = use_context::<CreateServerContext>();
        if context.server_name.with(|v| v.trim().is_empty()) {
            return;
        }
        let info = ServerCreateInfo {
            name: context.server_name.get_clone(),
            is_public: context.is_public.get(),
        };
        create_server(server_context.list, info);
        context.is_modal_open.set(false);
        context.modal_view.set(ModalView::Home);
        context.server_name.set(String::new());
    };

    let handle_back = move |e: web_sys::MouseEvent| {
        e.stop_propagation();
        let context = use_context::<CreateServerContext>();
        match context.origin.get() {
            CreateServerOrigin::Sidebar | CreateServerOrigin::OverviewHome => {
                context.modal_view.set(ModalView::Home);
            }
            CreateServerOrigin::Dashboard => {
                context.is_modal_open.set(false);
                context.modal_view.set(ModalView::Home);
                context.public_servers.set(vec![]);
            }
            CreateServerOrigin::OverviewTab => {
                context.is_modal_open.set(false);
                context.modal_view.set(ModalView::Home);
                context.public_servers.set(vec![]);
                if let Some(m_ctx) = try_use_context::<UiModalContext>() {
                    m_ctx.is_server_overview_open.set(true);
                }
            }
        }
    };

    let is_home_visible = create_memo(move || {
        let context = use_context::<CreateServerContext>();
        context.is_modal_open.get() && context.modal_view.get() == ModalView::Home
    });
    let is_create_visible = create_memo(move || {
        let context = use_context::<CreateServerContext>();
        context.is_modal_open.get() && context.modal_view.get() == ModalView::Create
    });
    let is_join_visible = create_memo(move || {
        let context = use_context::<CreateServerContext>();
        context.is_modal_open.get() && context.modal_view.get() == ModalView::Join
    });

    view! {
        Indexed(
            list=server_context.list,
            view=move |server| {
                let id = server.id;
                let tip = server.name.clone();
                let name = create_signal(server.name);
                let icon_url = create_signal(server.icon_url);

                let is_active_memo = create_memo(move || {
                    server_context.current.with(|c| c.as_ref().map(|s| s.id) == Some(id))
                });
                let is_active_dyn: MaybeDyn<bool> = is_active_memo.into();

                let pill_class = classes(vec![
                    "server-icon-pill".into(),
                    ("active", is_active_dyn.clone()).into(),
                ]);

                let s_id_str = id.to_string();
                let s_name_str = tip.clone();
                view! {
                    div(
                        class="server-icon-wrapper",
                        data-server-id=s_id_str,
                        data-server-name=s_name_str,
                    ) {
                        div(class=pill_class)
                        ServerIcon(
                            server_name=name,
                            icon_url=icon_url,
                            is_active=is_active_dyn,
                            data-tip=tip,
                            on:click=move |_| {
                                let c_ctx = use_context::<CreateServerContext>();
                                c_ctx.is_modal_open.set(false);
                                if let Some(m_ctx) = try_use_context::<crate::components::common::UiModalContext>() {
                                    m_ctx.selected_dm_name.set(None);
                                }
                                select_server(id);
                            },
                        )
                    }
                }
            },
        )
        ServerIcon(
            server_name=create_signal("+".to_string()),
            icon_url=create_signal(String::default()),
            data-tip="Добавить сервер",
            on:click=on_plus_click,
        )

        (if is_home_visible.get() {
            view! {
                div(class="server-modal-overlay", on:click=close) {
                    div(
                        class="server-modal",
                        on:click=move |e: web_sys::MouseEvent| e.stop_propagation(),
                    ) {
                        div(class="server-modal-header") {
                            span { "Серверы" }
                        }
                        div(class="server-modal-body") {
                            div(class="server-modal-card create", on:click=open_create) {
                                div(class="server-modal-icon create") {
                                    svg(
                                        xmlns="http://www.w3.org/2000/svg",
                                        width="32",
                                        height="32",
                                        viewBox="0 0 24 24",
                                        fill="none",
                                        stroke="currentColor",
                                        stroke-width="2",
                                        stroke-linecap="round",
                                        stroke-linejoin="round",
                                    ) {
                                        line(x1="12", y1="5", x2="12", y2="19")
                                        line(x1="5", y1="12", x2="19", y2="12")
                                    }
                                }
                                span(class="server-modal-label") { "Создать сервер" }
                            }
                            div(class="server-modal-card join", on:click=open_join) {
                                div(class="server-modal-icon join") {
                                    svg(
                                        xmlns="http://www.w3.org/2000/svg",
                                        width="32",
                                        height="32",
                                        viewBox="0 0 24 24",
                                        fill="none",
                                        stroke="currentColor",
                                        stroke-width="2",
                                        stroke-linecap="round",
                                        stroke-linejoin="round",
                                    ) {
                                        path(d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4")
                                        polyline(points="10 17 15 12 10 7")
                                        line(x1="15", y1="12", x2="3", y2="12")
                                    }
                                }
                                span(class="server-modal-label") { "Присоединиться" }
                            }
                        }
                    }
                }
            }
        } else {
            view! {}
        })

        (if is_create_visible.get() {
            view! {
                div(class="server-modal-overlay", on:click=close) {
                    div(
                        class="server-modal create-form",
                        on:click=move |e: web_sys::MouseEvent| e.stop_propagation(),
                    ) {
                        div(class="server-modal-header") {
                            span { "Создать сервер" }
                        }
                        div(class="create-form-body") {
                            div(class="input-group") {
                                label { "Название сервера" }
                                input(
                                    r#type="text",
                                    placeholder="Мой сервер",
                                    bind:value=context.server_name,
                                )
                            }
                            div(class="create-form-tabs") {
                                div(class="toggle-wrapper") {
                                    div(class="toggle-tabs") {
                                        div(
                                            class=classes(vec![
                                                "toggle-tab".into(),
                                                ("active", { !context.is_public.get() }.into()).into(),
                                            ]),
                                            on:click=move |_| context.is_public.set(false),
                                        ) {
                                            LockIcon()
                                            span { "Приватный" }
                                        }
                                        div(
                                            class=classes(vec![
                                                "toggle-tab".into(),
                                                ("active", { context.is_public.get() }.into()).into(),
                                            ]),
                                            on:click=move |_| context.is_public.set(true),
                                        ) {
                                            GlobeIcon()
                                            span { "Публичный" }
                                        }
                                        div(
                                            class=classes(vec![
                                                "floating-bg".into(),
                                                ("public", { context.is_public.get() }.into()).into(),
                                            ]),
                                        )
                                    }
                                }
                                span(class="checkbox-hint") {
                                    (if context.is_public.get() {
                                        view! {
                                            GlobeIcon()
                                            span { "Публичные серверы видны всем пользователям" }
                                        }
                                    } else {
                                        view! {
                                            LockIcon()
                                            span { "Только приглашённые пользователи" }
                                        }
                                    })
                                }
                            }
                            div(class="create-form-actions") {
                                button(
                                    class="back-btn create-back-btn",
                                    on:click=handle_back,
                                    title="Назад",
                                ) {
                                    ArrowLeftIcon()
                                    span { "Назад" }
                                }
                                button(
                                    class="submit-btn create-submit",
                                    on:click=handle_create,
                                ) { "Создать" }
                            }
                        }
                    }
                }
            }
        } else {
            view! {}
        })

        (if is_join_visible.get() {
            view! {
                JoinServerModal(
                    on_back=handle_back,
                    on_close=close,
                )
            }
        } else {
            view! {}
        })
    }
}
