use sycamore::{futures::spawn_local_scoped, prelude::*};
use wasm_bindgen::{closure::Closure, JsCast};

use crate::{
    components::{
        common::{ActiveScreen, ScreenWrapper},
        layout::header::AppHeader,
        pages::home_panel::HomePanel,
        templates::auth_form::AuthForm,
    },
    http::api::Api,
};

#[component]
pub fn App() -> View {
    let screen_wrapper = ScreenWrapper(create_signal(ActiveScreen::Loading));
    provide_context(screen_wrapper);

    // Pause background aurora animations when window/tab is hidden (0% GPU)
    if let Some(window) = web_sys::window() {
        if let Some(doc) = window.document() {
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
    }

    spawn_local_scoped(async move {
        // Check if saved authentication tokens are valid
        if Api::get_profile().await.is_ok() {
            screen_wrapper.set(ActiveScreen::Home);
        } else {
            screen_wrapper.set(ActiveScreen::Login);
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
        main() {
            AppHeader()
            AuthForm()
            HomePanel()
        }
    }
}

