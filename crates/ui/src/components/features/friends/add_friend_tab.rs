//! Send friend request by username tab with input validation and feedback.

use sycamore::prelude::*;
use crate::components::ui::icons::UsersIcon;

pub(crate) fn view_add_friend<F>(
    input_signal: Signal<String>,
    result_signal: Signal<Option<(bool, String)>>,
    on_submit: F,
) -> View
where
    F: Fn() + Copy + 'static,
{
    let handle_keydown = move |e: web_sys::KeyboardEvent| {
        if e.key() == "Enter" {
            on_submit();
        }
    };

    view! {
        div(class="friends-tab-content add-friend-pane") {
            div(class="add-friend-hero-box") {
                h3(class="add-friend-title") { "ДОБАВИТЬ В ДРУЗЬЯ" }
                p(class="add-friend-desc") {
                    "Вы можете отправить запрос дружбы, указав имя пользователя (username) нужного человека."
                }

                div(class="add-friend-input-box") {
                    input(
                        r#type="text",
                        placeholder="Введите username (например: alex_smirnov)...",
                        bind:value=input_signal,
                        on:keydown=handle_keydown,
                    )
                    button(
                        class="add-friend-submit-btn",
                        on:click=move |_| on_submit(),
                    ) {
                        span { "Отправить запрос" }
                    }
                }

                (if let Some((is_err, msg)) = result_signal.get_clone() {
                    let alert_class = if is_err {
                        "add-friend-status-box error"
                    } else {
                        "add-friend-status-box success"
                    };
                    view! {
                        div(class=alert_class) {
                            span { (msg) }
                        }
                    }
                } else {
                    view! {}
                })
            }

            div(class="add-friend-guide-card") {
                div(class="guide-header") {
                    UsersIcon()
                    h4 { "Как работает система друзей в Gilvave?" }
                }
                ul(class="guide-steps") {
                    li {
                        span(class="step-num") { "1" }
                        span(class="step-text") { "Узнайте уникальное имя пользователя (username) вашего друга." }
                    }
                    li {
                        span(class="step-num") { "2" }
                        span(class="step-text") { "Введите его в поле выше и нажмите «Отправить запрос»." }
                    }
                    li {
                        span(class="step-num") { "3" }
                        span(class="step-text") { "Когда пользователь примет ваше приглашение, он сразу появится во вкладке «Все друзья»!" }
                    }
                }
            }
        }
    }
}
