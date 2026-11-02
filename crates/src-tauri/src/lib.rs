use std::sync::Arc;

use tauri::Manager;
use tauri_plugin_tracing::{Builder, LevelFilter, WebviewLayer};
use tokio::sync::Mutex;
use tracing_subscriber::{Registry, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::database::initialize_database;

mod database;
mod handler;

#[derive(Debug)]
pub struct CloseToTrayState(pub std::sync::atomic::AtomicBool);

impl CloseToTrayState {
    pub fn new(enabled: bool) -> Self {
        Self(std::sync::atomic::AtomicBool::new(enabled))
    }
}

impl Default for CloseToTrayState {
    fn default() -> Self {
        Self::new(true)
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::menu::{Menu, MenuItem};
    use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

    let open_item = MenuItem::with_id(app, "open", "Открыть Gilvave", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Выход", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open_item, &quit_item])?;

    let mut builder = TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("Gilvave")
        .on_menu_event(|app, event| {
            match event.id().as_ref() {
                "open" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.unminimize();
                        let _ = window.set_focus();
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.unminimize();
                        let _ = window.set_focus();
                    }
                }
            }
        });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    builder.build(app)?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let tracing_builder = Builder::new()
        .with_max_level(LevelFilter::DEBUG)
        .with_target("hyper", LevelFilter::WARN);
    let filter = tracing_builder.build_filter();

    tauri::Builder::default()
        .plugin(tracing_builder.build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![handler::handle_command])
        .on_window_event(|window, event| {
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let state = window.state::<CloseToTrayState>();
                if state.0.load(std::sync::atomic::Ordering::Relaxed) {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .setup(|app| {
            let app_handle = app.handle().clone();

            Registry::default()
                .with(fmt::layer())
                .with(WebviewLayer::new(app.handle().clone()))
                .with(filter)
                .init();

            let db = initialize_database(&app_handle).unwrap();
            app_handle.manage(Arc::new(Mutex::new(Some(db))));
            app_handle.manage(CloseToTrayState::new(true));

            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            setup_tray(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("ошибка при запуске приложения Tauri");
}
