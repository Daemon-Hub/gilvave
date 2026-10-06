use gilvave_core::dto::{
    channel::{ChannelType, ChannelView},
    message::MessageView,
    server::{MemberView, Server, ServerSmallPart},
};
use sycamore::{futures::spawn_local_scoped, prelude::*};

use crate::{
    components::{
        common::{
            ChannelContext, ScreenWrapper, ServerContext, UiModalContext, UserProfileContext,
            classes,
        },
        features::{
            channels::{
                channel_panel::ChannelPanel, create_channel_modal::CreateChannelModal,
                voice_channel_area::VoiceChannelArea,
            },
            chat::messages_area::{ExpandedMessageEditorModal, MessagesArea},
            home::{home_dashboard::HomeDashboard, home_nav_panel::HomeNavPanel},
            members::members_panel::MembersPanel,
            profile::{ProfileSettingsModal, ThemeCatalogModal, ThemeJsonEditorModal},
            servers::{
                server_settings_modal::ServerSettingsModal, server_sidebar::ServerSidebar,
                SidebarControls,
            },
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
        username: create_signal(String::default()),
        avatar: create_signal(String::default()),
        banner: create_signal(String::default()),
        bio: create_signal(String::default()),
        is_muted: create_signal(false),
        is_deafened: create_signal(false),
    };
    provide_context(user_profile);

    let initial_custom_themes = crate::components::common::CustomTheme::load_all();
    crate::components::common::CustomTheme::sync_preview_styles(&initial_custom_themes);

    let ui_modal_context = UiModalContext {
        is_server_settings_open: create_signal(false),
        is_create_channel_open: create_signal(false),
        create_channel_type: create_signal(gilvave_core::dto::channel::ChannelType::TEXT),
        is_profile_settings_open: create_signal(false),
        selected_dm_name: create_signal(None),
        home_tab: create_signal(crate::components::common::HomeTab::Chats),
        draft_message: create_signal(String::new()),
        is_message_editor_open: create_signal(false),
        app_theme: create_signal(crate::components::common::AppTheme::load_saved()),
        custom_themes: create_signal(initial_custom_themes),
        is_windowed_mode: create_signal(crate::components::common::load_windowed_mode()),
        is_theme_catalog_open: create_signal(false),
        is_theme_json_editor_open: create_signal(false),
        theme_json_editor_content: create_signal(String::new()),
    };
    provide_context(ui_modal_context);

    create_effect(move || {
        let theme = ui_modal_context.app_theme.get_clone();
        let customs = ui_modal_context.custom_themes.get_clone();
        theme.apply(&customs);
    });

    create_effect(move || {
        let windowed = ui_modal_context.is_windowed_mode.get();
        crate::components::common::save_windowed_mode(windowed);
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
                    user_profile.username.set(user.username);
                    user_profile.avatar.set(user.avatar);
                }
            });

            if !ws_started.get() {
                ws_started.set(true);
                spawn_local_scoped(async move {
                    let _ = WsService::listen_web_socket().await;
                });
            }
            spawn_local_scoped(async move {
                let servers = Api::get_user_servers().await.unwrap_or_default();
                server_context.list.set(servers);
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
    let home_pill_class = move || if is_home_active.get() { "home-pill active" } else { "home-pill" };
    let home_btn_class = move || if is_home_active.get() { "home-avatar-btn active" } else { "home-avatar-btn" };

    let discord_content_class = create_memo(move || {
        if server_context.current.with(|c| c.is_some()) {
            "discord-content in-server"
        } else {
            match ui_modal_context.home_tab.get() {
                crate::components::common::HomeTab::Chats => "discord-content in-home show-chats",
                crate::components::common::HomeTab::Dashboard => "discord-content in-home show-dashboard",
            }
        }
    });

    let is_server_settings_open = ui_modal_context.is_server_settings_open;
    let is_create_channel_open = ui_modal_context.is_create_channel_open;
    let is_profile_settings_open = ui_modal_context.is_profile_settings_open;
    let is_theme_catalog_open = ui_modal_context.is_theme_catalog_open;
    let is_theme_json_editor_open = ui_modal_context.is_theme_json_editor_open;
    let is_message_editor_open = ui_modal_context.is_message_editor_open;
    let is_fullbleed: MaybeDyn<bool> = (move || {
        crate::components::common::is_mobile_device() || !ui_modal_context.is_windowed_mode.get()
    }).into();

    let active_channel_type = create_memo(move || {
        channel_context.current.with(|c| c.as_ref().map(|ch| ch.r#type))
    });

    view! {
        div(
            class=classes(vec![
                "discord-container".into(),
                "home-panel-container".into(),
                ("active", is_home_screen.clone()).into(),
                ("fullbleed", is_fullbleed.clone()).into(),
            ]),
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

                    div(class="separator")
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
    }
}

fn header_server_view(server_signal: Signal<Option<Server>>, modal_context: UiModalContext) -> View {
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
