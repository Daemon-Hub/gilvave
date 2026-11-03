//! Security information tip card, empty state card, and terminate confirmation banner.

use sycamore::prelude::*;
use crate::components::ui::icons::ShieldCheckIcon;

#[component(inline_props)]
pub fn TerminateConfirmBanner<F1, F2>(on_confirm: F1, on_cancel: F2) -> View
where
    F1: Fn(web_sys::MouseEvent) + Copy + 'static,
    F2: Fn(web_sys::MouseEvent) + Copy + 'static,
{
    view! {
        div(class="terminate-confirm-banner") {
            div(class="confirm-banner-content") {
                strong { "Завершить все остальные сессии?" }
                p {
                    "Вы будете разлогинены на всех устройствах, кроме текущего. Для повторного входа потребуется ввести пароль."
                }
            }
            div(class="confirm-banner-actions") {
                button(
                    class="confirm-cancel-btn",
                    on:click=on_cancel,
                ) {
                    "Отмена"
                }
                button(
                    class="confirm-danger-btn",
                    on:click=on_confirm,
                ) {
                    "Да, завершить все"
                }
            }
        }
    }
}

#[component]
pub fn EmptySessionsCard() -> View {
    view! {
        div(class="sessions-empty-card") {
            div(class="empty-icon-shield") {
                ShieldCheckIcon()
            }
            h5 { "Других активных сессий нет" }
            p {
                "Вход в ваш аккаунт выполнен только с этого устройства. Никаких подозрительных входов не обнаружено."
            }
        }
    }
}

#[component]
pub fn SecurityTipCard() -> View {
    view! {
        div(class="sessions-security-tip-card") {
            div(class="tip-icon") {
                ShieldCheckIcon()
            }
            div(class="tip-content") {
                h5 { "Безопасность аккаунта" }
                p {
                    "Если вы увидели незнакомое устройство в списке, немедленно завершите его сессию и измените пароль во вкладке «Безопасность». Это предотвратит несанкционированный доступ к вашей переписке и серверам."
                }
            }
        }
    }
}
