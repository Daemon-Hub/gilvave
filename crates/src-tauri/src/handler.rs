use gilvave_core::{
    dto::command::{CommandArgs, CommandResponse, CommandResult},
    error::{ErrorCode, ErrorInfo},
    security::{get_access_token, get_refresh_token, set_access_token, set_refresh_token},
    settings::collect_device_info,
};
use std::sync::Arc;
use tauri::{Manager, State, WebviewWindow};
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
            {
                let state = window.state::<crate::CloseToTrayState>();
                if state.0.load(std::sync::atomic::Ordering::Relaxed) {
                    let _ = window.hide();
                } else {
                    let _ = window.close();
                }
            }
            CommandResult::Ok(CommandResponse::WindowClose)
        }
        CommandArgs::GetCloseToTray => {
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            {
                let state = window.state::<crate::CloseToTrayState>();
                CommandResult::Ok(CommandResponse::GetCloseToTray(
                    state.0.load(std::sync::atomic::Ordering::Relaxed),
                ))
            }
            #[cfg(any(target_os = "android", target_os = "ios"))]
            {
                CommandResult::Ok(CommandResponse::GetCloseToTray(false))
            }
        }
        CommandArgs::SetCloseToTray { enabled } => {
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            {
                let state = window.state::<crate::CloseToTrayState>();
                state.0.store(enabled, std::sync::atomic::Ordering::Relaxed);
            }
            let _ = enabled;
            CommandResult::Ok(CommandResponse::SetCloseToTray)
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
        CommandArgs::SaveCachedFile {
            file_id,
            filename,
            download_url,
        } => {
            let media_dir = get_media_cache_dir(&window);
            let file_path = get_cache_file_path(&media_dir, &file_id, &filename);

            let token = {
                let db_lock = db.lock().await;
                if let Some(ref conn) = *db_lock {
                    match crate::database::get_token(conn, "access_token") {
                        Ok(Some(t)) if !t.is_empty() => t,
                        _ => get_access_token(),
                    }
                } else {
                    get_access_token()
                }
            };

            let _ = ensure_file_downloaded(&file_path, &file_id, download_url, &token).await;
            CommandResult::Ok(CommandResponse::SaveCachedFile)
        }
        CommandArgs::GetCachedMedia {
            file_id,
            filename,
            download_url,
        } => {
            let media_dir = get_media_cache_dir(&window);
            let effective_filename = if !filename.is_empty() && filename.contains('.') {
                filename.clone()
            } else if let Some(ref d_url) = download_url
                && let Some(last_seg) = d_url.split(['/', '?', '#']).rfind(|s| !s.is_empty())
                && last_seg.contains('.')
            {
                last_seg.to_string()
            } else {
                filename.clone()
            };

            let file_path = get_cache_file_path(&media_dir, &file_id, &effective_filename);

            #[cfg(not(target_os = "android"))]
            let primary_dir = get_desktop_downloads_dir(&window);
            #[cfg(target_os = "android")]
            let primary_dir = get_android_downloads_dir();

            if !file_path.is_file() || std::fs::metadata(&file_path).map(|m| m.len() == 0).unwrap_or(true) {
                let dl_file = primary_dir.join(&effective_filename);
                let dl_fallback = primary_dir.join(&filename);
                if dl_file.is_file() && std::fs::metadata(&dl_file).map(|m| m.len() > 0).unwrap_or(false) {
                    let _ = std::fs::copy(&dl_file, &file_path);
                } else if dl_fallback.is_file() && std::fs::metadata(&dl_fallback).map(|m| m.len() > 0).unwrap_or(false) {
                    let _ = std::fs::copy(&dl_fallback, &file_path);
                } else if let Ok(entries) = std::fs::read_dir(&media_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file()
                            && std::fs::metadata(&path).map(|m| m.len() > 0).unwrap_or(false)
                            && let Some(name_os) = path.file_name().and_then(|n| n.to_str())
                            && (name_os == file_id || name_os.starts_with(&format!("{file_id}_")))
                        {
                            let _ = std::fs::copy(&path, &file_path);
                            break;
                        }
                    }
                }
            }

            if !file_path.is_file() || std::fs::metadata(&file_path).map(|m| m.len() == 0).unwrap_or(true) {
                let token = {
                    let db_lock = db.lock().await;
                    if let Some(ref conn) = *db_lock {
                        match crate::database::get_token(conn, "access_token") {
                            Ok(Some(t)) if !t.is_empty() => t,
                            _ => get_access_token(),
                        }
                    } else {
                        get_access_token()
                    }
                };
                let _ = ensure_file_downloaded(&file_path, &file_id, download_url, &token).await;
            }

            if file_path.is_file()
                && std::fs::metadata(&file_path).map(|m| m.len() > 0).unwrap_or(false)
                && let Ok(bytes) = std::fs::read(&file_path)
            {
                let (file_type, detected_mime) = gilvave_core::files::detect_file_type(&effective_filename, &bytes);
                let mime = if detected_mime != "application/octet-stream" {
                    detected_mime
                } else {
                    let ext = gilvave_core::files::get_file_extension(&effective_filename, "").to_ascii_lowercase();
                    match ext.as_str() {
                        "png" => "image/png",
                        "jpg" | "jpeg" | "jfif" => "image/jpeg",
                        "gif" => "image/gif",
                        "webp" => "image/webp",
                        "svg" => "image/svg+xml",
                        "bmp" => "image/bmp",
                        "ico" => "image/x-icon",
                        "mp4" => "video/mp4",
                        "webm" => "video/webm",
                        _ => match file_type {
                            gilvave_core::dto::files::FileType::Image => "image/png",
                            gilvave_core::dto::files::FileType::Video => "video/mp4",
                            _ => "image/png",
                        },
                    }
                };
                use base64::Engine;
                let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                let data_url = format!("data:{mime};base64,{b64}");
                return Ok(CommandResult::Ok(CommandResponse::GetCachedMedia(Some(data_url))));
            }

            CommandResult::Ok(CommandResponse::GetCachedMedia(None))
        }
        CommandArgs::OpenFile {
            file_id,
            filename,
            download_url,
        } => {
            #[cfg(not(target_os = "android"))]
            let primary_dir = get_desktop_downloads_dir(&window);
            #[cfg(target_os = "android")]
            let primary_dir = get_android_downloads_dir();

            let file_path = primary_dir.join(&filename);

            let media_dir = get_media_cache_dir(&window);
            let cached_path = get_cache_file_path(&media_dir, &file_id, &filename);
            if !file_path.is_file() && cached_path.is_file() {
                let _ = std::fs::copy(&cached_path, &file_path);
            }

            let token = {
                let db_lock = db.lock().await;
                if let Some(ref conn) = *db_lock {
                    match crate::database::get_token(conn, "access_token") {
                        Ok(Some(t)) if !t.is_empty() => t,
                        _ => get_access_token(),
                    }
                } else {
                    get_access_token()
                }
            };

            if let Err(e) = ensure_file_downloaded(&file_path, &file_id, download_url, &token).await {
                return Ok(CommandResult::Error(ErrorInfo::from_code(
                    ErrorCode::CommandExecutionFailed,
                    format!("Failed to open file: {e}"),
                )));
            }

            if !cached_path.is_file() && file_path.is_file() {
                let _ = std::fs::copy(&file_path, &cached_path);
            }

            use tauri_plugin_opener::OpenerExt;
            let path_str = file_path.display().to_string();
            let opened = window.app_handle().opener().open_path(&path_str, None::<&str>).is_ok();
            if !opened {
                tracing::warn!("opener open_path failed, using system fallback");
                #[cfg(target_os = "windows")]
                {
                    use std::os::windows::process::CommandExt;
                    const CREATE_NO_WINDOW: u32 = 0x08000000;
                    let _ = std::process::Command::new("cmd")
                        .args(["/C", "start", "", &path_str])
                        .creation_flags(CREATE_NO_WINDOW)
                        .spawn();
                }
                #[cfg(target_os = "macos")]
                {
                    let _ = std::process::Command::new("open")
                        .arg(&path_str)
                        .spawn();
                }
                #[cfg(all(unix, not(target_os = "macos"), not(target_os = "android")))]
                {
                    let _ = std::process::Command::new("xdg-open")
                        .arg(&path_str)
                        .spawn();
                }
            }
            CommandResult::Ok(CommandResponse::OpenFile)
        }
        CommandArgs::SaveFileToDownloads {
            file_id,
            filename,
            download_url,
        } => {
            #[cfg(not(target_os = "android"))]
            let target_dir = get_desktop_downloads_dir(&window);
            #[cfg(target_os = "android")]
            let target_dir = get_android_downloads_dir();

            let file_path = target_dir.join(&filename);

            let media_dir = get_media_cache_dir(&window);
            let cached_path = get_cache_file_path(&media_dir, &file_id, &filename);
            if !file_path.is_file() && cached_path.is_file() {
                let _ = std::fs::copy(&cached_path, &file_path);
            }

            let token = {
                let db_lock = db.lock().await;
                if let Some(ref conn) = *db_lock {
                    match crate::database::get_token(conn, "access_token") {
                        Ok(Some(t)) if !t.is_empty() => t,
                        _ => get_access_token(),
                    }
                } else {
                    get_access_token()
                }
            };

            if let Err(e) = ensure_file_downloaded(&file_path, &file_id, download_url, &token).await {
                return Ok(CommandResult::Error(ErrorInfo::from_code(
                    ErrorCode::CommandExecutionFailed,
                    format!("Failed to save to downloads: {e}"),
                )));
            }

            if !cached_path.is_file() && file_path.is_file() {
                let _ = std::fs::copy(&file_path, &cached_path);
            }

            CommandResult::Ok(CommandResponse::SaveFileToDownloads(file_path.display().to_string()))
        }
    })
}

