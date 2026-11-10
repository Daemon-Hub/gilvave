//! Mobile gestures: Android hardware back button handler and touch swipe navigation.

use sycamore::prelude::*;
use wasm_bindgen::{prelude::Closure, JsCast};
use gilvave_core::dto::servers::channels::ChannelView;

use crate::components::common::{
    ChannelContext, CreateServerContext, ServerContext, UiModalContext,
};

pub(crate) fn setup_android_back_button(
    modal_ctx: UiModalContext,
    chan_ctx: ChannelContext,
    create_srv_ctx: CreateServerContext,
) {
    if let Some(window) = web_sys::window() {
        let on_android_back = Closure::<dyn Fn() -> bool>::new(move || {
            // Priority 0: Lightbox photo viewer & Fullscreen video player
            if modal_ctx.active_fullscreen_video.with(|v| v.is_some()) {
                modal_ctx.active_fullscreen_video.set(None);
                return true;
            }
            if modal_ctx.lightbox_image.with(|img| img.is_some()) {
                modal_ctx.lightbox_image.set(None);
                return true;
            }
            // Priority 1: Message editing/replying/forwarding/editor modals
            if modal_ctx.is_message_editor_open.get() {
                modal_ctx.is_message_editor_open.set(false);
                return true;
            }
            if modal_ctx.is_forward_modal_open.get() {
                modal_ctx.is_forward_modal_open.set(false);
                return true;
            }
            if modal_ctx.editing_message.with(|m| m.is_some()) {
                modal_ctx.editing_message.set(None);
                return true;
            }
            if modal_ctx.replying_to.with(|r| r.is_some()) {
                modal_ctx.replying_to.set(None);
                return true;
            }
            // Priority 2: Full modals & overlays
            if create_srv_ctx.is_modal_open.get() {
                create_srv_ctx.is_modal_open.set(false);
                return true;
            }
            if modal_ctx.is_theme_json_editor_open.get() {
                modal_ctx.is_theme_json_editor_open.set(false);
                return true;
            }
            if modal_ctx.is_theme_catalog_open.get() {
                modal_ctx.is_theme_catalog_open.set(false);
                return true;
            }
            if modal_ctx.is_server_settings_open.get() {
                modal_ctx.is_server_settings_open.set(false);
                return true;
            }
            if modal_ctx.is_server_overview_open.get() {
                modal_ctx.is_server_overview_open.set(false);
                return true;
            }
            if modal_ctx.is_create_channel_open.get() {
                modal_ctx.is_create_channel_open.set(false);
                return true;
            }
            if modal_ctx.is_profile_settings_open.get() {
                modal_ctx.is_profile_settings_open.set(false);
                return true;
            }
            // Priority 3: Active DM chat -> back to chats list
            if modal_ctx.selected_dm_name.with(|dm| dm.is_some()) {
                modal_ctx.selected_dm_name.set(None);
                return true;
            }
            // Priority 4: Active server channel -> back to channels list
            if chan_ctx.current.with(|c| c.is_some()) {
                chan_ctx.current.set(None);
                return true;
            }
            // Already at the top-level list: do not consume, allow Android to exit/minimize
            false
        });
        let on_esc_key = Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(move |e: web_sys::KeyboardEvent| {
            if e.key() == "Escape" {
                if modal_ctx.active_fullscreen_video.with(|v| v.is_some()) {
                    modal_ctx.active_fullscreen_video.set(None);
                    return;
                }
                if modal_ctx.lightbox_image.with(|img| img.is_some()) {
                    modal_ctx.lightbox_image.set(None);
                }
            }
        });
        let _ = window.add_event_listener_with_callback("keydown", on_esc_key.as_ref().unchecked_ref());
        on_esc_key.forget();

        let _ = js_sys::Reflect::set(
            &window,
            &wasm_bindgen::JsValue::from_str("__onAndroidBackPressed"),
            on_android_back.as_ref().unchecked_ref(),
        );
        on_android_back.forget();
    }
}

pub(crate) fn use_swipe_gestures(
    channel_context: ChannelContext,
    ui_modal_context: UiModalContext,
    server_context: ServerContext,
) -> (impl Fn(web_sys::TouchEvent) + Copy + 'static, impl Fn(web_sys::TouchEvent) + Copy + 'static) {
    let last_selected_channel = create_signal::<Option<ChannelView>>(None);
    let last_selected_dm = create_signal::<Option<String>>(None);

    create_effect(move || {
        if let Some(ch) = channel_context.current.get_clone() {
            last_selected_channel.set(Some(ch));
        }
    });

    create_effect(move || {
        if let Some(dm) = ui_modal_context.selected_dm_name.get_clone() {
            last_selected_dm.set(Some(dm));
        }
    });

    let touch_start_x = create_signal(0.0f64);
    let touch_start_y = create_signal(0.0f64);

    let on_touch_start = move |e: web_sys::TouchEvent| {
        if let Some(touch) = e.touches().get(0) {
            touch_start_x.set(touch.client_x() as f64);
            touch_start_y.set(touch.client_y() as f64);
        }
    };

    let on_touch_end = move |e: web_sys::TouchEvent| {
        if let Some(target) = e.target()
            && let Ok(el) = target.dyn_into::<web_sys::HtmlElement>()
        {
            let tag = el.tag_name().to_lowercase();
            if tag == "input" || tag == "textarea" || el.is_content_editable() {
                return;
            }
        }

        if let Some(touch) = e.changed_touches().get(0) {
            let dx = (touch.client_x() as f64) - touch_start_x.get();
            let dy = (touch.client_y() as f64) - touch_start_y.get();

            // Swipe threshold: at least 60px horizontal, with horizontal dominance (dx > 1.4 * dy)
            if dx.abs() > 60.0 && dx.abs() > 1.4 * dy.abs() {
                let is_in_server = server_context.current.with(|c| c.is_some());
                let is_in_channel = channel_context.current.with(|c| c.is_some());
                let is_in_dm = ui_modal_context.selected_dm_name.with(|dm| dm.is_some());

                if dx > 0.0 {
                    // Swipe right -> return from active chat to channel/chats list
                    if is_in_channel {
                        channel_context.current.set(None);
                    } else if is_in_dm {
                        ui_modal_context.selected_dm_name.set(None);
                    }
                } else {
                    // Swipe left -> return to active chat from channel/chats list
                    if is_in_server && !is_in_channel {
                        if let Some(ch) = last_selected_channel.get_clone() {
                            channel_context.current.set(Some(ch));
                        } else {
                            let first_text = channel_context.text.with(|list| list.first().cloned());
                            if let Some(ch) = first_text {
                                channel_context.current.set(Some(ch));
                            }
                        }
                    } else if !is_in_server
                        && !is_in_dm
                        && let Some(dm) = last_selected_dm.get_clone()
                    {
                        ui_modal_context.selected_dm_name.set(Some(dm));
                    }
                }
            }
        }
    };

    (on_touch_start, on_touch_end)
}
