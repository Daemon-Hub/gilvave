//! Logic for sending, updating, and dispatching channel messages with deferred attachment uploads.

use futures_util::future::join_all;
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

    // 1. Редактирование существующего сообщения
    if let Some(edit_msg) = editing {
        spawn_local_scoped(async move {
            let mut final_content = raw_text.trim().to_string();

            if !attachments.is_empty() {
                modal_context.is_uploading_attachment.set(true);

                let upload_futures = attachments.iter().map(|att| {
                    let att = att.clone();
                    async move {
                        if let Some(existing_url) = &att.existing_url {
                            if att.is_previewable {
                                Ok(format!("![{}]({})", att.name, existing_url))
                            } else if let Some(fid) = att.existing_file_id {
                                Ok(format!("[{}](file:{})", att.name, fid))
                            } else {
                                Ok(format!("[{}]({})", att.name, existing_url))
                            }
                        } else if let Some(file) = &att.file {
                            match crate::http::api::Api::upload_file(file).await {
                                Ok(res) => {
                                    let file_url = if res.url.starts_with('/') {
                                        format!("{}{}", gilvave_core::settings::BASE_HTTP_URL, res.url)
                                    } else {
                                        res.url
                                    };
                                    crate::components::common::media::cache_uploaded_file_storage(
                                        &res.id.to_string(),
                                        &res.name,
                                        file,
                                    )
                                    .await;
                                    if res.is_previewable {
                                        Ok(format!("![{}]({})", res.name, file_url))
                                    } else {
                                        Ok(format!("[{}](file:{})", res.name, res.id))
                                    }
                                }
                                Err(err) => Err(format!("Ошибка загрузки «{}»: {}", att.name, err.1)),
                            }
                        } else {
                            Ok(String::new())
                        }
                    }
                });

                let results = join_all(upload_futures).await;
                modal_context.is_uploading_attachment.set(false);

                let mut uploaded_snippets = Vec::with_capacity(results.len());
                let mut upload_err: Option<String> = None;
                for res in results {
                    match res {
                        Ok(snip) if !snip.is_empty() => uploaded_snippets.push(snip),
                        Ok(_) => {}
                        Err(err) => {
                            if upload_err.is_none() {
                                upload_err = Some(err);
                            }
                        }
                    }
                }

                if let Some(err_msg) = upload_err {
                    crate::components::ui::context_menu::clipboard::show_toast(
                        &modal_context,
                        true,
                        err_msg,
                    );
                    return;
                }

                if !uploaded_snippets.is_empty() {
                    let snippets_block = uploaded_snippets.join("\n\n");
                    if final_content.is_empty() {
                        final_content = snippets_block;
                    } else {
                        final_content = format!("{final_content}\n\n{snippets_block}");
                    }
                }

                for att in &attachments {
                    att.revoke();
                }
                modal_context.pending_attachments.set(Vec::new());
            }

            let sanitized = match gilvave_core::validation::validate_message(&final_content) {
                Ok(s) => s,
                Err(e) => {
                    web_sys::console::warn_1(&format!("[MESSAGES_AREA] edit validation failed: {e}").into());
                    return;
                }
            };

            let _ = WsService::message_update(edit_msg.channel_id, edit_msg.id, sanitized).await;
            modal_context.editing_message.set(None);
            draft_message.set(String::new());
        });
        return;
    }

    // 2. Отправка нового сообщения (оптимистичный UI с мгновенным появлением в чате)
    let optimistic_id = MessageId::default();
    let reply_to = replying.as_ref().map(|r| r.id);
    modal_context.replying_to.set(None);

    let mut optimistic_snippets = Vec::with_capacity(attachments.len());
    for att in &attachments {
        let att_url = att.preview_url.as_deref().unwrap_or("");
        if let Some(existing_url) = &att.existing_url {
            if att.is_previewable {
                optimistic_snippets.push(format!("![{}]({})", att.name, existing_url));
            } else if let Some(fid) = att.existing_file_id {
                optimistic_snippets.push(format!("[{}](file:{})", att.name, fid));
            } else {
                optimistic_snippets.push(format!("[{}]({})", att.name, existing_url));
            }
        } else if att.is_previewable {
            optimistic_snippets.push(format!("![{}]({})", att.name, att_url));
        } else {
            optimistic_snippets.push(format!("[{}]({})", att.name, att_url));
        }
    }

    let mut optimistic_content = raw_text.trim().to_string();
    if !optimistic_snippets.is_empty() {
        let snippets_block = optimistic_snippets.join("\n\n");
        if optimistic_content.is_empty() {
            optimistic_content = snippets_block;
        } else {
            optimistic_content = format!("{optimistic_content}\n\n{snippets_block}");
        }
    }

    let optimistic_msg = gilvave_core::dto::messages::MessageView {
        id: optimistic_id,
        channel_id,
        author_id: _user_profile.user_id.get(),
        author_name: _user_profile.username.get_clone(),
        content: optimistic_content,
        edited: false,
        reply_to_id: reply_to,
        forwarded_from: None,
        created_at: time::OffsetDateTime::now_utc(),
        is_pending: true,
        is_failed: false,
    };

    // Мгновенно добавляем сообщение в чат
    channel_context.messages.update(|list| {
        list.push(optimistic_msg);
    });

    // Мгновенно очищаем поле ввода и список прикрепленных файлов
    draft_message.set(String::new());
    modal_context.pending_attachments.set(Vec::new());

    // Мгновенно синхронизируем DOM поле ввода
    #[cfg(target_arch = "wasm32")]
    if let Some(window) = web_sys::window() {
        if let Some(doc) = window.document() {
            if let Ok(Some(editor)) = doc.query_selector(".chat-input") {
                editor.set_text_content(Some(""));
                let _ = editor.set_attribute("data-empty", "true");
            }
        }
    }

    // Прокручиваем к новому сообщению с поддержкой rAF после пересчета высоты DOM
    if let Some(node) = container.as_ref().and_then(|c| c.try_get()) {
        let el = node.unchecked_into::<HtmlElement>();
        el.set_scroll_top(el.scroll_height());
        #[cfg(target_arch = "wasm32")]
        if let Some(w) = web_sys::window() {
            let el_clone = el.clone();
            let cb = wasm_bindgen::closure::Closure::once_into_js(move || {
                el_clone.set_scroll_top(el_clone.scroll_height());
            });
            let _ = w.request_animation_frame(cb.as_ref().unchecked_ref());
        }
    }

    let ch_ctx = channel_context;
    let m_ctx = modal_context;
    spawn_local_scoped(async move {
        // Уступаем квант времени (20 мс) браузеру для отрисовки оптимистичного сообщения и спинеров загрузки
        gloo_timers::future::sleep(core::time::Duration::from_millis(20)).await;

        let mut uploaded_snippets = Vec::with_capacity(attachments.len());
        let mut upload_err: Option<String> = None;

        if !attachments.is_empty() {
            m_ctx.is_uploading_attachment.set(true);

            // Параллельная загрузка всех прикрепленных файлов без последовательного ожидания
            let upload_futures = attachments.iter().map(|att| {
                let att = att.clone();
                async move {
                    if let Some(existing_url) = &att.existing_url {
                        if att.is_previewable {
                            Ok(format!("![{}]({})", att.name, existing_url))
                        } else if let Some(fid) = att.existing_file_id {
                            Ok(format!("[{}](file:{})", att.name, fid))
                        } else {
                            Ok(format!("[{}]({})", att.name, existing_url))
                        }
                    } else if let Some(file) = &att.file {
                        match crate::http::api::Api::upload_file(file).await {
                            Ok(res) => {
                                let file_url = if res.url.starts_with('/') {
                                    format!("{}{}", gilvave_core::settings::BASE_HTTP_URL, res.url)
                                } else {
                                    res.url
                                };

                                crate::components::common::media::cache_uploaded_file_storage(
                                    &res.id.to_string(),
                                    &res.name,
                                    file,
                                )
                                .await;

                                if res.is_previewable {
                                    Ok(format!("![{}]({})", res.name, file_url))
                                } else {
                                    Ok(format!("[{}](file:{})", res.name, res.id))
                                }
                            }
                            Err(err) => Err(format!("Ошибка загрузки «{}»: {}", att.name, err.1)),
                        }
                    } else {
                        Ok(String::new())
                    }
                }
            });

            let results = join_all(upload_futures).await;
            m_ctx.is_uploading_attachment.set(false);

            for res in results {
                match res {
                    Ok(snippet) => {
                        if !snippet.is_empty() {
                            uploaded_snippets.push(snippet);
                        }
                    }
                    Err(err) => {
                        if upload_err.is_none() {
                            upload_err = Some(err);
                        }
                    }
                }
            }

            if let Some(err_msg) = upload_err {
                ch_ctx.messages.update(|list| {
                    if let Some(m) = list.iter_mut().find(|m| m.id == optimistic_id) {
                        m.is_pending = false;
                        m.is_failed = true;
                    }
                });
                crate::components::ui::context_menu::clipboard::show_toast(
                    &m_ctx,
                    true,
                    err_msg,
                );
                return;
            }

            for att in &attachments {
                att.revoke();
            }
        }

        let mut final_content = raw_text.trim().to_string();
        if !uploaded_snippets.is_empty() {
            let snippets_block = uploaded_snippets.join("\n\n");
            if final_content.is_empty() {
                final_content = snippets_block;
            } else {
                final_content = format!("{final_content}\n\n{snippets_block}");
            }
        }

        let sanitized = match gilvave_core::validation::validate_message(&final_content) {
            Ok(s) => s,
            Err(e) => {
                web_sys::console::warn_1(&format!("[MESSAGES_AREA] validation failed: {e}").into());
                ch_ctx.messages.update(|list| {
                    if let Some(m) = list.iter_mut().find(|m| m.id == optimistic_id) {
                        m.is_pending = false;
                        m.is_failed = true;
                    }
                });
                return;
            }
        };

        save_channel_last_read(
            channel_id,
            MessageId::default(),
            time::OffsetDateTime::now_utc().unix_timestamp(),
        );

        let res = WsService::message_create(channel_id, sanitized, reply_to, None).await;
        if res.is_err() {
            ch_ctx.messages.update(|list| {
                if let Some(m) = list.iter_mut().find(|m| m.id == optimistic_id) {
                    m.is_pending = false;
                    m.is_failed = true;
                }
            });
        }
    });
}
