use gilvave_core::{
    dto::command::{CommandArgs, CommandResponse, CommandResult},
    error::{ErrorCode, ErrorInfo},
    security::{get_access_token, get_refresh_token, set_access_token, set_refresh_token},
    settings::collect_device_info,
};
use std::sync::Arc;
use tauri::{State, WebviewWindow};
use tokio::sync::Mutex;

#[tauri::command]
pub async fn handle_command(
    window: WebviewWindow,
    command: CommandArgs,
    db: State<'_, Arc<Mutex<Option<rusqlite::Connection>>>>,
) -> Result<CommandResult, ()> {
    let _ = &window;
    Ok(match command {
        CommandArgs::GetAccessToken => {
            let db_lock = db.lock().await;
            let token = if let Some(ref conn) = *db_lock {
                match crate::database::get_token(conn, "access_token") {
                    Ok(Some(t)) if !t.is_empty() => t,
                    _ => get_access_token(),
                }
            } else {
                get_access_token()
            };
            CommandResult::Ok(CommandResponse::GetAccessToken(token))
        }
        CommandArgs::GetRefreshToken => {
            let db_lock = db.lock().await;
            let token = if let Some(ref conn) = *db_lock {
                match crate::database::get_token(conn, "refresh_token") {
                    Ok(Some(t)) if !t.is_empty() => t,
                    _ => get_refresh_token(),
                }
            } else {
                get_refresh_token()
            };
            CommandResult::Ok(CommandResponse::GetRefreshToken(token))
        }
        CommandArgs::SetAccessToken { token } => {
            set_access_token(&token);
            let db_lock = db.lock().await;
            if let Some(ref conn) = *db_lock {
                let _ = crate::database::set_token(conn, "access_token", &token);
            }
            CommandResult::Ok(CommandResponse::SetAccessToken)
        }
        CommandArgs::SetRefreshToken { token } => {
            set_refresh_token(&token);
            let db_lock = db.lock().await;
            if let Some(ref conn) = *db_lock {
                let _ = crate::database::set_token(conn, "refresh_token", &token);
            }
            CommandResult::Ok(CommandResponse::SetRefreshToken)
        }
        CommandArgs::GetDeviceInfo => {
            CommandResult::Ok(CommandResponse::GetDeviceInfo(collect_device_info()))
        }
        CommandArgs::WindowMinimize => {
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            let _ = window.minimize();
            CommandResult::Ok(CommandResponse::WindowMinimize)
        }
        CommandArgs::WindowToggleMaximize => {
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            if window.is_maximized().unwrap_or(false) {
                let _ = window.unmaximize();
            } else {
                let _ = window.maximize();
            }
            CommandResult::Ok(CommandResponse::WindowToggleMaximize)
        }
        CommandArgs::WindowClose => {
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            let _ = window.close();
            CommandResult::Ok(CommandResponse::WindowClose)
        }
        CommandArgs::WindowStartDragging => {
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            let _ = window.start_dragging();
            CommandResult::Ok(CommandResponse::WindowStartDragging)
        }
        CommandArgs::OpenDevTools => {
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            if window.is_devtools_open() {
                window.close_devtools();
            } else {
                window.open_devtools();
            }
            CommandResult::Ok(CommandResponse::OpenDevTools)
        }
        // SQLite Offline Cache Commands
        CommandArgs::GetCachedMessages {
            channel_id,
            limit,
            before,
        } => {
            let db_lock = db.lock().await;
            if let Some(ref conn) = *db_lock {
                match crate::database::get_cached_messages(conn, channel_id, limit, before) {
                    Ok(messages) => CommandResult::Ok(CommandResponse::GetCachedMessages(messages)),
                    Err(e) => CommandResult::Error(ErrorInfo::from_code(
                        ErrorCode::CommandExecutionFailed,
                        format!("Failed to get cached messages: {e}"),
                    )),
                }
            } else {
                CommandResult::Ok(CommandResponse::GetCachedMessages(vec![]))
            }
        }
        CommandArgs::SyncCachedMessages {
            channel_id,
            messages,
        } => {
            let mut db_lock = db.lock().await;
            if let Some(ref mut conn) = *db_lock {
                match crate::database::sync_cached_messages(conn, channel_id, &messages) {
                    Ok(()) => CommandResult::Ok(CommandResponse::SyncCachedMessages),
                    Err(e) => CommandResult::Error(ErrorInfo::from_code(
                        ErrorCode::CommandExecutionFailed,
                        format!("Failed to sync cached messages: {e}"),
                    )),
                }
            } else {
                CommandResult::Ok(CommandResponse::SyncCachedMessages)
            }
        }
        CommandArgs::SaveCachedMessage { message } => {
            let db_lock = db.lock().await;
            if let Some(ref conn) = *db_lock {
                match crate::database::save_cached_message(conn, &message) {
                    Ok(()) => CommandResult::Ok(CommandResponse::SaveCachedMessage),
                    Err(e) => CommandResult::Error(ErrorInfo::from_code(
                        ErrorCode::CommandExecutionFailed,
                        format!("Failed to save cached message: {e}"),
                    )),
                }
            } else {
                CommandResult::Ok(CommandResponse::SaveCachedMessage)
            }
        }
        CommandArgs::UpdateCachedMessage {
            id,
            content,
            edited,
        } => {
            let db_lock = db.lock().await;
            if let Some(ref conn) = *db_lock {
                match crate::database::update_cached_message(conn, id, &content, edited) {
                    Ok(()) => CommandResult::Ok(CommandResponse::UpdateCachedMessage),
                    Err(e) => CommandResult::Error(ErrorInfo::from_code(
                        ErrorCode::CommandExecutionFailed,
                        format!("Failed to update cached message: {e}"),
                    )),
                }
            } else {
                CommandResult::Ok(CommandResponse::UpdateCachedMessage)
            }
        }
        CommandArgs::DeleteCachedMessage { id } => {
            let db_lock = db.lock().await;
            if let Some(ref conn) = *db_lock {
                match crate::database::delete_cached_message(conn, id) {
                    Ok(()) => CommandResult::Ok(CommandResponse::DeleteCachedMessage),
                    Err(e) => CommandResult::Error(ErrorInfo::from_code(
                        ErrorCode::CommandExecutionFailed,
                        format!("Failed to delete cached message: {e}"),
                    )),
                }
            } else {
                CommandResult::Ok(CommandResponse::DeleteCachedMessage)
            }
        }
        CommandArgs::GetCachedServers => {
            let db_lock = db.lock().await;
            if let Some(ref conn) = *db_lock {
                match crate::database::get_cached_servers(conn) {
                    Ok(servers) => CommandResult::Ok(CommandResponse::GetCachedServers(servers)),
                    Err(e) => CommandResult::Error(ErrorInfo::from_code(
                        ErrorCode::CommandExecutionFailed,
                        format!("Failed to get cached servers: {e}"),
                    )),
                }
            } else {
                CommandResult::Ok(CommandResponse::GetCachedServers(vec![]))
            }
        }
        CommandArgs::SyncCachedServers { servers } => {
            let mut db_lock = db.lock().await;
            if let Some(ref mut conn) = *db_lock {
                match crate::database::sync_cached_servers(conn, &servers) {
                    Ok(()) => CommandResult::Ok(CommandResponse::SyncCachedServers),
                    Err(e) => CommandResult::Error(ErrorInfo::from_code(
                        ErrorCode::CommandExecutionFailed,
                        format!("Failed to sync cached servers: {e}"),
                    )),
                }
            } else {
                CommandResult::Ok(CommandResponse::SyncCachedServers)
            }
        }
        CommandArgs::GetCachedChannels { server_id } => {
            let db_lock = db.lock().await;
            if let Some(ref conn) = *db_lock {
                match crate::database::get_cached_channels(conn, server_id) {
                    Ok(channels) => CommandResult::Ok(CommandResponse::GetCachedChannels(channels)),
                    Err(e) => CommandResult::Error(ErrorInfo::from_code(
                        ErrorCode::CommandExecutionFailed,
                        format!("Failed to get cached channels: {e}"),
                    )),
                }
            } else {
                CommandResult::Ok(CommandResponse::GetCachedChannels(vec![]))
            }
        }
        CommandArgs::SyncCachedChannels {
            server_id,
            channels,
        } => {
            let mut db_lock = db.lock().await;
            if let Some(ref mut conn) = *db_lock {
                match crate::database::sync_cached_channels(conn, server_id, &channels) {
                    Ok(()) => CommandResult::Ok(CommandResponse::SyncCachedChannels),
                    Err(e) => CommandResult::Error(ErrorInfo::from_code(
                        ErrorCode::CommandExecutionFailed,
                        format!("Failed to sync cached channels: {e}"),
                    )),
                }
            } else {
                CommandResult::Ok(CommandResponse::SyncCachedChannels)
            }
        }
    })
}
