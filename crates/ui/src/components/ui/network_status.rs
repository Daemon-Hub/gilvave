use sycamore::prelude::*;

use crate::components::common::{ConnectivityContext, NetworkStatus};

#[component]
pub fn NetworkStatusBadge() -> View {
    let connectivity = use_context::<ConnectivityContext>();
    let status = connectivity.status;

    let on_retry = move |_| {
        if status.get() == NetworkStatus::Offline {
            status.set(NetworkStatus::Reconnecting);
            sycamore::futures::spawn_local_scoped(async move {
                if crate::http::api::Api::get_profile().await.is_ok() {
                    status.set(NetworkStatus::Connected);
                    gloo_timers::future::sleep(core::time::Duration::from_millis(1800)).await;
                    if status.get() == NetworkStatus::Connected {
                        status.set(NetworkStatus::Online);
                    }
                } else {
                    status.set(NetworkStatus::Offline);
                }
            });
        }
    };

    let badge_class = create_memo(move || match status.get() {
        NetworkStatus::Online => "floating-network-badge status-online",
        NetworkStatus::Offline => "floating-network-badge status-offline",
        NetworkStatus::Reconnecting => "floating-network-badge status-reconnecting",
        NetworkStatus::Connected => "floating-network-badge status-connected",
    });

    view! {
        div(
            class=badge_class.get(),
            role="status",
            aria-live="polite",
            title=if status.get() == NetworkStatus::Offline { "Нет подключения к интернету. Нажмите для повтора" } else { "" },
            on:click=on_retry
        ) {
            (match status.get() {
                NetworkStatus::Online => view! {},
                NetworkStatus::Offline => view! {
                    span(class="badge-dot")
                    span(class="badge-icon") {
                        svg(
                            viewBox="0 0 24 24",
                            fill="none",
                            stroke="currentColor",
                            stroke-width="2",
                            stroke-linecap="round",
                            stroke-linejoin="round"
                        ) {
                            line(x1="1", y1="1", x2="23", y2="23")
                            path(d="M16.72 11.06A10.94 10.94 0 0 1 19 12.55")
                            path(d="M5 12.55a10.94 10.94 0 0 1 5.17-2.39")
                            path(d="M10.71 5.05A16 16 0 0 1 22.58 9")
                            path(d="M1.42 9a15.91 15.91 0 0 1 4.7-2.88")
                            path(d="M8.53 16.11a6 6 0 0 1 6.95 0")
                            line(x1="12", y1="20", x2="12.01", y2="20")
                        }
                    }
                    span(class="badge-text") { "Оффлайн" }
                },
                NetworkStatus::Reconnecting => view! {
                    span(class="badge-icon") {
                        svg(
                            class="spin-icon",
                            viewBox="0 0 24 24",
                            fill="none",
                            stroke="currentColor",
                            stroke-width="2.2",
                            stroke-linecap="round",
                            stroke-linejoin="round"
                        ) {
                            path(d="M21.5 2v6h-6M21.34 15.57a10 10 0 1 1-.57-8.38l5.67-5.67")
                        }
                    }
                    span(class="badge-text") { "Подключение..." }
                },
                NetworkStatus::Connected => view! {
                    span(class="badge-icon") {
                        svg(
                            viewBox="0 0 24 24",
                            fill="none",
                            stroke="currentColor",
                            stroke-width="2.5",
                            stroke-linecap="round",
                            stroke-linejoin="round"
                        ) {
                            polyline(points="20 6 9 17 4 12")
                        }
                    }
                    span(class="badge-text") { "Подключено" }
                },
            })
        }
    }
}
