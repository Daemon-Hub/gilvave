//! Context menu UI component, overlay rendering, and public API re-exports.

pub mod builder;
pub mod clipboard;
pub mod icons;
pub mod menu_input;
pub mod menu_message;
pub mod menu_server_channel;
pub mod menu_user;
pub mod types;

use sycamore::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

pub use builder::handle_global_context_menu;
#[allow(unused_imports)]
pub use clipboard::{
    copy_to_clipboard, cut_from_target_element, get_window_selection_text,
    paste_into_target_element, select_all_in_target_element, show_toast,
};
#[allow(unused_imports)]
pub use types::{ContextMenuContext, ContextMenuItem};

use self::icons::MenuIcon;

#[component]
pub fn AppContextMenu() -> View {
    let ctx = use_context::<ContextMenuContext>();

    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            let ctx_clone = ctx;
            let on_key = wasm_bindgen::closure::Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(
                move |e: web_sys::KeyboardEvent| {
                    if e.key() == "Escape" && ctx_clone.is_open.get() {
                        ctx_clone.close();
                    }
                },
            );
            let _ =
                window.add_event_listener_with_callback("keydown", on_key.as_ref().unchecked_ref());
            on_key.forget();
        }
    }

    view! {
        (if ctx.is_open.get() {
            let raw_x = ctx.x.get();
            let raw_y = ctx.y.get();
            let items = ctx.items.get_clone();
            let item_count = items.len();
            let est_w = 224;
            let est_h = (item_count as i32 * 38) + 24;

            #[cfg(target_arch = "wasm32")]
            let (win_w, win_h) = if let Some(w) = web_sys::window() {
                let width = w.inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(1024.0) as i32;
                let height = w.inner_height().ok().and_then(|v| v.as_f64()).unwrap_or(768.0) as i32;
                (width, height)
            } else {
                (1024, 768)
            };
            #[cfg(not(target_arch = "wasm32"))]
            let (win_w, win_h) = (1024, 768);

            let opens_upwards = raw_y > (win_h / 2) || (raw_y + est_h > win_h - 16);
            let opens_leftwards = raw_x + est_w > win_w - 12;

            let (y_style, up_class) = if opens_upwards {
                let bottom_px = (win_h - raw_y).max(8);
                let max_h = (raw_y - 16).max(120);
                (format!("bottom: {bottom_px}px; max-height: {max_h}px;"), "open-up")
            } else {
                let top_px = raw_y.max(8);
                let max_h = (win_h - raw_y - 16).max(120);
                (format!("top: {top_px}px; max-height: {max_h}px;"), "open-down")
            };

            let (x_style, left_class) = if opens_leftwards {
                let right_px = (win_w - raw_x).max(8);
                (format!("right: {right_px}px;"), "open-left")
            } else {
                let left_px = raw_x.max(8);
                (format!("left: {left_px}px;"), "")
            };

            let menu_style = format!("{x_style} {y_style}");
            let menu_class = format!("app-context-menu {up_class} {left_class}").trim().to_string();

            let on_menu_click = move |e: web_sys::MouseEvent| {
                e.stop_propagation();
                #[cfg(target_arch = "wasm32")]
                if let Some(target) = e.target() {
                    if let Ok(el) = target.dyn_into::<web_sys::Element>() {
                        if let Ok(Some(btn)) = el.closest("[data-item-index]") {
                            if let Ok(idx) = btn
                                .get_attribute("data-item-index")
                                .unwrap_or_default()
                                .parse::<usize>()
                            {
                                let action = ctx
                                    .items
                                    .with(|it| it.get(idx).map(|item| item.on_click.clone()));
                                ctx.close();
                                if let Some(action) = action {
                                    action();
                                }
                            }
                        }
                    }
                }
            };

            let item_views = items
                .into_iter()
                .enumerate()
                .map(|(idx, item)| {
                    let mut btn_class = String::from("context-menu-item");
                    if item.danger {
                        btn_class.push_str(" is-danger");
                    }
                    let sep_view = if item.separator_above {
                        view! { div(class="context-menu-separator") }
                    } else {
                        view! {}
                    };
                    let shortcut_view = if let Some(sc) = item.shortcut {
                        view! { span(class="context-menu-shortcut") { (sc) } }
                    } else {
                        view! {}
                    };
                    view! {
                        (sep_view)
                        button(
                            r#type="button",
                            class=btn_class,
                            data-item-index=idx.to_string(),
                        ) {
                            span(class="context-menu-icon") {
                                MenuIcon(name=item.icon)
                            }
                            span(class="context-menu-label") { (item.label) }
                            (shortcut_view)
                        }
                    }
                })
                .collect::<Vec<_>>();

            view! {
                div(
                    class="context-menu-backdrop",
                    on:click=move |_| ctx.close(),
                    on:contextmenu=move |e: web_sys::MouseEvent| {
                        e.prevent_default();
                        ctx.close();
                    },
                )
                div(
                    class=menu_class,
                    style=menu_style,
                    role="menu",
                    tabindex="-1",
                    on:click=on_menu_click,
                    on:contextmenu=move |e: web_sys::MouseEvent| e.prevent_default(),
                ) {
                    (item_views)
                }
            }
        } else {
            view! {}
        })
    }
}

