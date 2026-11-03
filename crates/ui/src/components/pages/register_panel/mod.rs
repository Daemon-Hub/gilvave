//! Registration full-page panel coordinating registration form and email verification.

mod register_form;
mod waiting_confirmation;

pub(crate) use register_form::RegisterFormStage;
pub(crate) use waiting_confirmation::WaitingConfirmationStage;

use std::rc::Rc;
use serde::Deserialize;
use sycamore::{
    prelude::*,
    web::{console_error, console_log, events::SubmitEvent},
};
use validator::Validate;

use crate::{
    components::common::{classes, ActiveScreen, ScreenWrapper},
    http::api::Api,
    utils::invoke_command,
};
use gilvave_core::dto::{
    command::{CommandArgs, CommandResponse, CommandResult},
    users::{LoginRequest, RegisterRequest},
};

#[derive(Props)]
pub struct RegisterFormProps {
    pub is_active: MaybeDyn<bool>,
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

    let on_submit: Rc<dyn Fn(SubmitEvent)> = Rc::new(move |event: SubmitEvent| {
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
                                    gloo_timers::future::sleep(core::time::Duration::from_millis(
                                        5000,
                                    ))
                                    .await;

                                    if !is_waiting_confirmation.get()
                                        || poll_session_id.get() != current_session_id
                                        || !screen_wrapper.is_register()
                                    {
                                        break;
                                    }

                                    let device_info =
                                        match invoke_command(CommandArgs::GetDeviceInfo.to_json())
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
                                        console_log!("Email confirmed and autologin succeeded!");
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
                                "Ошибка при регистрации. Пожалуйста, попробуйте снова.".to_string()
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
    });

    let on_manual_check: Rc<dyn Fn()> = Rc::new(move || {
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
    });

    let on_back: Rc<dyn Fn()> = Rc::new(move || {
        is_waiting_confirmation.set(false);
        poll_session_id.set(poll_session_id.get() + 1);
        manual_status_msg.set(None);
        server_error.set(None);
        if screen_wrapper.is_register_confirm() {
            screen_wrapper.set(ActiveScreen::Register);
        }
    });

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
