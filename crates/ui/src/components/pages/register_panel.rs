use std::collections::HashMap;

use gilvave_core::dto::user::RegisterRequest;
use serde::Deserialize;
use sycamore::{
    futures::spawn_local_scoped,
    prelude::*,
    web::{console_error, events::SubmitEvent},
};
use validator::Validate;

use crate::{
    components::{
        common::{ActiveScreen, ScreenWrapper, classes},
        features::auth::social_buttons::SocialButtons,
        ui::{
            divider::Divider, input_group::InputGroup, spinner::Spinner,
            submit_button::SubmitButton,
        },
    },
    http::api::Api,
};

#[derive(Props)]
pub struct RegisterFormProps {
    is_active: MaybeDyn<bool>,
}

#[derive(Debug, Validate, Deserialize)]
struct RegisterData {
    #[validate(length(min = 2, max = 32))]
    name: String,
    #[validate(email)]
    email: String,
    #[validate(length(min = 6, max = 50))]
    password: String,
    #[validate(must_match(other = "password"))]
    confirm_password: String,
}

#[component]
pub fn RegisterPanel(props: RegisterFormProps) -> View {
    let name = create_signal(String::new());
    let email = create_signal(String::new());
    let password = create_signal(String::new());
    let confirm_password = create_signal(String::new());

    let name_error = create_signal(false);
    let email_error = create_signal(false);
    let password_error = create_signal(false);
    let confirm_error = create_signal(false);
    let loading = create_signal(false);

    let mut form_map: HashMap<&str, Signal<bool>> = [
        ("name", name_error),
        ("email", email_error),
        ("password", password_error),
        ("confirm_password", confirm_error),
    ]
    .into();

    let on_submit = move |event: SubmitEvent| {
        // not reset page
        event.prevent_default();

        let data = RegisterData {
            name: name.get_clone(),
            email: email.get_clone(),
            password: password.get_clone(),
            confirm_password: confirm_password.get_clone(),
        };

        match data.validate() {
            Ok(_) => {
                if gilvave_core::validation::validate_username(&data.name).is_err() {
                    form_map.values_mut().for_each(|signal| signal.set(false));
                    name_error.set(true);
                    return;
                }
                form_map.values_mut().for_each(|signal| signal.set(false));

                spawn_local_scoped(async move {
                    loading.set(true);
                    let res = Api::register(RegisterRequest {
                        username: data.name,
                        email: data.email,
                        password: data.password,
                    })
                    .await;
                    loading.set(false);
                    match res {
                        Ok(_) => {
                            console_log!("Register Success");
                            use_context::<ScreenWrapper>().set(ActiveScreen::Login);
                        }
                        Err(err) => {
                            console_error!("{err:#?}");
                        }
                    }
                });
            }
            Err(errors) => {
                form_map.values_mut().for_each(|signal| signal.set(false));
                for (field, _) in errors.field_errors() {
                    form_map[field.as_ref()].set(true);
                }
            }
        }
    };

    view! {
        div(
            class=classes(vec![
                "form-panel".into(),
                ("active", props.is_active.clone()).into(),
            ]),
        ) {
            h2 { "Создать аккаунт" }
            p(class="auth-subtitle") { "Присоединяйтесь к общению без границ" }
            form(on:submit=on_submit) {
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

                SubmitButton(on:click=move |_| console_log!("{name}")) { "Зарегистрироваться" }

                Divider() { "или зарегистрируйтесь через" }

                SocialButtons()
            }
            (if loading.get() {
                view! { Spinner() }
            } else {
                view! {}
            })
        }
    }
}
