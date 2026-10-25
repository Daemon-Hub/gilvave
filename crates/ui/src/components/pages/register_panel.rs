use serde::Deserialize;
use std::rc::Rc;
use sycamore::{
    prelude::*,
    web::{console_error, console_log, events::SubmitEvent},
};
use validator::Validate;

use crate::{
    components::{
        common::{ActiveScreen, ScreenWrapper, classes},
        features::auth::social_buttons::SocialButtons,
        ui::{
            divider::Divider,
            icons::{ArrowLeftIcon, CheckIcon, MailIcon},
            input_group::InputGroup,
            spinner::Spinner,
            submit_button::SubmitButton,
        },
    },
    http::api::Api,
    utils::invoke_command,
};
use gilvave_core::dto::{
    command::{CommandArgs, CommandResponse, CommandResult},
    users::{LoginRequest, RegisterRequest},
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

#[derive(Props)]
struct WaitingConfirmationProps {
    email: String,
    manual_checking: Signal<bool>,
    manual_status_msg: Signal<Option<String>>,
    on_manual_check: Rc<dyn Fn()>,
    on_back: Rc<dyn Fn()>,
}

#[component]
fn WaitingConfirmationStage(props: WaitingConfirmationProps) -> View {
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

#[derive(Props)]
struct RegisterFormStageProps {
    name: Signal<String>,
    email: Signal<String>,
    password: Signal<String>,
    confirm_password: Signal<String>,
    name_error: Signal<bool>,
    email_error: Signal<bool>,
    password_error: Signal<bool>,
    confirm_error: Signal<bool>,
    server_error: Signal<Option<String>>,
    loading: Signal<bool>,
    on_submit: Rc<dyn Fn(SubmitEvent)>,
}

#[component]
fn RegisterFormStage(props: RegisterFormStageProps) -> View {
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

#[component]
pub fn RegisterPanel(props: RegisterFormProps) -> View {
    let screen_wrapper = use_context::<ScreenWrapper>();

    let name = create_signal(String::new());
    let email = create_signal(String::new());
    let password = create_signal(String::new());
    let confirm_password = create_signal(String::new());

    let name_error = create_signal(false);
    let email_error = create_signal(false);
    let password_error = create_signal(false);
    let confirm_error = create_signal(false);
    let loading = create_signal(false);
    let server_error = create_signal::<Option<String>>(None);

    // Waiting for email confirmation state
    let is_waiting_confirmation = create_signal(false);
    let waiting_email = create_signal(String::new());
    let saved_password = create_signal(String::new());
    let poll_session_id = create_signal(0u32);
    let manual_checking = create_signal(false);
    let manual_status_msg = create_signal::<Option<String>>(None);

    let reset_errors = move || {
        name_error.set(false);
        email_error.set(false);
        password_error.set(false);
        confirm_error.set(false);
    };

    // Synchronize waiting confirmation state with screen_wrapper
    create_effect(move || {
        if screen_wrapper.is_register_confirm() {
            if waiting_email.with(|e| e.is_empty()) {
                waiting_email.set("demo@gilvave.app".to_string());
            }
            is_waiting_confirmation.set(true);
        } else if !screen_wrapper.is_register() && is_waiting_confirmation.get() {
            is_waiting_confirmation.set(false);
            poll_session_id.set(poll_session_id.get() + 1);
        }
    });

    on_cleanup(move || {
        is_waiting_confirmation.set(false);
        poll_session_id.set(poll_session_id.get() + 1);
    });

    let on_submit: Rc<dyn Fn(SubmitEvent)> = {
        let name = name;
        let email = email;
        let password = password;
        let confirm_password = confirm_password;
        let name_error = name_error;
        let email_error = email_error;
        let password_error = password_error;
        let confirm_error = confirm_error;
        let loading = loading;
        let server_error = server_error;
        let is_waiting_confirmation = is_waiting_confirmation;
        let waiting_email = waiting_email;
        let saved_password = saved_password;
        let poll_session_id = poll_session_id;
        let manual_status_msg = manual_status_msg;
        let screen_wrapper = screen_wrapper;

        Rc::new(move |event: SubmitEvent| {
            event.prevent_default();
            if loading.get() {
                return;
            }
            server_error.set(None);

            let data = RegisterData {
                name: name.get_clone(),
                email: email.get_clone(),
                password: password.get_clone(),
                confirm_password: confirm_password.get_clone(),
            };

            match data.validate() {
                Ok(_) => {
                    if gilvave_core::validation::validate_username(&data.name).is_err() {
                        reset_errors();
                        name_error.set(true);
                        return;
                    }
                    reset_errors();

                    let email_for_waiting = data.email.clone();
                    let password_for_waiting = data.password.clone();

                    wasm_bindgen_futures::spawn_local(async move {
                        loading.set(true);
                        let res = Api::register_validate(RegisterRequest {
                            username: data.name,
                            email: data.email,
                            password: data.password,
                        })
                        .await;
                        loading.set(false);
                        match res {
                            Ok(_) => {
                                waiting_email.set(email_for_waiting.clone());
                                saved_password.set(password_for_waiting.clone());
                                is_waiting_confirmation.set(true);
                                manual_status_msg.set(None);

                                let current_session_id = poll_session_id.get() + 1;
                                poll_session_id.set(current_session_id);

                                // Background autologin polling every 5 seconds
                                wasm_bindgen_futures::spawn_local(async move {
                                    loop {
                                        gloo_timers::future::sleep(
                                            core::time::Duration::from_millis(5000),
                                        )
                                        .await;

                                        if !is_waiting_confirmation.get()
                                            || poll_session_id.get() != current_session_id
                                            || !screen_wrapper.is_register()
                                        {
                                            break;
                                        }

                                        let device_info = match invoke_command(
                                            CommandArgs::GetDeviceInfo.to_json(),
                                        )
                                        .await
                                        {
                                            CommandResult::Ok(CommandResponse::GetDeviceInfo(
                                                info,
                                            )) => info.to_json(),
                                            _ => gilvave_core::settings::collect_device_info()
                                                .to_json(),
                                        };

                                        if !is_waiting_confirmation.get()
                                            || poll_session_id.get() != current_session_id
                                            || !screen_wrapper.is_register()
                                        {
                                            break;
                                        }

                                        let login_res = Api::login(LoginRequest {
                                            email: email_for_waiting.clone(),
                                            password: password_for_waiting.clone(),
                                            device_info,
                                        })
                                        .await;

                                        if !is_waiting_confirmation.get()
                                            || poll_session_id.get() != current_session_id
                                            || !screen_wrapper.is_register()
                                        {
                                            break;
                                        }

                                        if login_res.is_ok() {
                                            console_log!(
                                                "Email confirmed and autologin succeeded!"
                                            );
                                            is_waiting_confirmation.set(false);
                                            screen_wrapper.set(ActiveScreen::Home);
                                            break;
                                        }
                                    }
                                });
                            }
                            Err(err) => {
                                console_error!("{err:#?}");
                                let msg = if !err.1.trim().is_empty() {
                                    err.1
                                } else {
                                    "Ошибка при регистрации. Пожалуйста, попробуйте снова."
                                        .to_string()
                                };
                                server_error.set(Some(msg));
                            }
                        }
                    });
                }
                Err(errors) => {
                    reset_errors();
                    for (field, _) in errors.field_errors() {
                        match field.as_ref() {
                            "name" => name_error.set(true),
                            "email" => email_error.set(true),
                            "password" => password_error.set(true),
                            "confirm_password" => confirm_error.set(true),
                            _ => {}
                        }
                    }
                }
            }
        })
    };

    let on_manual_check: Rc<dyn Fn()> = {
        let manual_checking = manual_checking;
        let manual_status_msg = manual_status_msg;
        let waiting_email = waiting_email;
        let saved_password = saved_password;
        let is_waiting_confirmation = is_waiting_confirmation;
        let screen_wrapper = screen_wrapper;

        Rc::new(move || {
            if manual_checking.get() {
                return;
            }
            manual_checking.set(true);
            manual_status_msg.set(None);

            let target_email = waiting_email.get_clone();
            let target_password = saved_password.get_clone();

            wasm_bindgen_futures::spawn_local(async move {
                let device_info = match invoke_command(CommandArgs::GetDeviceInfo.to_json()).await {
                    CommandResult::Ok(CommandResponse::GetDeviceInfo(info)) => info.to_json(),
                    _ => gilvave_core::settings::collect_device_info().to_json(),
                };

                let res = Api::login(LoginRequest {
                    email: target_email,
                    password: target_password,
                    device_info,
                })
                .await;

                manual_checking.set(false);

                match res {
                    Ok(_) => {
                        console_log!("Manual login verification succeeded!");
                        is_waiting_confirmation.set(false);
                        screen_wrapper.set(ActiveScreen::Home);
                    }
                    Err(_) => {
                        manual_status_msg.set(Some(
                            "Подтверждение пока не получено. Пожалуйста, откройте ссылку из письма в браузере.".to_string(),
                        ));
                    }
                }
            });
        })
    };

    let on_back: Rc<dyn Fn()> = {
        let is_waiting_confirmation = is_waiting_confirmation;
        let poll_session_id = poll_session_id;
        let manual_status_msg = manual_status_msg;
        let server_error = server_error;

        Rc::new(move || {
            is_waiting_confirmation.set(false);
            poll_session_id.set(poll_session_id.get() + 1);
            manual_status_msg.set(None);
            server_error.set(None);
            if screen_wrapper.is_register_confirm() {
                screen_wrapper.set(ActiveScreen::Register);
            }
        })
    };

    let on_submit_sig = create_signal(on_submit);
    let on_manual_check_sig = create_signal(on_manual_check);
    let on_back_sig = create_signal(on_back);

    view! {
        div(
            class=classes(vec![
                "form-panel".into(),
                ("active", props.is_active.clone()).into(),
            ]),
        ) {
            (if is_waiting_confirmation.get() {
                view! {
                    WaitingConfirmationStage(
                        email=waiting_email.get_clone(),
                        manual_checking=manual_checking,
                        manual_status_msg=manual_status_msg,
                        on_manual_check=on_manual_check_sig.get_clone(),
                        on_back=on_back_sig.get_clone(),
                    )
                }
            } else {
                view! {}
            })

            (if !is_waiting_confirmation.get() {
                view! {
                    RegisterFormStage(
                        name=name,
                        email=email,
                        password=password,
                        confirm_password=confirm_password,
                        name_error=name_error,
                        email_error=email_error,
                        password_error=password_error,
                        confirm_error=confirm_error,
                        server_error=server_error,
                        loading=loading,
                        on_submit=on_submit_sig.get_clone(),
                    )
                }
            } else {
                view! {}
            })
        }
    }
}
