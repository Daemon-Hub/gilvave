use gilvave_core::dto::command::{CommandArgs, CommandResponse, CommandResult};
use gilvave_core::dto::user::LoginRequest;
use sycamore::web::console_error;
use sycamore::web::events::SubmitEvent;
use sycamore::{futures::spawn_local_scoped, prelude::*};

use crate::components::common::{ActiveScreen, ScreenWrapper, classes};
use crate::components::features::auth::social_buttons::SocialButtons;
use crate::components::ui::divider::Divider;
use crate::components::ui::input_group::InputGroup;
use crate::components::ui::spinner::Spinner;
use crate::components::ui::submit_button::SubmitButton;
use crate::http::api::Api;
use crate::utils::invoke_command;

#[derive(Props)]
pub struct LoginFormProps {
    is_active: MaybeDyn<bool>,
}

#[component]
pub fn LoginPanel(props: LoginFormProps) -> View {
    let email = create_signal(String::new());
    let password = create_signal(String::new());
    let email_error = create_signal(false);
    let password_error = create_signal(false);
    let loading = create_signal(false);

    let on_submit = move |event: SubmitEvent| {
        event.prevent_default();

        let mut is_valid = true;

        if email.with(|e| e.trim().is_empty()) {
            email_error.set(true);
            is_valid = false;
        } else {
            email_error.set(false);
        }

        if password.with(|p| p.is_empty()) {
            password_error.set(true);
            is_valid = false;
        } else {
            password_error.set(false);
        }

        if is_valid {
            email.with(|e| {
                password.with(|p| {
                    console_log!("Вход: email={e}, password={p}");
                });
            });

            spawn_local_scoped(async move {
                loading.set(true);
                let device_info = match invoke_command(CommandArgs::GetDeviceInfo.to_json()).await {
                    CommandResult::Ok(CommandResponse::GetDeviceInfo(info)) => info.to_json(),
                    _ => serde_json::Value::Null,
                };
                let res = Api::login(LoginRequest {
                    email: email.get_clone(),
                    password: password.get_clone(),
                    device_info,
                })
                .await;
                loading.set(false);
                match res {
                    Ok(_) => {
                        console_log!("Login Success");
                        use_context::<ScreenWrapper>().set(ActiveScreen::Home);
                    }
                    Err(err) => {
                        console_error!("{err:#?}")
                    }
                }
            });
        } else {
            console_log!(
                "error: email={}, password={}",
                email_error.get(),
                password_error.get()
            );
        }
    };

    view! {
        div(
            class=classes(vec![
                "form-panel".into(),
                ("active", props.is_active.clone()).into(),
            ]),
        ) {
            h2 { "Добро пожаловать!" }
            p(class="auth-subtitle") { "Войдите в свой аккаунт Gilvave" }
            form(on:submit=on_submit) {
                InputGroup(
                    r#type="text",
                    placeholder="adam@mail.com",
                    bind:value=email,
                    label="Email",
                    is_error=email_error.into(),
                    error_message="Введите email",
                )

                InputGroup(
                    r#type="password",
                    placeholder="••••••••",
                    bind:value=password,
                    label="Пароль",
                    is_error=password_error.into(),
                    error_message="Введите пароль",
                )

                a(href="#", class="forgot-link") { "Забыли пароль?" }

                SubmitButton() { "Войти" }

                Divider() { "или войдите через" }

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
