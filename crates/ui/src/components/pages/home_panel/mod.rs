//! Authenticated workspace container (sidebar, channels, chat, members, modals, search bar).

mod gestures;
mod header;
mod peel_corner;

pub(crate) use header::header_server_view;
pub(crate) use peel_corner::SidebarCornerPeel;

use gestures::{setup_android_back_button, use_swipe_gestures};
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
            EmojiPack, HomeTab, ModalView, OverviewTab, ScreenWrapper, ServerContext,
            UiModalContext, UserProfileContext, classes, is_mobile_device, load_close_to_tray,
            load_show_greeting, load_show_quick_actions, load_windowed_mode, save_close_to_tray,
            save_show_greeting, save_show_quick_actions, save_windowed_mode,
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
        ui::icons::{CloseSmallIcon, HashIcon, HomeIcon},
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
        friends: create_signal(Vec::new()),
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
        emoji_pack: create_signal(EmojiPack::load_saved()),
        is_windowed_mode: create_signal(load_windowed_mode()),
        is_close_to_tray: create_signal(load_close_to_tray()),
        is_show_greeting: create_signal(load_show_greeting()),
        is_show_quick_actions: create_signal(load_show_quick_actions()),
        is_theme_catalog_open: create_signal(false),
        is_theme_json_editor_open: create_signal(false),
        theme_json_editor_content: create_signal(String::new()),
        replying_to: create_signal(None),
        editing_message: create_signal(None),
        forwarding_message: create_signal(None),
        is_forward_modal_open: create_signal(false),
        is_server_overview_open: create_signal(false),
        server_overview_tab: create_signal(OverviewTab::Saved),
        toast_message: create_signal(None),
        lightbox_image: create_signal(None),
        active_fullscreen_video: create_signal(None),
        pending_attachments: create_signal(Vec::new()),
        is_uploading_attachment: create_signal(false),
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
        let pack = ui_modal_context.emoji_pack.get();
        pack.save();
    });

    create_effect(move || {
        let windowed = ui_modal_context.is_windowed_mode.get();
        save_windowed_mode(windowed);
    });

    create_effect(move || {
        let show_greeting = ui_modal_context.is_show_greeting.get();
        save_show_greeting(show_greeting);
    });

    create_effect(move || {
        let show_quick = ui_modal_context.is_show_quick_actions.get();
        save_show_quick_actions(show_quick);
    });

    create_effect(move || {
        let close_to_tray = ui_modal_context.is_close_to_tray.get();
        save_close_to_tray(close_to_tray);
        if crate::utils::is_tauri() {
            wasm_bindgen_futures::spawn_local(async move {
                let _ = crate::utils::invoke_command(
                    gilvave_core::dto::command::CommandArgs::SetCloseToTray {
                        enabled: close_to_tray,
                    }
                    .to_json(),
                )
                .await;
            });
        }
    });

    let server_context = ServerContext {
        current: create_signal::<Option<Server>>(None),
        list: create_signal::<Vec<ServerSmallPart>>(vec![]),
        members: create_signal::<Vec<MemberView>>(vec![]),
        online_users: create_signal::<std::collections::HashSet<gilvave_core::ids::UserId>>(
            std::collections::HashSet::new(),
        ),
    };
    provide_context(server_context);

    let channel_context = ChannelContext {
        text: create_signal::<Vec<ChannelView>>(vec![]),
        voice: create_signal::<Vec<ChannelView>>(vec![]),
        current: create_signal::<Option<ChannelView>>(None),
        messages: create_signal::<Vec<MessageView>>(vec![]),
    };
    provide_context(channel_context);

    let mut ws_started = false;
    create_effect(move || {
        if screen_wrapper.is_home() && !ws_started {
            ws_started = true;

            // 1. Instant 0ms cache restore from localStorage & SQLite
            if let Some(storage) = web_sys::window()
                .and_then(|w| w.local_storage().ok())
                .flatten()
            {
                if let Ok(Some(name)) = storage.get_item("gilvave_cached_username") {
                    user_profile.username.set(name);
                }
                if let Ok(Some(avatar)) = storage.get_item("gilvave_cached_avatar") {
                    user_profile.avatar.set(avatar);
                }
                if let Ok(Some(bio)) = storage.get_item("gilvave_cached_bio") {
                    user_profile.bio.set(bio);
                }
                if let Ok(Some(id_str)) = storage.get_item("gilvave_cached_user_id")
                    && let Ok(uuid) = gilvave_core::uuid::Uuid::parse_str(&id_str)
                {
                    user_profile
                        .user_id
                        .set(Some(gilvave_core::ids::UserId::from(uuid)));
                }
            }

            spawn_local_scoped(async move {
                let cached = CacheService::get_cached_servers().await;
                if !cached.is_empty() {
                    server_context.list.set(cached);
                }
            });

            // 2. Start WebSocket background connection loop
            spawn_local_scoped(async move {
                let _ = WsService::listen_web_socket().await;
            });

            // 3. Initial network fetch & retry loop for delayed connections or empty cache
            spawn_local_scoped(async move {
                let servers_synced = sync_servers_to_context(server_context).await;
                let _ = sync_profile_to_context(user_profile).await;

                if !servers_synced && server_context.list.with(|l| l.is_empty()) {
                    for delay_sec in [1, 2, 3, 5] {
                        gloo_timers::future::sleep(core::time::Duration::from_secs(delay_sec))
                            .await;
                        if !server_context.list.with(|l| l.is_empty()) {
                            break;
                        }
                        web_sys::console::log_1(
                            &"[HOME_PANEL] Retrying servers sync due to delayed initial connection...".into(),
                        );
                        if sync_servers_to_context(server_context).await {
                            let _ = sync_profile_to_context(user_profile).await;
                            break;
                        }
                    }
                }
            });

            // 4. Reactive WebSocket gateway event listener: re-sync servers and profile immediately on connect / reconnect
            let mut gw_rx = crate::gateway::handler::subscribe_gateway_events();
            spawn_local_scoped(async move {
                use futures_util::StreamExt;
                while let Some(event) = gw_rx.next().await {
                    match event {
                        crate::gateway::handler::GatewayEvent::Connected => {
                            web_sys::console::log_1(
                                &"[HOME_PANEL] Gateway connected -> syncing servers & profile"
                                    .into(),
                            );
                            sync_servers_to_context(server_context).await;
                            sync_profile_to_context(user_profile).await;
                            if let Some(srv) = server_context.current.get_clone() {
                                let _ = WsService::server_online(srv.id).await;
                            }
                        }
                        crate::gateway::handler::GatewayEvent::ServerOnline {
                            server_id,
                            online_users,
                        } => {
                            let is_current = server_context
                                .current
                                .with(|c| c.as_ref().map(|s| s.id) == Some(server_id));
                            if is_current {
                                server_context
                                    .online_users
                                    .set(online_users.into_iter().collect());
                            }
                        }
                        crate::gateway::handler::GatewayEvent::UserOnline { user_id } => {
                            server_context.online_users.update(|set| {
                                set.insert(user_id);
                            });
                        }
                        crate::gateway::handler::GatewayEvent::UserOffline { user_id } => {
                            server_context.online_users.update(|set| {
                                set.remove(&user_id);
                            });
                        }
                        _ => {}
                    }
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
        context.online_users.set(std::collections::HashSet::new());
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

    // Android hardware back button handler
    setup_android_back_button(ui_modal_context, channel_context, create_server_context);

    // Mobile swipe gestures
    let (on_touch_start, on_touch_end) =
        use_swipe_gestures(channel_context, ui_modal_context, server_context);

    view! {
        div(
            class=classes(vec![
                "discord-container".into(),
                "home-panel-container".into(),
                ("active", is_home_screen.clone()).into(),
                ("fullbleed", is_fullbleed.clone()).into(),
            ]),
            on:contextmenu=on_context_menu,
            on:touchstart=on_touch_start,
            on:touchend=on_touch_end,
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

                    SidebarCornerPeel()
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
        (if let Some((is_err, msg)) = ui_modal_context.toast_message.get_clone() {
            let alert_class = if is_err {
                "global-toast-notification error"
            } else {
                "global-toast-notification success"
            };
            let toast_sig = ui_modal_context.toast_message;
            view! {
                div(class=alert_class) {
                    span(class="toast-text") { (msg) }
                    button(
                        class="toast-close-btn",
                        on:click=move |_| toast_sig.set(None),
                    ) {
                        CloseSmallIcon()
                    }
                }
            }
        } else {
            view! {}
        })
        (if let Some(active) = ui_modal_context.active_fullscreen_video.get_clone() {
            view! {
                crate::components::features::chat::attachments::FullscreenVideoModal(active=active)
            }
        } else {
            view! {}
        })
        (if let Some((src, alt)) = ui_modal_context.lightbox_image.get_clone() {
            let lightbox_sig = ui_modal_context.lightbox_image;
            let is_vid = crate::components::common::media::is_video_source(&src, &alt);
            view! {
                div(
                    class="chat-image-lightbox-overlay",
                    on:click=move |_| lightbox_sig.set(None),
                ) {
                    div(class="lightbox-content", on:click=move |e: web_sys::MouseEvent| e.stop_propagation()) {
                        button(
                            class="lightbox-close-btn",
                            title="Закрыть (Esc)",
                            on:click=move |_| lightbox_sig.set(None),
                        ) {
                            CloseSmallIcon()
                        }
                        (if is_vid {
                            view! {
                                video(
                                    class="lightbox-full-video",
                                    src=src,
                                    title=alt,
                                    controls=true,
                                    autoplay=true,
                                    playsinline=true,
                                )
                            }
                        } else {
                            view! {
                                img(
                                    class="lightbox-full-img",
                                    src=src,
                                    alt=alt,
                                )
                            }
                        })
                    }
                }
            }
        } else {
            view! {}
        })
    }
}

async fn sync_servers_to_context(server_context: ServerContext) -> bool {
    match Api::get_user_servers().await {
        Ok(servers) => {
            server_context.list.set(servers.clone());
            CacheService::sync_cached_servers(servers).await;
            true
        }
        Err(err) => {
            web_sys::console::warn_1(
                &format!("[HOME_PANEL] get_user_servers failed: {err:?}").into(),
            );
            false
        }
    }
}

async fn sync_profile_to_context(user_profile: UserProfileContext) -> bool {
    if let Ok(user) = Api::get_profile().await {
        let avatar = user.avatar().to_string();
        let bio = user.bio().to_string();
        user_profile.user_id.set(Some(user.id));
        user_profile.username.set(user.username.clone());
        user_profile.avatar.set(avatar.clone());
        user_profile.bio.set(bio.clone());

        if let Some(storage) = web_sys::window()
            .and_then(|w| w.local_storage().ok())
            .flatten()
        {
            let _ = storage.set_item("gilvave_cached_user_id", &user.id.to_string());
            let _ = storage.set_item("gilvave_cached_username", &user.username);
            let _ = storage.set_item("gilvave_cached_avatar", &avatar);
            let _ = storage.set_item("gilvave_cached_bio", &bio);
        }

        if let Ok(friends) = Api::get_friends().await {
            user_profile.friends.set(friends);
        }
        true
    } else {
        false
    }
}
