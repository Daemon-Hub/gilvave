//! Action status bars displayed above the chat input (replying, editing, and pending attachment islands).

use crate::components::{
    common::UiModalContext,
    ui::icons::{CloseSmallIcon, EditIcon, FileTextIcon, ImageIcon, TrashIcon, VideoIcon},
};
use sycamore::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

#[derive(Clone, PartialEq, Eq)]
enum ConfirmTarget {
    SingleItem {
        id: String,
        name: String,
        is_video: bool,
        is_file: bool,
        existing_file_id: Option<gilvave_core::ids::FileId>,
    },
    GroupPhotos,
    GroupVideos,
    GroupFiles,
    CancelEdit,
}

#[component]
pub(crate) fn ActionIslands() -> View {
    let modal_context = use_context::<UiModalContext>();
    let confirm_target = create_signal(None::<ConfirmTarget>);

    view! {
        (move || {
            let reply_opt = modal_context.replying_to.get_clone();
            let is_editing = modal_context.editing_message.with(|e| e.is_some());
            let attachments = modal_context.pending_attachments.get_clone();

            let has_reply = reply_opt.is_some();
            let has_photos = attachments.iter().any(|a| a.is_previewable && !a.is_video);
            let has_videos = attachments.iter().any(|a| a.is_previewable && a.is_video);
            let has_files = attachments.iter().any(|a| !a.is_previewable);

            if !has_reply && !is_editing && !has_photos && !has_videos && !has_files {
                return view! {};
            }

            let reply_view = if let Some(reply_msg) = reply_opt {
                let author = reply_msg.author_name;
                let snippet = if reply_msg.content.chars().count() > 60 {
                    let s: String = reply_msg.content.chars().take(60).collect();
                    format!("{s}…")
                } else {
                    reply_msg.content
                };
                view! {
                    div(class="chat-action-banner chat-reply-banner") {
                        div(class="banner-spine")
                        div(class="banner-info") {
                            span(class="banner-title") { (format!("Ответ пользователю @{author}")) }
                            span(class="banner-snippet") { (snippet) }
                        }
                        button(
                            r#type="button",
                            class="banner-close-btn",
                            title="Отменить ответ (Esc)",
                            on:click=move |_| modal_context.replying_to.set(None),
                        ) {
                            CloseSmallIcon()
                        }
                    }
                }
            } else {
                view! {}
            };

            let edit_view = if is_editing {
                view! {
                    div(class="chat-edit-island") {
                        div(class="island-edit-icon") {
                            EditIcon()
                        }
                        span(class="island-title") { "Редактирование" }
                        button(
                            r#type="button",
                            class="island-close-btn",
                            title="Отменить редактирование",
                            on:click=move |_| {
                                confirm_target.set(Some(ConfirmTarget::CancelEdit));
                            },
                        ) {
                            CloseSmallIcon()
                        }
                    }
                }
            } else {
                view! {}
            };

            // 1. Остров фотографий
            let photo_items: Vec<_> = attachments.iter().filter(|a| a.is_previewable && !a.is_video).cloned().collect();
            let photos_view = if !photo_items.is_empty() {
                let cards: Vec<View> = photo_items.into_iter().map(|att| {
                    let att_id = att.id.clone();
                    let card_title = att.name.clone();
                    let img_alt = att.name.clone();
                    let preview_url = att.preview_url.clone();
                    let existing_fid = att.existing_file_id;

                    let url_click = preview_url.clone();
                    let name_click = card_title.clone();
                    let title_detach = card_title.clone();
                    let id_detach = att_id.clone();

                    view! {
                        div(
                            class="attachment-thumb-card photo-thumb-card",
                            title=card_title,
                            on:click=move |_| {
                                if let Some(url) = &url_click {
                                    modal_context.lightbox_image.set(Some((url.clone(), name_click.clone())));
                                }
                            },
                        ) {
                            (if let Some(url) = preview_url {
                                view! {
                                    img(class="thumb-media-img", src=url, alt=img_alt)
                                }
                            } else {
                                view! {
                                    div(class="thumb-media-placeholder") { "IMG" }
                                }
                            })
                            button(
                                r#type="button",
                                class="card-detach-btn",
                                title="Открепить",
                                on:click=move |e: web_sys::MouseEvent| {
                                    e.stop_propagation();
                                    confirm_target.set(Some(ConfirmTarget::SingleItem {
                                        id: id_detach.clone(),
                                        name: title_detach.clone(),
                                        is_video: false,
                                        is_file: false,
                                        existing_file_id: existing_fid,
                                    }));
                                },
                            ) {
                                CloseSmallIcon()
                            }
                        }
                    }
                }).collect();

                view! {
                    div(class="chat-action-island chat-photos-island") {
                        div(class="island-badge-icon photo-glow", title="Изображения") {
                            ImageIcon()
                        }
                        div(class="island-items-scroll") {
                            (cards)
                        }
                        button(
                            r#type="button",
                            class="island-group-close-btn",
                            title="Открепить все изображения",
                            on:click=move |_| {
                                confirm_target.set(Some(ConfirmTarget::GroupPhotos));
                            },
                        ) {
                            CloseSmallIcon()
                        }
                    }
                }
            } else {
                view! {}
            };

            // 2. Остров видео
            let video_items: Vec<_> = attachments.iter().filter(|a| a.is_previewable && a.is_video).cloned().collect();
            let videos_view = if !video_items.is_empty() {
                let cards: Vec<View> = video_items.into_iter().map(|att| {
                    let att_id = att.id.clone();
                    let card_title = att.name.clone();
                    let preview_url = att.preview_url.clone();
                    let existing_fid = att.existing_file_id;

                    let url_click = preview_url.clone();
                    let name_click = card_title.clone();
                    let title_detach = card_title.clone();
                    let id_detach = att_id.clone();

                    view! {
                        div(
                            class="attachment-thumb-card video-thumb-card",
                            title=card_title,
                            on:click=move |_| {
                                if let Some(url) = &url_click {
                                    modal_context.lightbox_image.set(Some((url.clone(), name_click.clone())));
                                }
                            },
                        ) {
                            (if let Some(url) = preview_url {
                                view! {
                                    div(class="thumb-video-wrapper") {
                                        video(class="thumb-media-video", src=url, muted=true, playsinline=true)
                                        div(class="thumb-play-overlay") {
                                            div(class="thumb-play-triangle")
                                        }
                                    }
                                }
                            } else {
                                view! {
                                    div(class="thumb-video-placeholder") { "MP4" }
                                }
                            })
                            button(
                                r#type="button",
                                class="card-detach-btn",
                                title="Открепить",
                                on:click=move |e: web_sys::MouseEvent| {
                                    e.stop_propagation();
                                    confirm_target.set(Some(ConfirmTarget::SingleItem {
                                        id: id_detach.clone(),
                                        name: title_detach.clone(),
                                        is_video: true,
                                        is_file: false,
                                        existing_file_id: existing_fid,
                                    }));
                                },
                            ) {
                                CloseSmallIcon()
                            }
                        }
                    }
                }).collect();

                view! {
                    div(class="chat-action-island chat-videos-island") {
                        div(class="island-badge-icon video-glow", title="Видео") {
                            VideoIcon()
                        }
                        div(class="island-items-scroll") {
                            (cards)
                        }
                        button(
                            r#type="button",
                            class="island-group-close-btn",
                            title="Открепить все видео",
                            on:click=move |_| {
                                confirm_target.set(Some(ConfirmTarget::GroupVideos));
                            },
                        ) {
                            CloseSmallIcon()
                        }
                    }
                }
            } else {
                view! {}
            };

            // 3. Остров файлов и документов
            let file_items: Vec<_> = attachments.iter().filter(|a| !a.is_previewable).cloned().collect();
            let files_view = if !file_items.is_empty() {
                let cards: Vec<View> = file_items.into_iter().map(|att| {
                    let att_id = att.id.clone();
                    let card_title = att.name.clone();
                    let file_name = att.name.clone();
                    let ext = att.file_extension();
                    let size_str = att.formatted_size();
                    let existing_fid = att.existing_file_id;

                    let title_detach = card_title.clone();
                    let id_detach = att_id.clone();

                    view! {
                        div(class="attachment-file-card", title=card_title) {
                            div(class="file-card-icon-badge") {
                                span { (ext) }
                            }
                            div(class="file-card-details") {
                                span(class="file-card-name") { (file_name) }
                                span(class="file-card-size") { (size_str) }
                            }
                            button(
                                r#type="button",
                                class="card-detach-btn",
                                title="Открепить",
                                on:click=move |e: web_sys::MouseEvent| {
                                    e.stop_propagation();
                                    confirm_target.set(Some(ConfirmTarget::SingleItem {
                                        id: id_detach.clone(),
                                        name: title_detach.clone(),
                                        is_video: false,
                                        is_file: true,
                                        existing_file_id: existing_fid,
                                    }));
                                },
                            ) {
                                CloseSmallIcon()
                            }
                        }
                    }
                }).collect();

                view! {
                    div(class="chat-action-island chat-files-island") {
                        div(class="island-badge-icon file-glow", title="Файлы") {
                            FileTextIcon()
                        }
                        div(class="island-items-scroll") {
                            (cards)
                        }
                        button(
                            r#type="button",
                            class="island-group-close-btn",
                            title="Открепить все файлы",
                            on:click=move |_| {
                                confirm_target.set(Some(ConfirmTarget::GroupFiles));
                            },
                        ) {
                            CloseSmallIcon()
                        }
                    }
                }
            } else {
                view! {}
            };

            let islands_bar = if is_editing || has_photos || has_videos || has_files {
                view! {
                    div(class="chat-action-islands-bar") {
                        (edit_view)
                        (photos_view)
                        (videos_view)
                        (files_view)
                    }
                }
            } else {
                view! {}
            };

            view! {
                div(class="chat-action-container") {
                    (reply_view)
                    (islands_bar)
                }
            }
        })

        // Модальное окно подтверждения удаления/открепления
        (move || {
            if let Some(target) = confirm_target.get_clone() {
                let title = match &target {
                    ConfirmTarget::SingleItem { is_video: true, .. } => "Открепить видео?",
                    ConfirmTarget::SingleItem { is_file: true, .. } => "Открепить файл?",
                    ConfirmTarget::SingleItem { .. } => "Открепить изображение?",
                    ConfirmTarget::GroupPhotos => "Открепить все изображения?",
                    ConfirmTarget::GroupVideos => "Открепить все видео?",
                    ConfirmTarget::GroupFiles => "Открепить все файлы?",
                    ConfirmTarget::CancelEdit => "Отменить редактирование?",
                };
                let desc = match &target {
                    ConfirmTarget::SingleItem { name, is_video: true, .. } => {
                        format!("Вы уверены, что хотите открепить видео «{name}»?")
                    }
                    ConfirmTarget::SingleItem { name, is_file: true, .. } => {
                        format!("Вы уверены, что хотите открепить файл «{name}»?")
                    }
                    ConfirmTarget::SingleItem { name, .. } => {
                        format!("Вы уверены, что хотите открепить изображение «{name}»?")
                    }
                    ConfirmTarget::GroupPhotos => "Все прикреплённые изображения будут удалены из сообщения.".to_string(),
                    ConfirmTarget::GroupVideos => "Все прикреплённые видеофайлы будут удалены из сообщения.".to_string(),
                    ConfirmTarget::GroupFiles => "Все прикреплённые файлы и документы будут удалены из сообщения.".to_string(),
                    ConfirmTarget::CancelEdit => "Все несохранённые изменения сообщения будут сброшены.".to_string(),
                };
                let confirm_btn_label = match &target {
                    ConfirmTarget::CancelEdit => "Сбросить",
                    _ => "Открепить",
                };

                let target_for_confirm = target.clone();
                view! {
                    div(
                        class="island-confirm-overlay",
                        on:click=move |_| confirm_target.set(None),
                    ) {
                        div(
                            class="island-confirm-card",
                            on:click=move |e: web_sys::MouseEvent| e.stop_propagation(),
                        ) {
                            div(class="island-confirm-header") {
                                div(class="island-confirm-icon-badge") {
                                    TrashIcon()
                                }
                                span(class="island-confirm-title") { (title) }
                            }
                            p(class="island-confirm-desc") { (desc) }
                            div(class="island-confirm-actions") {
                                button(
                                    r#type="button",
                                    class="island-confirm-btn cancel-btn",
                                    on:click=move |_| confirm_target.set(None),
                                ) {
                                    "Отмена"
                                }
                                button(
                                    r#type="button",
                                    class="island-confirm-btn danger-btn",
                                    on:click=move |_| {
                                        match &target_for_confirm {
                                            ConfirmTarget::SingleItem { id, existing_file_id, .. } => {
                                                let tid = id.clone();
                                                let efid = *existing_file_id;
                                                modal_context.pending_attachments.update(|list| {
                                                    if let Some(pos) = list.iter().position(|a| a.id == tid) {
                                                        let removed = list.remove(pos);
                                                        removed.revoke();
                                                    }
                                                });
                                                if let Some(fid) = efid {
                                                    sycamore::futures::spawn_local_scoped(async move {
                                                        let _ = crate::http::api::Api::delete_file(fid).await;
                                                    });
                                                }
                                            }
                                            ConfirmTarget::GroupPhotos => {
                                                let mut files_to_delete = Vec::new();
                                                modal_context.pending_attachments.update(|list| {
                                                    let mut retained = Vec::new();
                                                    for a in list.drain(..) {
                                                        if a.is_previewable && !a.is_video {
                                                            if let Some(fid) = a.existing_file_id {
                                                                files_to_delete.push(fid);
                                                            }
                                                            a.revoke();
                                                        } else {
                                                            retained.push(a);
                                                        }
                                                    }
                                                    *list = retained;
                                                });
                                                for fid in files_to_delete {
                                                    sycamore::futures::spawn_local_scoped(async move {
                                                        let _ = crate::http::api::Api::delete_file(fid).await;
                                                    });
                                                }
                                            }
                                            ConfirmTarget::GroupVideos => {
                                                let mut files_to_delete = Vec::new();
                                                modal_context.pending_attachments.update(|list| {
                                                    let mut retained = Vec::new();
                                                    for a in list.drain(..) {
                                                        if a.is_previewable && a.is_video {
                                                            if let Some(fid) = a.existing_file_id {
                                                                files_to_delete.push(fid);
                                                            }
                                                            a.revoke();
                                                        } else {
                                                            retained.push(a);
                                                        }
                                                    }
                                                    *list = retained;
                                                });
                                                for fid in files_to_delete {
                                                    sycamore::futures::spawn_local_scoped(async move {
                                                        let _ = crate::http::api::Api::delete_file(fid).await;
                                                    });
                                                }
                                            }
                                            ConfirmTarget::GroupFiles => {
                                                let mut files_to_delete = Vec::new();
                                                modal_context.pending_attachments.update(|list| {
                                                    let mut retained = Vec::new();
                                                    for a in list.drain(..) {
                                                        if !a.is_previewable {
                                                            if let Some(fid) = a.existing_file_id {
                                                                files_to_delete.push(fid);
                                                            }
                                                            a.revoke();
                                                        } else {
                                                            retained.push(a);
                                                        }
                                                    }
                                                    *list = retained;
                                                });
                                                for fid in files_to_delete {
                                                    sycamore::futures::spawn_local_scoped(async move {
                                                        let _ = crate::http::api::Api::delete_file(fid).await;
                                                    });
                                                }
                                            }
                                            ConfirmTarget::CancelEdit => {
                                                modal_context.editing_message.set(None);
                                                modal_context.draft_message.set(String::new());
                                                modal_context.pending_attachments.update(|list| {
                                                    for a in list.drain(..) {
                                                        a.revoke();
                                                    }
                                                });
                                                #[cfg(target_arch = "wasm32")]
                                                if let Some(window) = web_sys::window() {
                                                    if let Some(doc) = window.document() {
                                                        if let Ok(Some(editor)) = doc.query_selector(".chat-input") {
                                                            if let Ok(html_el) = editor.dyn_into::<web_sys::HtmlElement>() {
                                                                crate::components::common::sync_editor_content(&html_el, "");
                                                                let _ = html_el.set_attribute("data-empty", "true");
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        confirm_target.set(None);
                                    },
                                ) {
                                    (confirm_btn_label)
                                }
                            }
                        }
                    }
                }
            } else {
                view! {}
            }
        })
    }
}
