use sycamore::prelude::*;

use gilvave_core::{
    dto::{
        messages::MessageView,
        servers::channels::{ChannelType, ChannelView},
        servers::{MemberView, Server, ServerSmallPart},
    },
    ids::{ServerId, UserId},
};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum NetworkStatus {
    #[default]
    Online,
    Offline,
    Reconnecting,
    Connected,
}

#[derive(Clone, Copy)]
pub struct ConnectivityContext {
    pub status: Signal<NetworkStatus>,
}

#[derive(Clone, Copy)]
pub struct ChannelContext {
    pub current: Signal<Option<ChannelView>>,
    pub text: Signal<Vec<ChannelView>>,
    pub voice: Signal<Vec<ChannelView>>,
    pub messages: Signal<Vec<MessageView>>,
}

#[derive(Clone, Copy)]
pub struct ServerContext {
    pub current: Signal<Option<Server>>,
    pub list: Signal<Vec<ServerSmallPart>>,
    pub members: Signal<Vec<MemberView>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum CreateServerOrigin {
    #[default]
    Sidebar,
    Dashboard,
    OverviewTab,
    OverviewHome,
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum OverviewTab {
    #[default]
    Saved,
    My,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ModalView {
    Home,
    Create,
    Join,
}

#[derive(Clone, Copy)]
pub struct CreateServerContext {
    pub is_modal_open: Signal<bool>,
    pub modal_view: Signal<ModalView>,
    pub server_name: Signal<String>,
    pub is_public: Signal<bool>,
    pub public_servers: Signal<Vec<Server>>,
    pub expanded_id: Signal<Option<ServerId>>,
    pub from_dashboard: Signal<bool>,
    pub origin: Signal<CreateServerOrigin>,
    pub public_servers_page: Signal<u32>,
    pub has_more_public_servers: Signal<bool>,
}

#[derive(Clone, Copy)]
pub struct UserProfileContext {
    pub user_id: Signal<Option<UserId>>,
    pub username: Signal<String>,
    pub avatar: Signal<String>,
    pub banner: Signal<String>,
    pub bio: Signal<String>,
    pub is_muted: Signal<bool>,
    pub is_deafened: Signal<bool>,
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum HomeTab {
    #[default]
    Chats,
    Dashboard,
}

use super::theme::{AppTheme, CustomTheme};

#[derive(Clone, Copy)]
pub struct UiModalContext {
    pub is_server_settings_open: Signal<bool>,
    pub is_create_channel_open: Signal<bool>,
    pub create_channel_type: Signal<ChannelType>,
    pub is_profile_settings_open: Signal<bool>,
    pub selected_dm_name: Signal<Option<String>>,
    pub home_tab: Signal<HomeTab>,
    pub draft_message: Signal<String>,
    pub is_message_editor_open: Signal<bool>,
    pub app_theme: Signal<AppTheme>,
    pub custom_themes: Signal<Vec<CustomTheme>>,
    pub is_windowed_mode: Signal<bool>,
    pub is_theme_catalog_open: Signal<bool>,
    pub is_theme_json_editor_open: Signal<bool>,
    pub theme_json_editor_content: Signal<String>,
    pub replying_to: Signal<Option<MessageView>>,
    pub editing_message: Signal<Option<MessageView>>,
    pub forwarding_message: Signal<Option<MessageView>>,
    pub is_forward_modal_open: Signal<bool>,
    pub is_server_overview_open: Signal<bool>,
    pub server_overview_tab: Signal<OverviewTab>,
}