fn get_media_cache_dir(window: &WebviewWindow) -> std::path::PathBuf {
    use tauri::Manager;
    let app_dir = window
        .path()
        .app_local_data_dir()
        .unwrap_or_else(|_| std::env::temp_dir().join("Gilvave"));
    let media_dir = app_dir.join("media_cache");
    let _ = std::fs::create_dir_all(&media_dir);
    media_dir
}

fn get_cache_file_path(media_dir: &std::path::Path, file_id: &str, filename: &str) -> std::path::PathBuf {
    let safe_name: String = filename
        .chars()
        .filter(|c| c.is_alphanumeric() || matches!(c, '.' | '_' | '-'))
        .collect();
    if safe_name.is_empty() {
        media_dir.join(file_id)
    } else {
        media_dir.join(format!("{}_{}", file_id, safe_name))
    }
}

#[cfg(not(target_os = "android"))]
fn get_desktop_downloads_dir(window: &tauri::WebviewWindow) -> std::path::PathBuf {
    use tauri::Manager;
    if let Ok(dl) = window.path().download_dir() {
        return dl.join("Gilvave");
    }
    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        return std::path::PathBuf::from(userprofile).join("Downloads").join("Gilvave");
    }
    if let Ok(home) = std::env::var("HOME") {
        return std::path::PathBuf::from(home).join("Downloads").join("Gilvave");
    }
    std::env::temp_dir().join("Gilvave").join("Downloads")
}

