//! Logic for sending, updating, and dispatching channel messages.

use sycamore::{futures::spawn_local_scoped, prelude::*};
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use super::super::chat_scroll::save_channel_last_read;
use crate::{
    components::common::{ChannelContext, UiModalContext, UserProfileContext},
    gateway::service::WsService,
};
use gilvave_core::ids::MessageId;

pub(crate) fn send_channel_message(
    channel_context: ChannelContext,
    modal_context: UiModalContext,
    draft_message: Signal<String>,
    _user_profile: UserProfileContext,
    container: Option<NodeRef>,
) {
    let Some(channel_id) = channel_context.current.with(|c| c.as_ref().map(|ch| ch.id)) else {
        return;
    };

    let sanitized = match draft_message.with(|msg| gilvave_core::validation::validate_message(msg))
    {
        Ok(s) => s,
        Err(e) => {
            web_sys::console::warn_1(&format!("[MESSAGES_AREA] on_submit ignored: {e}").into());
            return;
        }
    };

    let editing = modal_context.editing_message.get_clone();
    let replying = modal_context.replying_to.get_clone();

    if let Some(edit_msg) = editing {
        spawn_local_scoped(async move {
            web_sys::console::log_1(
                &format!("[MESSAGES_AREA] invoking MessageUpdate for {}", edit_msg.id).into(),
            );
            let res = WsService::message_update(edit_msg.channel_id, edit_msg.id, sanitized).await;
            web_sys::console::log_1(&format!("[MESSAGES_AREA] MessageUpdate res: {res:?}").into());
        });
        modal_context.editing_message.set(None);
    } else {
        let reply_to = replying.as_ref().map(|r| r.id);
        modal_context.replying_to.set(None);

        save_channel_last_read(
            channel_id,
            MessageId::default(),
            time::OffsetDateTime::now_utc().unix_timestamp(),
        );

        spawn_local_scoped(async move {
            web_sys::console::log_1(
                &format!(
                    "[MESSAGES_AREA] invoking MessageCreate for {channel_id} with reply_to={reply_to:?}"
                )
                .into(),
            );
            let res = WsService::message_create(channel_id, sanitized, reply_to, None).await;
            web_sys::console::log_1(&format!("[MESSAGES_AREA] MessageCreate res: {res:?}").into());
        });

        if let Some(node) = container.as_ref().and_then(|c| c.try_get()) {
            let el = node.unchecked_into::<HtmlElement>();
            el.set_scroll_top(el.scroll_height());
        }
    }

    draft_message.set(String::new());
}
