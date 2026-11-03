//! Registration form inputs stage (username, email, password, and validation).

use std::rc::Rc;
use sycamore::{prelude::*, web::events::SubmitEvent};

use crate::components::{
    features::auth::social_buttons::SocialButtons,
    ui::{
        divider::Divider, input_group::InputGroup, spinner::Spinner,
        submit_button::SubmitButton,
    },
};

#[derive(Props)]
pub(crate) struct RegisterFormStageProps {
    pub name: Signal<String>,
    pub email: Signal<String>,
    pub password: Signal<String>,
    pub confirm_password: Signal<String>,
    pub name_error: Signal<bool>,
    pub email_error: Signal<bool>,
    pub password_error: Signal<bool>,
    pub confirm_error: Signal<bool>,
    pub server_error: Signal<Option<String>>,
    pub loading: Signal<bool>,
    pub on_submit: Rc<dyn Fn(SubmitEvent)>,
}

#[component]
pub(crate) fn RegisterFormStage(props: RegisterFormStageProps) -> View {
    let name = props.name;
    let email = props.email;
    let password = props.password;
    let confirm_password = props.confirm_password;
    let name_error = props.name_error;
    let email_error = props.email_error;
    let password_error = props.password_error;
    let confirm_error = props.confirm_error;
    let server_error = props.server_error;
    let loading = props.loading;
    let on_submit = props.on_submit;

    view! {
        div(class="register-form-stage") {
            h2 { "Создать аккаунт" }
            p(class="auth-subtitle") { "Присоединяйтесь к общению без границ" }

            (if let Some(err) = server_error.get_clone() {
                view! {
                    div(class="auth-error-banner") { (err) }
                }
            } else {
                view! {}
            })

            form(on:submit=move |e| on_submit(e)) {
                InputGroup(
                    r#type="text",
                    placeholder="Ислам",
                    bind:value=name,
                    label="Имя пользователя",
                    is_error=name_error.into(),
                    error_message="Имя от 2 до 32 символов (без Zalgo/спецсимволов)",
                )

                InputGroup(
                    r#type="email",
                    placeholder="example@mail.com",
                    bind:value=email,
                    label="Email",
                    is_error=email_error.into(),
                    error_message="Введите корректный email",
                )

                InputGroup(
                    r#type="password",
                    placeholder="••••••••",
                    bind:value=password,
                    label="Пароль",
                    is_error=password_error.into(),
                    error_message="Пароль должен быть не менее 6 символов",
                )

                InputGroup(
                    r#type="password",
                    placeholder="••••••••",
                    bind:value=confirm_password,
                    label="Подтверждение пароля",
                    is_error=confirm_error.into(),
                    error_message="Пароли не совпадают",
                )

                a(href="#", class="forgot-link") {
                    "Нажимая кнопку «Зарегистрироваться», вы соглашаетесь с Условиями использования Gilvave"
                }

                SubmitButton() { "Зарегистрироваться" }

                Divider() { "или зарегистрируйтесь через" }

                SocialButtons()
            }

            (if loading.get() {
                view! { Spinner(label="Проверяем почту и отправляем письмо...") }
            } else {
                view! {}
            })
        }
    }
}