#[cfg(target_os = "android")]
fn get_android_downloads_dir() -> std::path::PathBuf {
    let p_downloads = std::path::PathBuf::from("/sdcard/Downloads/Gilvave");
    let p_download = std::path::PathBuf::from("/sdcard/Download/Gilvave");
    let p_storage = std::path::PathBuf::from("/storage/emulated/0/Download/Gilvave");

    if p_downloads.exists() || std::path::Path::new("/sdcard/Downloads").exists() {
        p_downloads
    } else if p_download.exists() || std::path::Path::new("/sdcard/Download").exists() {
        p_download
    } else if std::path::Path::new("/storage/emulated/0/Download").exists() {
        p_storage
    } else {
        p_downloads
    }
}


async fn ensure_file_downloaded(
    target_path: &std::path::Path,
    file_id: &str,
    download_url: Option<String>,
    token: &str,
) -> Result<(), String> {
    if target_path.is_file() && std::fs::metadata(target_path).map(|m| m.len() > 0).unwrap_or(false) {
        return Ok(());
    }

    if let Some(parent) = target_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let raw_url = download_url.unwrap_or_default();
    let url = if raw_url.starts_with("http://") || raw_url.starts_with("https://") {
        raw_url
    } else if let Some(fid) = raw_url.strip_prefix("file:") {
        format!("{}/files/{}", gilvave_core::settings::BASE_HTTP_URL, fid.trim())
    } else if let Some(fid) = raw_url.strip_prefix("file_id:") {
        format!("{}/files/{}", gilvave_core::settings::BASE_HTTP_URL, fid.trim())
    } else if raw_url.starts_with('/') {
        format!("{}{}", gilvave_core::settings::BASE_HTTP_URL, raw_url)
    } else if !file_id.is_empty() {
        format!("{}/files/{}", gilvave_core::settings::BASE_HTTP_URL, file_id)
    } else {
        raw_url
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

    let canonical_url = if !file_id.is_empty() {
        Some(format!("{}/files/{}", gilvave_core::settings::BASE_HTTP_URL, file_id))
    } else {
        None
    };

    let mut resp = None;
    if url.starts_with("http://") || url.starts_with("https://") {
        let mut req = client.get(&url);
        if !token.is_empty() {
            req = req.header("Authorization", format!("Bearer {token}"));
        }
        if let Ok(r) = req.send().await
            && r.status().is_success()
        {
            resp = Some(r);
        }
    }

    if resp.is_none()
        && let Some(ref c_url) = canonical_url
        && c_url != &url
    {
        let mut req = client.get(c_url);
        if !token.is_empty() {
            req = req.header("Authorization", format!("Bearer {token}"));
        }
        if let Ok(r) = req.send().await
            && r.status().is_success()
        {
            resp = Some(r);
        }
    }

    let resp = resp.ok_or_else(|| format!("Failed to download file from {url}"))?;

    use futures_util::StreamExt;
    use std::io::Write;

    let filename = target_path.file_name().and_then(|n| n.to_str()).unwrap_or("file");
    let temp_path = target_path.with_file_name(format!(".tmp_{}_{}", file_id, filename));
    let mut file = std::fs::File::create(&temp_path)
        .map_err(|e| format!("Failed to create temp file: {e}"))?;

    let mut stream = resp.bytes_stream();
    while let Some(chunk_res) = stream.next().await {
        let chunk = chunk_res.map_err(|e| format!("Error streaming chunk: {e}"))?;
        file.write_all(&chunk)
            .map_err(|e| format!("Error writing chunk to disk: {e}"))?;
    }
    file.flush()
        .map_err(|e| format!("Error flushing file: {e}"))?;
    drop(file);

    if let Err(_e) = std::fs::rename(&temp_path, target_path) {
        let _ = std::fs::copy(&temp_path, target_path);
        let _ = std::fs::remove_file(&temp_path);
    }

    Ok(())
}
