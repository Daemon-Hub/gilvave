use sycamore::prelude::*;

use crate::components::{
    common::{HomeTab, UiModalContext},
    ui::icons::{ChatBubbleIcon, HomeIcon, PlusIcon, UsersIcon, ZapIcon},
};

#[derive(Clone, PartialEq)]
pub struct DmContact {
    pub id: &'static str,
    pub name: &'static str,
    pub avatar: &'static str,
    pub status: &'static str, // "online", "idle", "dnd", "offline"
    pub activity: &'static str,
    pub unread: usize,
}

#[derive(Clone, PartialEq)]
pub struct GroupChat {
    pub id: &'static str,
    pub name: &'static str,
    pub icon_letter: &'static str,
    pub members_count: usize,
}

#[component]
pub fn HomeNavPanel() -> View {
    let modal_context = use_context::<UiModalContext>();

    let dms = vec![
        DmContact {
            id: "dm-1",
            name: "Алексей Смирнов",
            avatar: "https://images.unsplash.com/photo-1534528741775-53994a69daeb?w=100",
            status: "online",
            activity: "Кодит на Rust 🦀",
            unread: 2,
        },
        DmContact {
            id: "dm-2",
            name: "Елена Васильева",
            avatar: "https://images.unsplash.com/photo-1494790108377-be9c29b29330?w=100",
            status: "online",
            activity: "В сети",
            unread: 0,
        },
        DmContact {
            id: "dm-3",
            name: "Gilvave Bot",
            avatar: "https://images.unsplash.com/photo-1618005182384-a83a8bd57fbe?w=100",
            status: "online",
            activity: "Бот • v0.2.0",
            unread: 0,
        },
        DmContact {
            id: "dm-4",
            name: "Анна Морозова",
            avatar: "https://images.unsplash.com/photo-1438761681033-6461ffad8d80?w=100",
            status: "idle",
            activity: "Не на месте",
            unread: 0,
        },
        DmContact {
            id: "dm-5",
            name: "Михаил Ковалёв",
            avatar: "https://images.unsplash.com/photo-1507003211169-0a1dd7228f2d?w=100",
            status: "offline",
            activity: "Был 2 часа назад",
            unread: 0,
        },
    ];

    let groups = vec![
        GroupChat {
            id: "grp-1",
            name: "🚀 Команда Gilvave",
            icon_letter: "G",
            members_count: 4,
        },
        GroupChat {
            id: "grp-2",
            name: "🎮 Пятничный кооп",
            icon_letter: "C",
            members_count: 6,
        },
    ];

    let dms_signal = create_signal(dms);
    let groups_signal = create_signal(groups);

    let on_friends_click = move |_| {
        modal_context.selected_dm_name.set(None);
        modal_context.home_tab.set(HomeTab::Dashboard);
    };

    view! {
        div(class="channel-panel home-nav-panel") {
            div(class="channel-list") {
                div(class="home-mobile-nav-toggle") {
                    button(
                        class=if modal_context.home_tab.get() == HomeTab::Chats {
                            "mobile-nav-btn active"
                        } else {
                            "mobile-nav-btn"
                        },
                        on:click=move |_| modal_context.home_tab.set(HomeTab::Chats),
                    ) {
                        ChatBubbleIcon()
                        span { "Чаты" }
                    }
                    button(
                        class=if modal_context.home_tab.get() == HomeTab::Dashboard {
                            "mobile-nav-btn active"
                        } else {
                            "mobile-nav-btn"
                        },
                        on:click=move |_| modal_context.home_tab.set(HomeTab::Dashboard),
                    ) {
                        HomeIcon()
                        span { "Главная" }
                    }
                }

                div(class="home-quick-nav") {
                    div(
                        class=if modal_context.selected_dm_name.with(|d| d.is_none()) && modal_context.home_tab.get() == HomeTab::Dashboard {
                            "home-nav-item active"
                        } else {
                            "home-nav-item"
                        },
                        on:click=on_friends_click,
                    ) {
                        span(class="nav-icon") { UsersIcon() }
                        span(class="nav-title") { "Друзья" }
                    }

                    div(class="home-nav-item") {
                        span(class="nav-icon") { ZapIcon() }
                        span(class="nav-title") { "Входящие" }
                    }
                }

                div(class="channel-header") {
                    span { "Личные сообщения" }
                    button(
                        class="channel-add-btn",
                        title="Начать переписку",
                        on:click=move |_| {
                            modal_context.selected_dm_name.set(Some("Новое сообщение".to_string()));
                        },
                    ) { PlusIcon() }
                }

                Indexed(
                    list=dms_signal,
                    view=move |dm| {
                        let dm_name = dm.name;
                        let is_active = create_memo(move || {
                            modal_context.selected_dm_name.with(|s| s.as_deref() == Some(dm_name))
                        });

                        let item_class = move || {
                            if is_active.get() {
                                "dm-item active"
                            } else {
                                "dm-item"
                            }
                        };

                        let on_select = move |_| {
                            modal_context.selected_dm_name.set(Some(dm_name.to_string()));
                        };

                        let status_class = format!("status-dot {}", dm.status);

                        view! {
                            div(class=item_class(), on:click=on_select) {
                                div(class="dm-avatar") {
                                    img(src=dm.avatar, alt="")
                                    div(class=status_class)
                                }
                                div(class="dm-info") {
                                    span(class="dm-name") { (dm.name) }
                                    span(class="dm-activity") { (dm.activity) }
                                }
                                (if dm.unread > 0 {
                                    let unread_str = dm.unread.to_string();
                                    view! {
                                        span(class="unread-badge") { (unread_str) }
                                    }
                                } else {
                                    view! {}
                                })
                            }
                        }
                    },
                )

                div(class="channel-header") {
                    span { "Группы" }
                }

                Indexed(
                    list=groups_signal,
                    view=move |grp| {
                        let grp_name = grp.name;
                        let on_select_group = move |_| {
                            modal_context.selected_dm_name.set(Some(grp_name.to_string()));
                        };

                        let members_str = format!("{} участников", grp.members_count);

                        view! {
                            div(class="dm-item group-item", on:click=on_select_group) {
                                div(class="group-icon-circle") {
                                    span { (grp.icon_letter) }
                                }
                                div(class="dm-info") {
                                    span(class="dm-name") { (grp.name) }
                                    span(class="dm-activity") { (members_str) }
                                }
                            }
                        }
                    },
                )
            }
        }
    }
}
