use gilvave_core::{
    dto::{
        channel::ChannelView,
        message::MessageView,
        server::{MemberView, Server, ServerSmallPart},
    },
    ids::ServerId,
};
use sycamore::prelude::*;

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
}

#[derive(Clone, Copy)]
pub struct UserProfileContext {
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
    pub create_channel_type: Signal<gilvave_core::dto::channel::ChannelType>,
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
}

