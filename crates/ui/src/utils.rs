use gilvave_core::dto::command::CommandResult;
use serde_json::Value;
use tauri_sys::core::invoke;

pub fn is_tauri() -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };
    let Ok(internals) = js_sys::Reflect::get(&window, &"__TAURI_INTERNALS__".into()) else {
        return false;
    };
    if internals.is_undefined() || internals.is_null() {
        return false;
    }
    let is_polyfill = js_sys::Reflect::get(&internals, &"__IS_POLYFILL__".into())
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    !is_polyfill
}

pub async fn invoke_command(args: Value) -> CommandResult {
    if is_tauri() {
        invoke::<CommandResult>("handle_command", &args).await
    } else {
        handle_command_web(args).await
    }
}

#[rustfmt::skip]
async fn handle_command_web(args: Value) -> CommandResult {
    use crate::security::{
        get_access_token, get_refresh_token, set_access_token, set_refresh_token,
    };
    use gilvave_core::{
        dto::command::{CommandArgs, CommandResponse},
        error::{ErrorCode, ErrorInfo},
        settings::collect_device_info,
    };

    let command: CommandArgs = match args.get("command") {
        Some(cmd_val) => match serde_json::from_value(cmd_val.clone()) {
            Ok(c) => c,
            Err(e) => {
                return CommandResult::Error(ErrorInfo::from_code(
                    ErrorCode::CommandDeserializationFailed,
                    format!("Failed to deserialize command: {e}"),
                ));
            }
        },
        None => match serde_json::from_value(args) {
            Ok(c) => c,
            Err(e) => {
                return CommandResult::Error(ErrorInfo::from_code(
                    ErrorCode::CommandDeserializationFailed,
                    format!("Failed to deserialize command: {e}"),
                ));
            }
        },
    };

    match command {
        CommandArgs::GetAccessToken => {
            CommandResult::Ok(CommandResponse::GetAccessToken(get_access_token()))
        }
        CommandArgs::GetRefreshToken => {
            CommandResult::Ok(CommandResponse::GetRefreshToken(get_refresh_token()))
        }
        CommandArgs::SetAccessToken { token } => {
            set_access_token(&token);
            CommandResult::Ok(CommandResponse::SetAccessToken)
        }
        CommandArgs::SetRefreshToken { token } => {
            set_refresh_token(&token);
            CommandResult::Ok(CommandResponse::SetRefreshToken)
        }
        CommandArgs::GetDeviceInfo => {
            CommandResult::Ok(CommandResponse::GetDeviceInfo(collect_device_info()))
        }
        CommandArgs::WindowMinimize => CommandResult::Ok(CommandResponse::WindowMinimize),
        CommandArgs::WindowToggleMaximize => {
            CommandResult::Ok(CommandResponse::WindowToggleMaximize)
        }
        CommandArgs::WindowClose => CommandResult::Ok(CommandResponse::WindowClose),
        CommandArgs::WindowStartDragging => CommandResult::Ok(CommandResponse::WindowStartDragging),
        CommandArgs::OpenDevTools => {
            #[cfg(target_arch = "wasm32")]
            {
                web_sys::console::log_1(&"[Gilvave] Открытие консоли разработчика...".into());
                let script = r#"
                    (function() {
                        if (window.eruda) {
                            if (window.eruda._isInit) {
                                var el = document.getElementById('eruda');
                                if (el && el.style.display !== 'none') {
                                    window.eruda.hide();
                                } else {
                                    window.eruda.show();
                                }
                            } else {
                                window.eruda.init();
                                window.eruda.show();
                            }
                        } else {
                            var s = document.createElement('script');
                            s.src = 'https://cdn.jsdelivr.net/npm/eruda';
                            s.onload = function() {
                                window.eruda.init();
                                window.eruda.show();
                            };
                            document.body.appendChild(s);
                        }
                    })();
                "#;
                let _ = js_sys::eval(script);
            }
            CommandResult::Ok(CommandResponse::OpenDevTools)
        }
    }
}

pub fn get_local_offset() -> time::UtcOffset {
    #[cfg(target_arch = "wasm32")]
    {
        let offset_minutes = js_sys::Date::new_0().get_timezone_offset() as i32;
        time::UtcOffset::from_whole_seconds(-offset_minutes * 60).unwrap_or(time::UtcOffset::UTC)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        time::UtcOffset::UTC
    }
}

pub fn to_local_datetime(dt: time::OffsetDateTime) -> time::OffsetDateTime {
    dt.to_offset(get_local_offset())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gilvave_core::dto::command::CommandArgs;
    use time::OffsetDateTime;

    #[test]
    fn test_to_local_datetime() {
        let dt = OffsetDateTime::now_utc();
        let local = to_local_datetime(dt);
        assert_eq!(dt.unix_timestamp(), local.unix_timestamp());
    }

    #[test]
    fn test_command_args_json_roundtrip() {
        let cmd = CommandArgs::GetAccessToken;
        let json = cmd.to_json();
        assert!(json.get("command").is_some());
        let parsed: CommandArgs = serde_json::from_value(json["command"].clone()).unwrap();
        assert!(matches!(parsed, CommandArgs::GetAccessToken));
    }

    #[test]
    fn test_event_message_view_deserialization() {
        use gilvave_core::dto::messages::MessageView;
        use gilvave_core::ids::{ChannelId, MessageId, UserId};

        let view = MessageView {
            id: MessageId::default(),
            channel_id: ChannelId::default(),
            author_id: Some(UserId::default()),
            author_name: "test_user".to_string(),
            content: "hello world".to_string(),
            edited: false,
            reply_to_id: None,
            forwarded_from: None,
            created_at: OffsetDateTime::now_utc(),
            is_pending: false,
        };
        let payload_json = serde_json::to_value(&view).unwrap();
        let event_json = serde_json::json!({
            "event": "message_new",
            "id": 1,
            "payload": payload_json
        });
        let raw_str = serde_json::to_string(&event_json).unwrap();
        let deserialized: tauri_sys::event::Event<MessageView> =
            serde_json::from_str(&raw_str).unwrap();
        assert_eq!(deserialized.event, "message_new");
        assert_eq!(deserialized.id, 1);
        assert_eq!(deserialized.payload, view);
    }
}
