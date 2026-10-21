use sycamore::prelude::*;

use crate::components::ui::icons::{ChevronDownIcon, RefreshCwIcon};

#[component(inline_props)]
pub fn ShowMoreBanner(
    is_visible: Signal<bool>,
    is_loading: Signal<bool>,
    on_click: impl Fn(web_sys::MouseEvent) + 'static,
) -> View {
    let on_click_handler = move |e: web_sys::MouseEvent| {
        if !is_loading.get() {
            on_click(e);
        }
    };

    view! {
        div(
            class=move || {
                if is_visible.get() {
                    "show-more-floating-container visible"
                } else {
                    "show-more-floating-container"
                }
            },
        ) {
            button(
                class=move || {
                    if is_loading.get() {
                        "show-more-banner loading"
                    } else {
                        "show-more-banner"
                    }
                },
                on:click=on_click_handler,
                disabled=move || is_loading.get(),
            ) {
                (move || {
                    if is_loading.get() {
                        view! {
                            RefreshCwIcon()
                            span { "Загрузка..." }
                        }
                    } else {
                        view! {
                            ChevronDownIcon()
                            span { "Показать ещё" }
                        }
                    }
                })
            }
        }
    }
}
