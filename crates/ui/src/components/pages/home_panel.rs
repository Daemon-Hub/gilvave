use sycamore::{futures::spawn_local_scoped, prelude::*};

use gilvave_core::dto::{
    messages::MessageView,
    servers::{
        MemberView, Server, ServerSmallPart,
        channels::{ChannelType, ChannelView},
    },
};

use crate::{
    cache::CacheService,
    components::{
        common::{
            AppTheme, ChannelContext, CreateServerContext, CreateServerOrigin, CustomTheme,
            HomeTab, ModalView, OverviewTab, ScreenWrapper, ServerContext, UiModalContext,
            UserProfileContext, classes, is_mobile_device, load_windowed_mode, save_windowed_mode,
        },
        features::{
            channels::{
                channel_panel::ChannelPanel, create_channel_modal::CreateChannelModal,
                voice_channel_area::VoiceChannelArea,
            },
            chat::{
                forward_modal::ForwardMessageModal,
                messages_area::{ExpandedMessageEditorModal, MessagesArea},
            },
            home::{home_dashboard::HomeDashboard, home_nav_panel::HomeNavPanel},
            members::members_panel::MembersPanel,
            profile::{ProfileSettingsModal, ThemeCatalogModal, ThemeJsonEditorModal},
            servers::{ServerOverviewModal, ServerSettingsModal, ServerSidebar, SidebarControls},
        },
        ui::icons::{GearIcon, HashIcon, HomeIcon},
    },
    gateway::service::WsService,
    http::api::Api,
};

