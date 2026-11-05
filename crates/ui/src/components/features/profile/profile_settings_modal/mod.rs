//! Profile settings modal container, tabs navigation, and settings state.

pub mod appearance_tab;
pub mod profile_tab;
pub mod security_tab;
pub mod system_tab;

use sycamore::{futures::spawn_local_scoped, prelude::*};

use self::appearance_tab::AppearanceTab;
use self::profile_tab::ProfileTab;
use self::security_tab::SecurityTab;
use self::system_tab::SystemTab;
use crate::components::common::{
    ActiveScreen, ScreenWrapper, UiModalContext, UserProfileContext, classes,
};
use crate::components::features::profile::sessions_tab::SessionsTab;
use crate::components::ui::icons::{
    CloseSmallIcon, GearIcon, LockIcon, LogOutIcon, MonitorIcon, PaletteIcon, UserIcon,
};
use crate::http::api::Api;
use gilvave_core::dto::users::profile::UpdateProfileInfo;

#[derive(Clone, Copy, PartialEq)]
enum SettingsTab {
    Profile,
    Appearance,
    System,
    Security,
    Sessions,
}

#[component]
pub fn ProfileSettingsModal() -> View {
    let screen_wrapper = use_context::<ScreenWrapper>();
    let user_profile = use_context::<UserProfileContext>();
    let modal_context = use_context::<UiModalContext>();

    let active_tab = create_signal(SettingsTab::Profile);
    let show_logout_confirm = create_signal(false);

    let name_input = create_signal(String::new());
    let avatar_input = create_signal(String::new());
    let banner_input = create_signal(String::new());
    let bio_input = create_signal(String::new());

    let old_password = create_signal(String::new());
    let new_password = create_signal(String::new());
    let confirm_password = create_signal(String::new());
    let password_msg = create_signal(String::new());

    let theme_status_msg = create_signal(String::new());
    let theme_status_is_error = create_signal(false);

    // Sync from user_profile when modal opens
    create_effect(move || {
        if modal_context.is_profile_settings_open.get() {
            name_input.set(user_profile.username.get_clone());
            avatar_input.set(user_profile.avatar.get_clone());
            banner_input.set(user_profile.banner.get_clone());
            bio_input.set(user_profile.bio.get_clone());
            password_msg.set(String::new());
            theme_status_msg.set(String::new());
        }
    });

    let close = move || {
        modal_context.is_profile_settings_open.set(false);
    };

    let handle_save = move || {
        let new_username = name_input.get_clone();
        let new_avatar = avatar_input.get_clone();
        let new_banner = banner_input.get_clone();
        let new_bio = bio_input.get_clone();

        let req = UpdateProfileInfo {
            username: if !new_username.trim().is_empty() {
                Some(new_username.clone())
            } else {
                None
            },
            name: None,
            bio: Some(new_bio.clone()),
            avatar: Some(new_avatar.clone()),
            banner: Some(new_banner.clone()),
        };

        if !new_username.trim().is_empty() {
            user_profile.username.set(new_username.clone());
        }
        user_profile.avatar.set(new_avatar.clone());
        user_profile.banner.set(new_banner.clone());
        user_profile.bio.set(new_bio.clone());
        modal_context.is_profile_settings_open.set(false);

        let modal_ctx_clone = modal_context;
        let user_profile_ctx = user_profile;
        spawn_local_scoped(async move {
            match Api::update_profile(&req).await {
                Ok(user_view) => {
                    user_profile_ctx.username.set(user_view.username.clone());
                    user_profile_ctx.avatar.set(user_view.avatar().to_string());
                    user_profile_ctx.bio.set(user_view.bio().to_string());
                    #[cfg(target_arch = "wasm32")]
                    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok()).flatten() {
                        let _ = storage.set_item("gilvave_cached_username", &user_view.username);
                        let _ = storage.set_item("gilvave_cached_avatar", user_view.avatar());
                        let _ = storage.set_item("gilvave_cached_bio", user_view.bio());
                    }
                    crate::components::ui::context_menu::show_toast(
                        &modal_ctx_clone,
                        false,
                        "Профиль успешно обновлён".to_string(),
                    );
                }
                Err(err) => {
                    crate::components::ui::context_menu::show_toast(
                        &modal_ctx_clone,
                        true,
                        format!("Ошибка обновления профиля: {}", err.1),
                    );
                }
            }
        });
    };

    let handle_change_password = move || {
        if new_password.with(|p| p.len() < 6) {
            password_msg.set("Пароль должен быть не менее 6 символов".to_string());
            return;
        }
        if new_password.with(|np| confirm_password.with(|cp| np != cp)) {
            password_msg.set("Пароли не совпадают".to_string());
            return;
        }
        password_msg.set("Пароль успешно обновлён!".to_string());
        old_password.set(String::new());
        new_password.set(String::new());
        confirm_password.set(String::new());
    };

    let handle_logout = move || {
        show_logout_confirm.set(true);
    };

    let confirm_logout = move || {
        show_logout_confirm.set(false);
        modal_context.is_profile_settings_open.set(false);
        screen_wrapper.set(ActiveScreen::Login);
    };

    view! {
        div(
            class="server-modal-overlay large",
            on:click=move |_| close(),
        ) {
            div(
                class="server-modal profile-settings-modal",
                on:click=move |e: web_sys::MouseEvent| e.stop_propagation(),
            ) {
                div(class="profile-modal-top-bar") {
                    div(class="profile-modal-title") { "Настройки профиля" }
                    button(class="modal-close-icon-btn", on:click=move |_| close(), title="Закрыть") {
                        CloseSmallIcon()
                    }
                }

                div(class="profile-modal-layout") {
                    div(class="profile-sidebar-tabs") {
                        h4 { "НАСТРОЙКИ" }
                        div(
                            class=classes(vec![
                                "profile-tab-item".into(),
                                ("active", { active_tab.get() == SettingsTab::Profile }.into()).into(),
                            ]),
                            on:click=move |_| active_tab.set(SettingsTab::Profile),
                        ) {
                            UserIcon()
                            span { "Профиль" }
                        }
                        div(
                            class=classes(vec![
                                "profile-tab-item".into(),
                                ("active", { active_tab.get() == SettingsTab::Appearance }.into()).into(),
                            ]),
                            on:click=move |_| active_tab.set(SettingsTab::Appearance),
                        ) {
                            PaletteIcon()
                            span { "Внешний вид" }
                        }
                        div(
                            class=classes(vec![
                                "profile-tab-item".into(),
                                ("active", { active_tab.get() == SettingsTab::System }.into()).into(),
                            ]),
                            on:click=move |_| active_tab.set(SettingsTab::System),
                        ) {
                            GearIcon()
                            span { "Система" }
                        }
                        div(
                            class=classes(vec![
                                "profile-tab-item".into(),
                                ("active", { active_tab.get() == SettingsTab::Security }.into()).into(),
                            ]),
                            on:click=move |_| active_tab.set(SettingsTab::Security),
                        ) {
                            LockIcon()
                            span { "Безопасность" }
                        }
                        div(
                            class=classes(vec![
                                "profile-tab-item".into(),
                                ("active", { active_tab.get() == SettingsTab::Sessions }.into()).into(),
                            ]),
                            on:click=move |_| active_tab.set(SettingsTab::Sessions),
                        ) {
                            MonitorIcon()
                            span { "Сессии" }
                        }

                        div(class="profile-tab-divider")

                        div(class="profile-tab-item logout", on:click=move |_| handle_logout()) {
                            LogOutIcon()
                            span { "Выйти из аккаунта" }
                        }
                    }

                    div(class="profile-content-body") {
                        (match active_tab.get() {
                            SettingsTab::Profile => view! {
                                ProfileTab(
                                    name_input=name_input,
                                    avatar_input=avatar_input,
                                    banner_input=banner_input,
                                    bio_input=bio_input,
                                    on_cancel=close,
                                    on_save=handle_save,
                                )
                            },
                            SettingsTab::Appearance => view! {
                                AppearanceTab(
                                    theme_status_msg=theme_status_msg,
                                    theme_status_is_error=theme_status_is_error,
                                )
                            },
                            SettingsTab::System => view! {
                                SystemTab()
                            },
                            SettingsTab::Security => view! {
                                SecurityTab(
                                    old_password=old_password,
                                    new_password=new_password,
                                    confirm_password=confirm_password,
                                    password_msg=password_msg,
                                    on_change_password=handle_change_password,
                                    on_cancel=close,
                                    on_logout=handle_logout,
                                    on_go_to_sessions=move || active_tab.set(SettingsTab::Sessions),
                                )
                            },
                            SettingsTab::Sessions => view! {
                                SessionsTab()
                            },
                        })
                    }
                }
            }
        }

        (if show_logout_confirm.get() {
            view! {
                div(
                    class="island-confirm-overlay profile-logout-confirm-overlay",
                    on:click=move |e: web_sys::MouseEvent| {
                        e.stop_propagation();
                        show_logout_confirm.set(false);
                    },
                ) {
                    div(
                        class="island-confirm-card profile-logout-confirm-card",
                        on:click=move |e: web_sys::MouseEvent| e.stop_propagation(),
                    ) {
                        div(class="island-confirm-header") {
                            div(class="island-confirm-icon-badge danger") {
                                LogOutIcon()
                            }
                            span(class="island-confirm-title") { "Выйти из аккаунта?" }
                        }
                        p(class="island-confirm-desc") {
                            "Вы действительно хотите завершить текущую сессию и выйти из своей учётной записи на этом устройстве?"
                        }
                        div(class="island-confirm-actions") {
                            button(
                                r#type="button",
                                class="island-confirm-btn cancel-btn",
                                on:click=move |e: web_sys::MouseEvent| {
                                    e.stop_propagation();
                                    show_logout_confirm.set(false);
                                },
                            ) {
                                "Отмена"
                            }
                            button(
                                r#type="button",
                                class="island-confirm-btn danger-btn",
                                on:click=move |e: web_sys::MouseEvent| {
                                    e.stop_propagation();
                                    confirm_logout();
                                },
                            ) {
                                "Выйти"
                            }
                        }
                    }
                }
            }
        } else {
            view! {}
        })
    }
}
