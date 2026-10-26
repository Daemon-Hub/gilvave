use gilvave_core::{
    dto::command::{CommandArgs, CommandResponse, CommandResult},
    security::{get_access_token, get_refresh_token, set_access_token, set_refresh_token},
    settings::collect_device_info,
};
use tauri::WebviewWindow;

#[tauri::command]
pub async fn handle_command(
    window: WebviewWindow,
    command: CommandArgs,
) -> Result<CommandResult, ()> {
    Ok(match command {
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
        CommandArgs::WindowMinimize => {
            let _ = window.minimize();
            CommandResult::Ok(CommandResponse::WindowMinimize)
        }
        CommandArgs::WindowToggleMaximize => {
            if window.is_maximized().unwrap_or(false) {
                let _ = window.unmaximize();
            } else {
                let _ = window.maximize();
            }
            CommandResult::Ok(CommandResponse::WindowToggleMaximize)
        }
        CommandArgs::WindowClose => {
            let _ = window.close();
            CommandResult::Ok(CommandResponse::WindowClose)
        }
        CommandArgs::WindowStartDragging => {
            let _ = window.start_dragging();
            CommandResult::Ok(CommandResponse::WindowStartDragging)
        }
        CommandArgs::OpenDevTools => {
            if window.is_devtools_open() {
                window.close_devtools();
            } else {
                window.open_devtools();
            }
            CommandResult::Ok(CommandResponse::OpenDevTools)
        }
    })
}
