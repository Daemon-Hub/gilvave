//! Logic for sending, updating, and dispatching channel messages with deferred attachment uploads.

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

    if modal_context.is_uploading_attachment.get() {
        return;
    }

    let raw_text = draft_message.get_clone();
    let attachments = modal_context.pending_attachments.get_clone();

    // Check message length limit on typed text
    let typed_chars = gilvave_core::validation::count_typed_chars(&raw_text);
    if typed_chars > gilvave_core::validation::MAX_MESSAGE_CHARS {
        crate::components::ui::context_menu::clipboard::show_toast(
            &modal_context,
            true,
            format!(
                "Превышен лимит символов: {} / {}",
                typed_chars,
                gilvave_core::validation::MAX_MESSAGE_CHARS
            ),
        );
        return;
    }

    // If no attachments and no text, nothing to send
    if attachments.is_empty() && raw_text.trim().is_empty() {
        return;
    }

    let editing = modal_context.editing_message.get_clone();
    let replying = modal_context.replying_to.get_clone();

    spawn_local_scoped(async move {
        let mut final_content = raw_text.trim().to_string();

        // 1. Deferred upload of pending attachments (only executed when Send is clicked!)
        if !attachments.is_empty() {
            modal_context.is_uploading_attachment.set(true);
            let mut uploaded_snippets = Vec::with_capacity(attachments.len());
            let mut upload_err: Option<String> = None;

            for att in &attachments {
                if let Some(existing_url) = &att.existing_url {
                    // Already on server (retained during message edit)
                    if att.is_previewable {
                        uploaded_snippets.push(format!("![{}]({})", att.name, existing_url));
                    } else if let Some(fid) = att.existing_file_id {
                        uploaded_snippets.push(format!("[{}](file:{})", att.name, fid));
                    } else {
                        uploaded_snippets.push(format!("[{}]({})", att.name, existing_url));
                    }
                } else if let Some(file) = &att.file {
                    // New local file upload
                    match crate::http::api::Api::upload_file(file).await {
                        Ok(res) => {
                            let file_url = if res.url.starts_with('/') {
                                format!("{}{}", gilvave_core::settings::BASE_HTTP_URL, res.url)
                            } else {
                                res.url
                            };

                            // Transparently cache the uploaded file locally so the user never has to re-download it
                            crate::components::common::media::cache_uploaded_file_storage(
                                &res.id.to_string(),
                                &res.name,
                                file,
                            )
                            .await;

                            if res.is_previewable {
                                uploaded_snippets.push(format!("![{}]({})", res.name, file_url));
                            } else {
                                uploaded_snippets.push(format!("[{}](file:{})", res.name, res.id));
                            }
                        }
                        Err(err) => {
                            upload_err = Some(format!("Ошибка загрузки «{}»: {}", att.name, err.1));
                            break;
                        }
                    }
                }
            }

            modal_context.is_uploading_attachment.set(false);

            if let Some(err_msg) = upload_err {
                crate::components::ui::context_menu::clipboard::show_toast(
                    &modal_context,
                    true,
                    err_msg,
                );
                return;
            }

            // All attachments uploaded successfully — format combined content
            if !uploaded_snippets.is_empty() {
                let snippets_block = uploaded_snippets.join("\n\n");
                if final_content.is_empty() {
                    final_content = snippets_block;
                } else {
                    final_content = format!("{final_content}\n\n{snippets_block}");
                }
            }

            // Clean up blob URLs
            for att in &attachments {
                att.revoke();
            }
            modal_context.pending_attachments.set(Vec::new());
        }

        // Validate final combined content
        let sanitized = match gilvave_core::validation::validate_message(&final_content) {
            Ok(s) => s,
            Err(e) => {
                web_sys::console::warn_1(&format!("[MESSAGES_AREA] on_submit validation failed: {e}").into());
                return;
            }
        };

        if let Some(edit_msg) = editing {
            web_sys::console::log_1(
                &format!("[MESSAGES_AREA] invoking MessageUpdate for {}", edit_msg.id).into(),
            );
            let res = WsService::message_update(edit_msg.channel_id, edit_msg.id, sanitized).await;
            web_sys::console::log_1(&format!("[MESSAGES_AREA] MessageUpdate res: {res:?}").into());
            modal_context.editing_message.set(None);
        } else {
            let reply_to = replying.as_ref().map(|r| r.id);
            modal_context.replying_to.set(None);

            save_channel_last_read(
                channel_id,
                MessageId::default(),
                time::OffsetDateTime::now_utc().unix_timestamp(),
            );

            web_sys::console::log_1(
                &format!(
                    "[MESSAGES_AREA] invoking MessageCreate for {channel_id} with reply_to={reply_to:?}"
                )
                .into(),
            );
            let res = WsService::message_create(channel_id, sanitized, reply_to, None).await;
            web_sys::console::log_1(&format!("[MESSAGES_AREA] MessageCreate res: {res:?}").into());

            if let Some(node) = container.as_ref().and_then(|c| c.try_get()) {
                let el = node.unchecked_into::<HtmlElement>();
                el.set_scroll_top(el.scroll_height());
            }
        }

        draft_message.set(String::new());
    });
}
