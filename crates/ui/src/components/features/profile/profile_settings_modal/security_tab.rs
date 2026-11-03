//! Security tab with password change form and session shortcuts.

use sycamore::prelude::*;
use crate::components::ui::icons::{LogOutIcon, MonitorIcon};

#[component(inline_props)]
pub fn SecurityTab<F1, F2, F3, F4>(
    old_password: Signal<String>,
    new_password: Signal<String>,
    confirm_password: Signal<String>,
    password_msg: Signal<String>,
    on_change_password: F1,
    on_cancel: F2,
    on_logout: F3,
    on_go_to_sessions: F4,
) -> View
where
    F1: Fn() + Copy + 'static,
    F2: Fn() + Copy + 'static,
    F3: Fn() + Copy + 'static,
    F4: Fn() + Copy + 'static,
{
    let handle_change_password = move |_| on_change_password();
    let close = move |_| on_cancel();
    let handle_logout = move |_| on_logout();
    let go_to_sessions = move |_| on_go_to_sessions();

    view! {
                                div(class="tab-pane") {
                                    h3(class="tab-title") { "Безопасность и вход" }

                                    div(class="input-group") {
                                        label { "ТЕКУЩИЙ ПАРОЛЬ" }
                                        input(r#type="password", placeholder="••••••••", bind:value=old_password)
                                    }

                                    div(class="input-group") {
                                        label { "НОВЫЙ ПАРОЛЬ" }
                                        input(r#type="password", placeholder="Минимум 6 символов", bind:value=new_password)
                                    }

                                    div(class="input-group") {
                                        label { "ПОДТВЕРДИТЕ НОВЫЙ ПАРОЛЬ" }
                                        input(r#type="password", placeholder="••••••••", bind:value=confirm_password)
                                    }

                                    (if password_msg.with(|m| !m.is_empty()) {
                                        let msg = password_msg.get_clone();
                                        view! {
                                            div(class="password-status-msg") { (msg) }
                                        }
                                    } else {
                                        view! {}
                                    })

                                    div(class="create-form-actions modal-footer-buttons") {
                                        button(class="back-btn", on:click=close) { "Отмена" }
                                        button(class="submit-btn", on:click=handle_change_password) { "Обновить пароль" }
                                    }

                                    div(class="security-divider")

                                    div(class="danger-zone") {
                                        h4 { "СЕССИЯ" }
                                        p { "Выход из текущей учётной записи на этом устройстве." }
                                        div(class="danger-zone-actions") {
                                            button(class="danger-btn logout-btn", on:click=handle_logout) {
                                                LogOutIcon()
                                                span { "Выйти из аккаунта" }
                                            }
                                            button(
                                                class="secondary-btn sessions-shortcut-btn",
                                                on:click=go_to_sessions,
                                            ) {
                                                MonitorIcon()
                                                span { "Управление сессиями →" }
                                            }
                                        }
                                    }
                                }

    }
}