#[component]
pub fn HomePanel() -> View {
    let screen_wrapper = use_context::<ScreenWrapper>();
    let is_home_screen: MaybeDyn<bool> = (move || screen_wrapper.is_home()).into();

    let user_profile = UserProfileContext {
        user_id: create_signal(None),
        username: create_signal(String::default()),
        avatar: create_signal(String::default()),
        banner: create_signal(String::default()),
        bio: create_signal(String::default()),
        is_muted: create_signal(false),
        is_deafened: create_signal(false),
    };
    provide_context(user_profile);

    let initial_custom_themes = CustomTheme::load_all();
    CustomTheme::sync_preview_styles(&initial_custom_themes);

    let ui_modal_context = UiModalContext {
        is_server_settings_open: create_signal(false),
        is_create_channel_open: create_signal(false),
        create_channel_type: create_signal(ChannelType::TEXT),
        is_profile_settings_open: create_signal(false),
        selected_dm_name: create_signal(None),
        home_tab: create_signal(HomeTab::Chats),
        draft_message: create_signal(String::new()),
        is_message_editor_open: create_signal(false),
        app_theme: create_signal(AppTheme::load_saved()),
        custom_themes: create_signal(initial_custom_themes),
        is_windowed_mode: create_signal(load_windowed_mode()),
        is_theme_catalog_open: create_signal(false),
        is_theme_json_editor_open: create_signal(false),
        theme_json_editor_content: create_signal(String::new()),
        replying_to: create_signal(None),
        editing_message: create_signal(None),
        forwarding_message: create_signal(None),
        is_forward_modal_open: create_signal(false),
        is_server_overview_open: create_signal(false),
        server_overview_tab: create_signal(OverviewTab::Saved),
    };
    provide_context(ui_modal_context);

    let create_server_context = CreateServerContext {
        is_modal_open: create_signal(false),
        modal_view: create_signal(ModalView::Home),
        server_name: create_signal(String::new()),
        is_public: create_signal(true),
        public_servers: create_signal::<Vec<Server>>(vec![]),
        expanded_id: create_signal::<Option<gilvave_core::ids::ServerId>>(None),
        from_dashboard: create_signal(false),
        origin: create_signal(CreateServerOrigin::Sidebar),
        public_servers_page: create_signal(1),
        has_more_public_servers: create_signal(false),
    };
    provide_context(create_server_context);

    create_effect(move || {
        let theme = ui_modal_context.app_theme.get_clone();
        let customs = ui_modal_context.custom_themes.get_clone();
        theme.apply(&customs);
    });

    create_effect(move || {
        let windowed = ui_modal_context.is_windowed_mode.get();
        save_windowed_mode(windowed);
    });

    let server_context = ServerContext {
        current: create_signal::<Option<Server>>(None),
        list: create_signal::<Vec<ServerSmallPart>>(vec![]),
        members: create_signal::<Vec<MemberView>>(vec![]),
    };
    provide_context(server_context);

    let channel_context = ChannelContext {
        text: create_signal::<Vec<ChannelView>>(vec![]),
        voice: create_signal::<Vec<ChannelView>>(vec![]),
        current: create_signal::<Option<ChannelView>>(None),
        messages: create_signal::<Vec<MessageView>>(vec![]),
    };
    provide_context(channel_context);

    let ws_started = create_signal(false);
    create_effect(move || {
        if screen_wrapper.is_home() {
            spawn_local_scoped(async move {
                if let Ok(user) = Api::get_profile().await {
                    let avatar = user.avatar().to_string();
                    let bio = user.bio().to_string();
                    user_profile.user_id.set(Some(user.id));
                    user_profile.username.set(user.username);
                    user_profile.avatar.set(avatar);
                    user_profile.bio.set(bio);
                }
            });

            if !ws_started.get() {
                ws_started.set(true);
                spawn_local_scoped(async move {
                    let _ = WsService::listen_web_socket().await;
                });
            }
            spawn_local_scoped(async move {
                let cached = CacheService::get_cached_servers().await;
                if !cached.is_empty() {
                    server_context.list.set(cached);
                }
                if let Ok(servers) = Api::get_user_servers().await {
                    server_context.list.set(servers.clone());
                    CacheService::sync_cached_servers(servers).await;
                }
            });
        }
    });

    let handle_home_click = move |_| {
        let ch_context = use_context::<ChannelContext>();
        let active_channel_id = ch_context.current.with(|c| c.as_ref().map(|ch| ch.id));
        ch_context.current.set(None);
        if let Some(channel_id) = active_channel_id {
            spawn_local_scoped(async move {
                let _ = WsService::left_channel(channel_id).await;
            });
        }
        let context = use_context::<ServerContext>();
        context.current.set(None);
        let m_ctx = use_context::<UiModalContext>();
        m_ctx.selected_dm_name.set(None);
    };

    let is_home_active = create_memo(move || server_context.current.with(|c| c.is_none()));
    let home_pill_class = move || {
        if is_home_active.get() {
            "home-pill active"
        } else {
            "home-pill"
        }
    };
    let home_btn_class = move || {
        if is_home_active.get() {
            "home-avatar-btn active"
        } else {
            "home-avatar-btn"
        }
    };

    let discord_content_class = create_memo(move || {
        if server_context.current.with(|c| c.is_some()) {
            "discord-content in-server"
        } else {
            match ui_modal_context.home_tab.get() {
                HomeTab::Chats => "discord-content in-home show-chats",
                HomeTab::Dashboard => "discord-content in-home show-dashboard",
            }
        }
    });

    let is_server_settings_open = ui_modal_context.is_server_settings_open;
    let is_create_channel_open = ui_modal_context.is_create_channel_open;
    let is_profile_settings_open = ui_modal_context.is_profile_settings_open;
    let is_theme_catalog_open = ui_modal_context.is_theme_catalog_open;
    let is_theme_json_editor_open = ui_modal_context.is_theme_json_editor_open;
    let is_message_editor_open = ui_modal_context.is_message_editor_open;
    let is_forward_modal_open = ui_modal_context.is_forward_modal_open;
    let is_server_overview_open = ui_modal_context.is_server_overview_open;
    let is_fullbleed: MaybeDyn<bool> =
        (move || is_mobile_device() || !ui_modal_context.is_windowed_mode.get()).into();

    let active_channel_type = create_memo(move || {
        channel_context
            .current
            .with(|c| c.as_ref().map(|ch| ch.r#type))
    });

    let context_menu_ctx = use_context::<crate::components::ui::ContextMenuContext>();
    let on_context_menu = move |event: web_sys::MouseEvent| {
        crate::components::ui::context_menu::handle_global_context_menu(
            &event,
            &context_menu_ctx,
            Some(&ui_modal_context),
            Some(&channel_context),
            Some(&server_context),
            Some(&user_profile),
        );
    };

    let is_peel_hovered = create_signal(false);
    let peel_cut_line_d = move || {
        if is_peel_hovered.get() {
            "M 14.80 3.00 L 45.00 3.00 L 45.00 4.60 L 14.80 4.60 A 0.80 0.80 0 0 1 14.80 3.00 Z"
        } else {
            "M 14.80 3.00 L 58.40 3.00 L 58.40 4.60 L 14.80 4.60 A 0.80 0.80 0 0 1 14.80 3.00 Z"
        }
    };
    let peel_void_d = move || {
        if is_peel_hovered.get() {
            "M 45.00 3.65 C 59.58 3.35 72.00 13.35 72.00 25.50 L 72.00 3.00 L 45.00 3.00 Z"
        } else {
            "M 58.40 3.65 C 65.74 3.35 72.00 8.15 72.00 14.20 L 72.00 3.00 L 58.40 3.00 Z"
        }
    };
    let peel_shadow_d = move || {
        if is_peel_hovered.get() {
            "M 45.00 3.65 L 72.00 25.50 C 64.90 23.93 54.37 29.97 43.92 27.50 C 40.87 18.73 47.08 9.72 45.00 3.65 Z"
        } else {
            "M 58.40 3.65 L 72.00 14.20 C 68.42 13.42 63.12 16.75 57.86 15.52 C 56.32 11.15 59.45 6.67 58.40 3.65 Z"
        }
    };
    let peel_flap_d = move || {
        if is_peel_hovered.get() {
            "M 45.00 3.65 C 59.58 3.35 72.00 13.35 72.00 25.50 C 66.60 22.35 58.50 26.15 50.94 23.57 Q 48.78 23.00 48.21 20.84 C 45.00 14.90 48.78 8.15 45.00 3.65 Z"
        } else {
            "M 58.40 3.65 C 65.74 3.35 72.00 8.15 72.00 14.20 C 69.28 12.63 65.20 14.85 61.44 13.58 Q 60.30 13.28 60.00 12.14 C 58.40 9.25 60.30 5.89 58.40 3.65 Z"
        }
    };
    let peel_bevel_d = move || {
        if is_peel_hovered.get() {
            "M 45.00 3.65 C 48.78 8.15 45.00 14.90 48.21 20.84 Q 48.78 23.00 50.94 23.57 C 58.50 26.15 66.60 22.35 72.00 25.50"
        } else {
            "M 58.40 3.65 C 60.30 5.89 58.40 9.25 60.00 12.14 Q 60.30 13.28 61.44 13.58 C 65.20 14.85 69.28 12.63 72.00 14.20"
        }
    };
    let peel_crease_d = move || {
        if is_peel_hovered.get() {
            "M 45.00 3.65 C 59.58 3.35 72.00 13.35 72.00 25.50"
        } else {
            "M 58.40 3.65 C 65.74 3.35 72.00 8.15 72.00 14.20"
        }
    };
    let peel_hit_d = move || {
        if is_peel_hovered.get() {
            "M 44.00 0.00 L 72.00 0.00 L 72.00 27.00 L 44.00 27.00 Z"
        } else {
            "M 54.00 0.00 L 72.00 0.00 L 72.00 16.00 L 54.00 16.00 Z"
        }
    };

    view! {
        div(
            class=classes(vec![
                "discord-container".into(),
                "home-panel-container".into(),
                ("active", is_home_screen.clone()).into(),
                ("fullbleed", is_fullbleed.clone()).into(),
            ]),
            on:contextmenu=on_context_menu,
        ) {
            div(class="discord-sidebar") {
                div(class="discord-sidebar-top") {
                    div(class="home-button-wrapper") {
                        div(class=home_pill_class)
                        button(
                            class=home_btn_class,
                            on:click=handle_home_click,
                            title="Главная (Личные сообщения и друзья)",
                        ) {
                            div(class="home-avatar-inner") {
                                (if user_profile.avatar.with(|a| !a.is_empty()) {
                                    let av = user_profile.avatar.get_clone();
                                    view! { img(src=av, alt="") }
                                } else {
                                    let initial = user_profile.username.with(|u| u.chars().next().unwrap_or('?').to_uppercase().to_string());
                                    view! { span { (initial) } }
                                })
                            }
                            div(class="home-badge") {
                                HomeIcon()
                            }
                        }
                    }

                    // Interactive Top-Right Paper Corner Peel & Cut Line
                    // Horizontal razor slit cut line (like the separator) meeting the curled corner
                    // ONLY the curled corner in the top-right triggers unrolling and click
                    div(
                        class="sidebar-corner-peel",
                    ) {
                        svg(
                            class="corner-peel-svg",
                            viewBox="0 0 72 32",
                            fill="none",
                            xmlns="http://www.w3.org/2000/svg",
                        ) {
                            defs {
                                // 1. Brand separator gradient (continuous across slit and void)
                                linearGradient(id="cornerUnderlayerGrad", x1="14.0", y1="3.0", x2="72.0", y2="3.0", gradientUnits="userSpaceOnUse") {
                                    stop(offset="0%", stop-color="var(--color-server-separator-stop-1, #22d3ee)")
                                    stop(offset="50%", stop-color="var(--color-server-separator-stop-2, #818cf8)")
                                    stop(offset="100%", stop-color="var(--color-server-separator-stop-3, #f59e0b)")
                                }

                                // 2. Flap underside: darker shade of the theme separator gradient
                                linearGradient(id="gilvaveFlapGrad", x1="14.0", y1="3.0", x2="72.0", y2="3.0", gradientUnits="userSpaceOnUse") {
                                    stop(offset="0%", stop-color="var(--color-server-flap-stop-1, #0e7490)")
                                    stop(offset="50%", stop-color="var(--color-server-flap-stop-2, #3730a3)")
                                    stop(offset="100%", stop-color="var(--color-server-flap-stop-3, #78350f)")
                                }

                                // 3. Soft Gaussian blur for drop shadow
                                filter(id="cornerShadowBlur", x="-40%", y="-40%", width="180%", height="180%") {
                                    feGaussianBlur(stdDeviation="1.0")
                                }
                            }

                            // 1. Horizontal razor slit cut line (non-interactive visual line)
                            path(
                                class="corner-cut-line",
                                d=peel_cut_line_d,
                                fill="url(#cornerUnderlayerGrad)",
                            )

                            // 2. Interactive Corner Hit Group: ONLY the top-right corner responds to mouse events!
                            g(
                                class="peel-corner-hit",
                                on:mouseenter=move |_| is_peel_hovered.set(true),
                                on:mouseleave=move |_| is_peel_hovered.set(false),
                                on:click=move |_| ui_modal_context.is_server_overview_open.set(true),
                            ) {
                                title { "Все серверы" }
                                // Transparent dynamic hit pad matching the corner geometry
                                path(
                                    class="peel-hit-pad",
                                    d=peel_hit_d,
                                    fill="transparent",
                                )

                                // Exposed luminous underlayer void behind peeled corner
                                path(
                                    class="corner-void",
                                    d=peel_void_d,
                                    fill="url(#cornerUnderlayerGrad)",
                                )

                                // Soft drop shadow cast onto the sheet
                                path(
                                    class="corner-shadow",
                                    d=peel_shadow_d,
                                    fill="rgba(0, 0, 0, 0.75)",
                                    filter="url(#cornerShadowBlur)",
                                )

                                // The 3D curled paper corner (pointing LEFT-DOWN towards servers!)
                                path(
                                    class="corner-flap",
                                    d=peel_flap_d,
                                    fill="url(#gilvaveFlapGrad)",
                                    stroke="var(--color-server-flap-stroke, #3b1124)",
                                    stroke-width="0.3",
                                )

                                // Paper thickness bevel highlight along free cut edges
                                path(
                                    class="corner-bevel",
                                    d=peel_bevel_d,
                                    stroke="rgba(255, 255, 255, 0.65)",
                                    stroke-width="0.5",
                                    fill="none",
                                    stroke-linecap="round",
                                )

                                // Roll Arch crease line along the fold
                                path(
                                    class="corner-crease",
                                    d=peel_crease_d,
                                    stroke="rgba(129, 140, 248, 0.4)",
                                    stroke-width="0.4",
                                    fill="none",
                                    stroke-linecap="round",
                                )
                            }
                        }
                    }
                }

                div(class="discord-sidebar-servers") {
                    ServerSidebar()
                }

                div(class="discord-sidebar-bottom") {
                    div(class="separator")
                    SidebarControls()
                }
            }

            div(class="discord-main") {
                div(class="discord-header") {
                    (header_server_view(server_context.current, ui_modal_context))

                    div(class="search-bar") {
                        span(class="search-bar-icon") {
                            svg(viewBox="0 0 24 24") {
                                circle(cx="11", cy="11", r="7")
                                line(x1="20", y1="20", x2="16.35", y2="16.35")
                            }
                        }
                        input(
                            r#type="text",
                            class="search-bar-input",
                            placeholder="Поиск...",
                        )
                        span(class="search-bar-shortcut") { "Ctrl K" }
                    }
                }

                div(class=discord_content_class) {
                    (if server_context.current.with(|c| c.is_some()) {
                        view! {
                            ChannelPanel()

                            (match active_channel_type.get() {
                                Some(ChannelType::TEXT) => view! { MessagesArea() },
                                Some(ChannelType::VOICE) => view! { VoiceChannelArea() },
                                None => view! {
                                    div(class="no-channel-selected") {
                                        div(class="no-channel-content") {
                                            span(class="no-channel-icon") { HashIcon() }
                                            h3 { "Выберите канал" }
                                            p { "Выберите текстовый или голосовой канал в списке слева, чтобы начать общение." }
                                        }
                                    }
                                },
                            })
                        }
                    } else {
                        view! {
                            HomeNavPanel()
                            HomeDashboard()
                        }
                    })
                }
            }

            (if server_context.current.with(|c| c.is_some()) {
                MembersPanel()
            } else {
                view!{}
            })
        }

        (if is_server_settings_open.get() {
            view! { ServerSettingsModal() }
        } else {
            view! {}
        })
        (if is_create_channel_open.get() {
            view! { CreateChannelModal() }
        } else {
            view! {}
        })
        (if is_profile_settings_open.get() {
            view! { ProfileSettingsModal() }
        } else {
            view! {}
        })
        (if is_theme_catalog_open.get() {
            view! { ThemeCatalogModal() }
        } else {
            view! {}
        })
        (if is_theme_json_editor_open.get() {
            view! { ThemeJsonEditorModal() }
        } else {
            view! {}
        })
        (if is_message_editor_open.get() {
            view! { ExpandedMessageEditorModal() }
        } else {
            view! {}
        })
        (if is_forward_modal_open.get() {
            view! { ForwardMessageModal() }
        } else {
            view! {}
        })
        (if is_server_overview_open.get() {
            view! { ServerOverviewModal() }
        } else {
            view! {}
        })
    }
}

fn header_server_view(
    server_signal: Signal<Option<Server>>,
    modal_context: UiModalContext,
) -> View {
    let header_data = server_signal.with(|server_opt| {
        server_opt.as_ref().map(|server| {
            let cover = (!server.cover.is_empty()).then(|| server.cover.clone());
            let icon = (!server.icon_url.is_empty()).then(|| server.icon_url.clone());
            let initial = server.icon_url.is_empty().then(|| {
                server
                    .name
                    .chars()
                    .next()
                    .unwrap_or('?')
                    .to_uppercase()
                    .to_string()
            });
            let members_str = format!("{} участников", server.members_count);
            let s_name = server.name.clone();
            (cover, icon, initial, s_name, members_str)
        })
    });

    if let Some((cover, icon, initial, s_name, members_str)) = header_data {
        let cover_view = if let Some(c) = cover {
            view! {
                div(class="header-server-cover") {
                    img(src=c, alt="")
                }
            }
        } else {
            view! {}
        };

        let icon_view = if let Some(i) = icon {
            view! { img(src=i, alt="") }
        } else {
            let init_str = initial.unwrap_or_default();
            view! { span { (init_str) } }
        };

        view! {
            div(class="header-server-info") {
                (cover_view)
                div(class="header-server-icon") {
                    (icon_view)
                }
                div(class="header-server-text") {
                    span(class="header-server-name") { (s_name) }
                    span(class="header-server-members") { (members_str) }
                }
                button(
                    class="header-settings-btn",
                    on:click=move |_| modal_context.is_server_settings_open.set(true),
                    title="Настройки сервера",
                ) {
                    GearIcon()
                }
            }
        }
    } else {
        view! {
            div(class="header-server-info") {
                div(class="header-server-icon home") {
                    HomeIcon()
                }
                div(class="header-server-text") {
                    span(class="header-server-name") { "Личные сообщения" }
                }
            }
        }
    }
}
