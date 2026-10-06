use sycamore::prelude::*;

use crate::components::{
    common::{UiModalContext, UserProfileContext},
    ui::icons::{GearIcon, HeadphonesIcon, HeadphonesOffIcon, MicIcon, MicOffIcon},
};

#[component]
pub fn SidebarControls() -> View {
    let user_profile = use_context::<UserProfileContext>();
    let modal_context = use_context::<UiModalContext>();

    let on_toggle_mic = move |_| {
        let new_muted = !user_profile.is_muted.get();
        user_profile.is_muted.set(new_muted);
        // TODO [WS]: Send ServerSend::VoiceStateUpdate if currently connected to a voice channel
    };

    let on_toggle_deafen = move |_| {
        let new_deafened = !user_profile.is_deafened.get();
        user_profile.is_deafened.set(new_deafened);
        // TODO [WS]: Send ServerSend::VoiceStateUpdate if currently connected to a voice channel
    };

    let on_open_settings = move |_| {
        modal_context.is_profile_settings_open.set(true);
    };

    let mic_btn_class = move || {
        if user_profile.is_muted.get() {
            "sidebar-ctrl-btn mic muted"
        } else {
            "sidebar-ctrl-btn mic"
        }
    };

    let mic_btn_tip = move || {
        if user_profile.is_muted.get() {
            "Включить микрофон"
        } else {
            "Отключить микрофон"
        }
    };

    let deafen_btn_class = move || {
        if user_profile.is_deafened.get() {
            "sidebar-ctrl-btn deafen deafened"
        } else {
            "sidebar-ctrl-btn deafen"
        }
    };

    let deafen_btn_tip = move || {
        if user_profile.is_deafened.get() {
            "Включить звук"
        } else {
            "Заглушить звук"
        }
    };

    view! {
        div(class="sidebar-controls") {
            button(
                class=mic_btn_class,
                on:click=on_toggle_mic,
                data-tip=mic_btn_tip,
                title=mic_btn_tip,
            ) {
                (if user_profile.is_muted.get() {
                    view! { MicOffIcon() }
                } else {
                    view! { MicIcon() }
                })
            }

            button(
                class=deafen_btn_class,
                on:click=on_toggle_deafen,
                data-tip=deafen_btn_tip,
                title=deafen_btn_tip,
            ) {
                (if user_profile.is_deafened.get() {
                    view! { HeadphonesOffIcon() }
                } else {
                    view! { HeadphonesIcon() }
                })
            }

            button(
                class="sidebar-ctrl-btn settings",
                on:click=on_open_settings,
                data-tip="Настройки пользователя",
                title="Настройки пользователя",
            ) {
                GearIcon()
            }
        }
    }
}
