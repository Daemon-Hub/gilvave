use sycamore::{futures::spawn_local_scoped, prelude::*};
use wasm_bindgen::{JsCast, closure::Closure};

use crate::{
    components::{
        common::{ActiveScreen, ConnectivityContext, NetworkStatus, ScreenWrapper},
        layout::header::AppHeader,
        pages::home_panel::HomePanel,
        templates::auth_form::AuthForm,
        ui::{
            AppContextMenu, ContextMenuContext, NetworkStatusBadge,
            context_menu::handle_global_context_menu,
        },
    },
    http::api::Api,
    utils::is_tauri,
};

#[component]
pub fn App() -> View {
    let screen_wrapper = ScreenWrapper(create_signal(ActiveScreen::Loading));
    provide_context(screen_wrapper);

    let is_initially_online = web_sys::window()
        .map(|w| w.navigator().on_line())
        .unwrap_or(true);
    let initial_status = if is_initially_online {
        NetworkStatus::Online
    } else {
        NetworkStatus::Offline
    };
    let connectivity = ConnectivityContext {
        status: create_signal(initial_status),
    };
    provide_context(connectivity);

    let context_menu_ctx = ContextMenuContext::new();
    provide_context(context_menu_ctx);

    let on_global_context_menu = move |event: web_sys::MouseEvent| {
        if event.default_prevented() {
            return;
        }
        handle_global_context_menu(&event, &context_menu_ctx, None, None, None, None);
    };

    // Listen to network online/offline events on window
    if let Some(window) = web_sys::window() {
        let status_offline = connectivity.status;
        let on_offline = Closure::<dyn Fn()>::new(move || {
            status_offline.set(NetworkStatus::Offline);
        });
        let _ = window.add_event_listener_with_callback(
            "offline",
            on_offline.as_ref().unchecked_ref(),
        );
        on_offline.forget();

        let status_online = connectivity.status;
        let on_online = Closure::<dyn Fn()>::new(move || {
            status_online.set(NetworkStatus::Reconnecting);
        });
        let _ = window.add_event_listener_with_callback(
            "online",
            on_online.as_ref().unchecked_ref(),
        );
        on_online.forget();
    }

    // Subscribe to WebSocket gateway connectivity events
    let mut gateway_rx = crate::gateway::handler::subscribe_gateway_events();
    spawn_local_scoped(async move {
        use futures_util::StreamExt;
        while let Some(event) = gateway_rx.next().await {
            match event {
                crate::gateway::handler::GatewayEvent::Connected => {
                    if connectivity.status.get() != NetworkStatus::Online {
                        connectivity.status.set(NetworkStatus::Connected);
                        gloo_timers::future::sleep(core::time::Duration::from_millis(1800)).await;
                        if connectivity.status.get() == NetworkStatus::Connected {
                            connectivity.status.set(NetworkStatus::Online);
                        }
                    }
                }
                crate::gateway::handler::GatewayEvent::Disconnected => {
                    connectivity.status.set(NetworkStatus::Offline);
                }
                _ => {}
            }
        }
    });

    // Block all keyboard shortcuts used to debug or inspect the app (F12, Ctrl+Shift+I/J/C/K/E, Ctrl+U)
    if let Some(window) = web_sys::window() {
        let on_debug_key =
            Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(move |e: web_sys::KeyboardEvent| {
                let key = e.key();
                let is_ctrl = e.ctrl_key() || e.meta_key();
                let is_shift = e.shift_key();

                let is_debug_shortcut = key == "F12"
                    || (is_ctrl
                        && is_shift
                        && matches!(
                            key.as_str(),
                            "I" | "i" | "J" | "j" | "C" | "c" | "K" | "k" | "E" | "e"
                        ))
                    || (is_ctrl && !is_shift && !e.alt_key() && key.eq_ignore_ascii_case("U"));

                if is_debug_shortcut {
                    e.prevent_default();
                    e.stop_propagation();
                }
            });
        let _ = window.add_event_listener_with_callback_and_bool(
            "keydown",
            on_debug_key.as_ref().unchecked_ref(),
            true,
        );
        on_debug_key.forget();
    }

    // Pause background aurora animations when window/tab is hidden (0% GPU)
    if let Some(window) = web_sys::window()
        && let Some(doc) = window.document()
    {
        let doc_clone = doc.clone();
        let on_visibility = Closure::<dyn Fn()>::new(move || {
            if let Some(root) = doc_clone.document_element() {
                if doc_clone.hidden() {
                    let _ = root.set_attribute("data-window-hidden", "true");
                } else {
                    let _ = root.remove_attribute("data-window-hidden");
                }
            }
        });
        let _ = doc.add_event_listener_with_callback(
            "visibilitychange",
            on_visibility.as_ref().unchecked_ref(),
        );
        on_visibility.forget();
    }

    spawn_local_scoped(async move {
        // Check if saved authentication tokens exist
        let token = Api::fetch_access_token().await;
        if token.is_empty() {
            screen_wrapper.set(ActiveScreen::Login);
            return;
        }

        match Api::get_profile().await {
            Ok(_) => {
                screen_wrapper.set(ActiveScreen::Home);
            }
            Err(e) => {
                web_sys::console::warn_1(&format!("[APP] Session check failed: {:?}", e).into());
                // If unauthorized (401 / 403), credentials are invalid or revoked
                if e.0 == 401 || e.0 == 403 {
                    screen_wrapper.set(ActiveScreen::Login);
                } else if is_tauri() {
                    // Offline mode on Desktop/Mobile: saved token exists and network failed,
                    // allow entering Home screen to view cached SQLite data
                    web_sys::console::log_1(&"[APP] Offline mode active: loading cached workspace".into());
                    connectivity.status.set(NetworkStatus::Offline);
                    screen_wrapper.set(ActiveScreen::Home);
                } else {
                    screen_wrapper.set(ActiveScreen::Login);
                }
            }
        }
    });

    view! {
        div(class="bg-aurora", aria-hidden="true") {
            div(class="aurora-orb aurora-cyan")
            div(class="aurora-orb aurora-violet")
            div(class="aurora-orb aurora-amber")
        }
        (if screen_wrapper.is_loading() {
            view! {
                div(class="app-startup-loader") {
                    div(class="spinner") {
                        div(class="spinner-ring")
                        div(class="spinner-ring")
                        div(class="spinner-ring")
                        div(class="spinner-core")
                    }
                }
            }
        } else {
            view! {}
        })
        main(on:contextmenu=on_global_context_menu) {
            AppHeader()
            AuthForm()
            HomePanel()
            AppContextMenu()
            NetworkStatusBadge()
        }
    }
}
