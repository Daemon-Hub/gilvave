//! Email confirmation waiting stage with radar pulse animation and manual verification.

use std::rc::Rc;
use sycamore::prelude::*;
use crate::components::ui::icons::{ArrowLeftIcon, CheckIcon, MailIcon};

#[derive(Props)]
pub(crate) struct WaitingConfirmationProps {
    pub email: String,
    pub manual_checking: Signal<bool>,
    pub manual_status_msg: Signal<Option<String>>,
    pub on_manual_check: Rc<dyn Fn()>,
    pub on_back: Rc<dyn Fn()>,
}

#[component]
pub(crate) fn WaitingConfirmationStage(props: WaitingConfirmationProps) -> View {
    let email_str = props.email;
    let manual_checking = props.manual_checking;
    let manual_status_msg = props.manual_status_msg;
    let on_manual_check = props.on_manual_check;
    let on_back = props.on_back;

    view! {
        div(class="waiting-confirmation-panel") {
            div(class="waiting-icon-container") {
                div(class="waiting-icon-halo")
                div(class="waiting-icon-bubble") {
                    MailIcon()
                }
            }

            h2 { "Проверьте вашу почту" }
            p(class="waiting-subtitle") { "Мы отправили письмо со ссылкой подтверждения на адрес:" }
            div(class="waiting-email-chip") {
                span { (email_str) }
            }

            p(class="waiting-instruction") {
                "Пожалуйста, перейдите по ссылке в письме для активации аккаунта. После подтверждения приложение автоматически войдёт в систему."
            }

            div(class="waiting-polling-indicator") {
                div(class="polling-radar") {
                    div(class="radar-core")
                    div(class="radar-wave")
                }
                span(class="polling-label") {
                    (if manual_checking.get() {
                        "Проверка статуса..."
                    } else {
                        "Ожидание подтверждения..."
                    })
                }
            }

            (if let Some(msg) = manual_status_msg.get_clone() {
                view! {
                    div(class="waiting-status-alert") { (msg) }
                }
            } else {
                view! {}
            })

            div(class="waiting-actions") {
                button(
                    class="submit-btn confirm-check-btn",
                    on:click=move |_| on_manual_check(),
                    disabled=manual_checking.get(),
                ) {
                    CheckIcon()
                    span {
                        (if manual_checking.get() {
                            "Проверяем..."
                        } else {
                            "Я подтвердил"
                        })
                    }
                }

                button(
                    class="waiting-back-btn",
                    on:click=move |_| on_back(),
                ) {
                    ArrowLeftIcon()
                    span { "Назад / Изменить почту" }
                }
            }

            p(class="waiting-spam-hint") {
                "Не пришло письмо? Проверьте папку «Спам» или «Рассылки»."
            }
        }
    }
}
