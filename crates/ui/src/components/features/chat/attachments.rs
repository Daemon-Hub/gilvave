//! Native Sycamore components for message attachments: Media Gallery, Video Player, and File Cards.

use gilvave_core::dto::command::{CommandArgs, CommandResponse, CommandResult};
use sycamore::{futures::spawn_local_scoped, prelude::*};
use wasm_bindgen::JsCast;

use crate::components::common::{
    media::{open_file_card_async, parse_file_id_from_url, ActiveFullscreenVideo, ParsedAttachment},
    UiModalContext,
};
use crate::utils::{invoke_command, is_tauri};

fn format_time(secs: f64) -> String {
    if secs.is_nan() || secs.is_infinite() || secs < 0.0 {
        return "0:00".to_string();
    }
    let total = secs.floor() as u64;
    let s = total % 60;
    let m = (total / 60) % 60;
    let h = total / 3600;
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

/// Индивидуальное изображение сообщения с локальным дисковым кешированием,
/// размытой карточкой-заглушкой и крутящимся спиннером во время загрузки.
#[component(inline_props)]
pub fn ChatMessageImage(
    name: String,
    url: String,
    modal_ctx: UiModalContext,
) -> View {
    let file_id = parse_file_id_from_url(&url)
        .map(|id| id.to_string())
        .unwrap_or_else(|| {
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            url.hash(&mut hasher);
            format!("{:016x}", hasher.finish())
        });

    let canonical_url = if url.starts_with("http://") || url.starts_with("https://") {
        url.clone()
    } else if let Some(fid) = parse_file_id_from_url(&url) {
        format!("{}/files/{}", gilvave_core::settings::BASE_HTTP_URL, fid)
    } else if url.starts_with('/') {
        format!("{}{}", gilvave_core::settings::BASE_HTTP_URL, url)
    } else {
        url.clone()
    };

    let is_desktop = is_tauri();
    let is_loaded = create_signal(false);
    let is_error = create_signal(false);
    let active_src = create_signal(if !is_desktop || url.starts_with("data:") || url.starts_with("blob:") {
        canonical_url.clone()
    } else {
        String::new()
    });

    if is_desktop && !url.starts_with("data:") && !url.starts_with("blob:") {
        let fid = file_id.clone();
        let fname = name.clone();
        let durl = canonical_url.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let cmd = CommandArgs::GetCachedMedia {
                file_id: fid.clone(),
                filename: fname,
                download_url: Some(durl.clone()),
            };
            match invoke_command(cmd.to_json()).await {
                CommandResult::Ok(CommandResponse::GetCachedMedia(Some(data_url))) => {
                    active_src.set(data_url);
                }
                _ => {
                    // Fallback to direct HTTP URL instead of premature erroring
                    if durl.starts_with("http://") || durl.starts_with("https://") {
                        active_src.set(durl);
                    } else if !fid.is_empty() {
                        active_src.set(format!("{}/files/{}", gilvave_core::settings::BASE_HTTP_URL, fid));
                    } else {
                        is_error.set(true);
                    }
                }
            }
        });
    }

    let on_img_load = move |_e: web_sys::Event| {
        if !is_loaded.get_untracked() {
            is_loaded.set(true);
        }
        if is_error.get_untracked() {
            is_error.set(false);
        }
        #[cfg(target_arch = "wasm32")]
        if let Some(target) = _e.current_target() {
            if let Ok(el) = target.dyn_into::<web_sys::HtmlElement>() {
                if let Some(closest_list) = el.closest(".messages-list").ok().flatten() {
                    if let Ok(list_el) = closest_list.dyn_into::<web_sys::HtmlElement>() {
                        let sh = list_el.scroll_height();
                        let ch = list_el.client_height();
                        let st = list_el.scroll_top();
                        if (sh - ch - st).abs() <= 120 {
                            list_el.set_scroll_top(sh - ch);
                        }
                    }
                }
            }
        }
    };

    let on_img_error = move |_: web_sys::ErrorEvent| {
        if active_src.with_untracked(|s| !s.is_empty()) && !is_error.get_untracked() {
            is_error.set(true);
        }
    };

    let name_click = name.clone();
    let canonical_for_click = canonical_url.clone();
    let fid_retry = file_id.clone();
    let on_img_click = move |e: web_sys::MouseEvent| {
        e.stop_propagation();
        if is_error.get_untracked() {
            let u = canonical_for_click.clone();
            let n = name_click.clone();
            let fid = fid_retry.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let cmd = CommandArgs::GetCachedMedia {
                    file_id: fid,
                    filename: n,
                    download_url: Some(u.clone()),
                };
                match invoke_command(cmd.to_json()).await {
                    CommandResult::Ok(CommandResponse::GetCachedMedia(Some(data_url))) => {
                        active_src.set(data_url);
                        is_loaded.set(true);
                        is_error.set(false);
                    }
                    _ => {
                        if u.starts_with("http://") || u.starts_with("https://") {
                            active_src.set(u);
                        }
                    }
                }
            });
        }
        let effective_src = active_src.with(|s| {
            if !s.is_empty() {
                s.clone()
            } else {
                canonical_for_click.clone()
            }
        });
        modal_ctx
            .lightbox_image
            .set(Some((effective_src, name_click.clone())));
    };

    let name_display = name.clone();
    let name_img = name.clone();

    view! {
        div(
            class=move || {
                if is_error.get() {
                    "chat-media-wrapper error"
                } else if is_loaded.get() {
                    "chat-media-wrapper loaded"
                } else {
                    "chat-media-wrapper loading"
                }
            },
            on:click=on_img_click,
        ) {
            (move || {
                if is_error.get() {
                    let lbl = name_display.clone();
                    view! {
                        div(class="chat-media-placeholder chat-media-error-card") {
                            div(class="chat-media-error-icon") {
                                svg(
                                    viewBox="0 0 24 24",
                                    fill="none",
                                    stroke="currentColor",
                                    "stroke-width"="2",
                                    "stroke-linecap"="round",
                                    "stroke-linejoin"="round",
                                    class="chat-media-offline-svg",
                                ) {
                                    rect(x="3", y="3", width="18", height="18", rx="2", ry="2")
                                    circle(cx="8.5", cy="8.5", r="1.5")
                                    polyline(points="21 15 16 10 5 21")
                                    line(x1="2", y1="2", x2="22", y2="22")
                                }
                            }
                            span(class="chat-media-error-title") { (lbl) }
                            span(class="chat-media-error-badge") { "Файл недоступен офлайн" }
                        }
                    }
                } else if !is_loaded.get() {
                    view! {
                        div(class="chat-media-placeholder chat-media-loading-card") {
                            div(class="chat-media-spinner")
                            span(class="chat-media-loading-text") { "Загрузка..." }
                        }
                    }
                } else {
                    view! {}
                }
            })

            img(
                class=move || {
                    if is_loaded.get() {
                        "chat-message-image loaded"
                    } else {
                        "chat-message-image hidden"
                    }
                },
                src=move || active_src.get_clone(),
                alt=name_img.clone(),
                loading="lazy",
                decoding="async",
                on:load=on_img_load,
                on:error=on_img_error,
            )
        }
    }
}

/// Корневой контейнер отображения вложений сообщения
#[component(inline_props)]
pub fn MessageAttachmentsView(
    attachments: Vec<ParsedAttachment>,
    modal_ctx: UiModalContext,
    #[prop(default = false)]
    is_pending: bool,
    #[prop(default = false)]
    is_failed: bool,
) -> View {
    let mut media_items = Vec::new();
    let mut file_items = Vec::new();

    for att in attachments {
        match att {
            ParsedAttachment::Image { name, url } => {
                media_items.push((name, url, false));
            }
            ParsedAttachment::Video { name, url } => {
                media_items.push((name, url, true));
            }
            ParsedAttachment::File { name, file_id, url } => {
                let fid_str = file_id.map(|id| id.to_string()).unwrap_or_default();
                file_items.push((name, fid_str, url));
            }
        }
    }

    let media_node = if !media_items.is_empty() {
        let is_multi = media_items.len() > 1;
        let gallery_class = if is_multi {
            "chat-media-attachment chat-message-media-gallery multi-media"
        } else {
            "chat-media-attachment chat-message-media-gallery single-media"
        };

        let items_views = media_items
            .into_iter()
            .map(|(name, url, is_vid)| {
                if is_vid {
                    view! {
                        InlineVideoPlayer(
                            url=url,
                            title=name,
                            modal_ctx=modal_ctx,
                        )
                    }
                } else {
                    view! {
                        ChatMessageImage(
                            name=name,
                            url=url,
                            modal_ctx=modal_ctx,
                        )
                    }
                }
            })
            .collect::<Vec<_>>();

        view! {
            div(class=gallery_class) {
                (items_views)
            }
        }
    } else {
        view! {}
    };

    let files_node = if !file_items.is_empty() {
        let file_views = file_items
            .into_iter()
            .map(|(name, fid_str, url)| {
                view! {
                    FileAttachmentCard(
                        name=name,
                        file_id_str=fid_str,
                        url=url,
                        is_pending=is_pending,
                        is_failed=is_failed,
                    )
                }
            })
            .collect::<Vec<_>>();

        view! {
            div(class="chat-files-list") {
                (file_views)
            }
        }
    } else {
        view! {}
    };

    view! {
        div(class="message-attachments-container") {
            (media_node)
            (files_node)
        }
    }
}

/// Вычисляет адаптивные габариты мини-плеера под реальное соотношение сторон видео
fn compute_player_dimensions(vw: u32, vh: u32) -> (f64, f64) {
    if vw == 0 || vh == 0 {
        return (400.0, 225.0);
    }
    let ar = vw as f64 / vh as f64;
    let max_w = 420.0;
    let max_h = 330.0;
    let min_w = 220.0;
    let min_h = 160.0;

    let (mut target_w, mut target_h);

    if ar >= 1.0 {
        target_w = max_w;
        target_h = target_w / ar;
        if target_h > max_h {
            target_h = max_h;
            target_w = target_h * ar;
        }
    } else {
        target_h = (max_h + 30.0).min(380.0);
        target_w = target_h * ar;
        if target_w < min_w {
            target_w = min_w;
            target_h = target_w / ar;
            if target_h > 380.0 {
                target_h = 380.0;
            }
        }
    }

    target_w = target_w.clamp(min_w, max_w);
    target_h = target_h.clamp(min_h, 380.0);

    (target_w.round(), target_h.round())
}

/// Нативный видеоплеер Sycamore для встроенного просмотра в чате
#[component(inline_props)]
pub fn InlineVideoPlayer(
    url: String,
    title: String,
    modal_ctx: UiModalContext,
) -> View {
    let video_ref = create_node_ref();
    let scrubber_ref = create_node_ref();

    let is_playing = create_signal(false);
    let is_muted = create_signal(false);
    let is_scrubbing = create_signal(false);
    let current_time = create_signal(0.0f64);
    let duration = create_signal(0.0f64);
    let video_dims = create_signal::<Option<(u32, u32)>>(None);

    let is_video_loading = create_signal(true);
    let is_video_error = create_signal(false);
    let active_video_src = create_signal(url.clone());

    let video_file_id = parse_file_id_from_url(&url)
        .map(|id| id.to_string())
        .unwrap_or_else(|| {
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            url.hash(&mut hasher);
            format!("{:016x}", hasher.finish())
        });

    let vid_url_fetch = url.clone();
    let vid_name_fetch = title.clone();
    let vid_fid_fetch = video_file_id.clone();
    spawn_local_scoped(async move {
        if is_tauri() {
            let cmd = CommandArgs::GetCachedMedia {
                file_id: vid_fid_fetch,
                filename: vid_name_fetch,
                download_url: Some(vid_url_fetch),
            };
            if let CommandResult::Ok(CommandResponse::GetCachedMedia(Some(data_url))) =
                invoke_command(cmd.to_json()).await
            {
                active_video_src.set(data_url);
                is_video_loading.set(false);
                is_video_error.set(false);
            }
        }
    });

    let url_for_fs = url.clone();
    let title_for_fs = title.clone();
    let title_for_error = title.clone();

    let toggle_play = move || {
        if let Some(video) = video_ref
            .try_get()
            .and_then(|n| n.dyn_into::<web_sys::HtmlVideoElement>().ok())
        {
            if is_playing.get() {
                let _ = video.pause();
            } else {
                let _ = video.play();
            }
        }
    };

    let toggle_mute = move || {
        if let Some(video) = video_ref
            .try_get()
            .and_then(|n| n.dyn_into::<web_sys::HtmlVideoElement>().ok())
        {
            let next_muted = !video.muted();
            video.set_muted(next_muted);
            is_muted.set(next_muted);
        }
    };

    let seek_to_ratio = move |ratio: f64| {
        let video_elem = video_ref
            .try_get()
            .and_then(|n| n.dyn_into::<web_sys::HtmlVideoElement>().ok());
        let mut dur = duration.get();
        if (dur <= 0.0 || dur.is_nan() || dur.is_infinite()) && let Some(v) = &video_elem {
            let vd = v.duration();
            if vd > 0.0 && !vd.is_nan() && !vd.is_infinite() {
                dur = vd;
                duration.set(vd);
            }
        }
        if dur > 0.0 && !dur.is_nan() && !dur.is_infinite() {
            let target_time = (ratio * dur).clamp(0.0, dur);
            current_time.set(target_time);
            if let Some(video) = video_elem {
                video.set_current_time(target_time);
            }
        }
    };

    let on_scrubber_pointer = move |client_x: f64| {
        if let Some(scrubber) = scrubber_ref
            .try_get()
            .and_then(|n| n.dyn_into::<web_sys::Element>().ok())
        {
            let rect = scrubber.get_bounding_client_rect();
            let width = rect.width();
            if width > 0.0 {
                let click_x = (client_x - rect.left()).clamp(0.0, width);
                let ratio = click_x / width;
                seek_to_ratio(ratio);
            }
        }
    };

    let open_fullscreen = move |e: web_sys::MouseEvent| {
        e.stop_propagation();
        let cur = if let Some(video) = video_ref
            .try_get()
            .and_then(|n| n.dyn_into::<web_sys::HtmlVideoElement>().ok())
        {
            let _ = video.pause();
            video.current_time()
        } else {
            current_time.get()
        };

        modal_ctx.active_fullscreen_video.set(Some(ActiveFullscreenVideo {
            url: url_for_fs.clone(),
            title: title_for_fs.clone(),
            initial_time: cur,
        }));
    };

    let check_video_dims = move || {
        if let Some(v) = video_ref
            .try_get()
            .and_then(|n| n.dyn_into::<web_sys::HtmlVideoElement>().ok())
        {
            let vw = v.video_width();
            let vh = v.video_height();
            if vw > 0 && vh > 0 && video_dims.get_untracked() != Some((vw, vh)) {
                video_dims.set(Some((vw, vh)));
            }
            let d = v.duration();
            if d > 0.0 && !d.is_nan() && !d.is_infinite() && duration.get_untracked() <= 0.0 {
                duration.set(d);
            }
        }
    };

    create_effect(move || {
        active_video_src.track();
        check_video_dims();
    });

    let card_class = move || {
        let mut cls = String::from("chat-message-video-card custom-player");
        if is_playing.get() {
            cls.push_str(" is-playing");
        } else {
            cls.push_str(" is-paused");
        }
        if is_muted.get() {
            cls.push_str(" is-muted");
        }
        cls
    };

    let card_style = move || {
        if let Some((vw, vh)) = video_dims.get() {
            let (w, _) = compute_player_dimensions(vw, vh);
            format!("width: {w}px; max-width: 100%;")
        } else {
            "width: 380px; max-width: 100%;".to_string()
        }
    };

    let stage_style = move || {
        if let Some((vw, vh)) = video_dims.get() {
            if vw > 0 && vh > 0 {
                format!("aspect-ratio: {vw} / {vh};")
            } else {
                "aspect-ratio: 16 / 9;".to_string()
            }
        } else {
            "aspect-ratio: 16 / 9;".to_string()
        }
    };

    let fill_style = move || {
        let dur = duration.get();
        let cur = current_time.get();
        let pct = if dur > 0.0 && !dur.is_nan() && !dur.is_infinite() {
            (cur / dur * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        };
        format!("width: {pct:.1}%;")
    };

    let cur_time_str = move || format_time(current_time.get());
    let dur_time_str = move || format_time(duration.get());

    let download_url = url.clone();
    let download_name = title.clone();

    view! {
        div(
            class=card_class(),
            style=card_style(),
            on:mouseup=move |_| is_scrubbing.set(false),
            on:touchend=move |_| is_scrubbing.set(false),
        ) {
            div(
                class="video-stage",
                style=stage_style(),
                on:click=move |e: web_sys::MouseEvent| {
                    e.stop_propagation();
                    toggle_play();
                },
            ) {
                video(
                    ref=video_ref,
                    class="chat-custom-video",
                    src=active_video_src.get_clone(),
                    preload="auto",
                    playsinline=true,
                    on:play=move |_| {
                        check_video_dims();
                        is_playing.set(true);
                    },
                    on:pause=move |_| is_playing.set(false),
                    on:ended=move |_| is_playing.set(false),
                    on:timeupdate=move |_| {
                        if let Some(v) = video_ref
                            .try_get()
                            .and_then(|n| n.dyn_into::<web_sys::HtmlVideoElement>().ok())
                        {
                            let d = v.duration();
                            if duration.get() <= 0.0 && d > 0.0 && !d.is_nan() && !d.is_infinite() {
                                duration.set(d);
                            }
                            if !is_scrubbing.get() && !v.seeking() {
                                current_time.set(v.current_time());
                            }
                        }
                    },
                    on:loadedmetadata=move |_| {
                        check_video_dims();
                        #[cfg(target_arch = "wasm32")]
                        if let Some(v) = video_ref
                            .try_get()
                            .and_then(|n| n.dyn_into::<web_sys::HtmlElement>().ok())
                        {
                            if let Some(closest_list) = v.closest(".messages-list").ok().flatten() {
                                if let Ok(list_el) = closest_list.dyn_into::<web_sys::HtmlElement>() {
                                    let sh = list_el.scroll_height();
                                    let ch = list_el.client_height();
                                    let st = list_el.scroll_top();
                                    if (sh - ch - st).abs() <= 120 {
                                        list_el.set_scroll_top(sh - ch);
                                    }
                                }
                            }
                        }
                    },
                    on:loadeddata=move |_| {
                        check_video_dims();
                        is_video_loading.set(false);
                        is_video_error.set(false);
                    },
                    on:durationchange=move |_| {
                        check_video_dims();
                    },
                    on:canplay=move |_| {
                        check_video_dims();
                        is_video_loading.set(false);
                        is_video_error.set(false);
                    },
                    on:error=move |_| {
                        is_video_loading.set(false);
                        is_video_error.set(true);
                    },
                )
                (move || if is_video_loading.get() {
                    view! {
                        div(class="chat-media-placeholder chat-media-loading-card video-overlay") {
                            div(class="chat-media-spinner")
                            span(class="chat-media-loading-text") { "Загрузка видео..." }
                        }
                    }
                } else if is_video_error.get() {
                    let vid_title = title_for_error.clone();
                    view! {
                        div(class="chat-media-placeholder chat-media-error-card video-overlay") {
                            div(class="chat-media-error-icon") {
                                svg(
                                    viewBox="0 0 24 24",
                                    fill="none",
                                    stroke="currentColor",
                                    "stroke-width"="2",
                                    "stroke-linecap"="round",
                                    "stroke-linejoin"="round",
                                    class="chat-media-offline-svg",
                                ) {
                                    polygon(points="23 7 16 12 23 17 23 7")
                                    rect(x="1", y="5", width="15", height="14", rx="2", ry="2")
                                    line(x1="1", y1="1", x2="23", y2="23")
                                }
                            }
                            span(class="chat-media-error-title") { (vid_title) }
                            span(class="chat-media-error-badge") { "Видео недоступно офлайн" }
                        }
                    }
                } else {
                    view! {}
                })
                (move || if !is_video_loading.get() && !is_video_error.get() {
                    view! {
                        button(
                            r#type="button",
                            class="video-center-play-btn",
                            on:click=move |e: web_sys::MouseEvent| {
                                e.stop_propagation();
                                toggle_play();
                            },
                        ) {
                            svg(class="play-icon", viewBox="0 0 24 24", fill="currentColor") {
                                polygon(points="5 3 19 12 5 21 5 3")
                            }
                        }
                    }
                } else {
                    view! {}
                })
            }
            div(class="video-custom-controls", on:click=move |e: web_sys::MouseEvent| e.stop_propagation()) {
                div(
                    ref=scrubber_ref,
                    class="video-scrubber-track",
                    on:mousedown=move |e: web_sys::MouseEvent| {
                        e.stop_propagation();
                        is_scrubbing.set(true);
                        on_scrubber_pointer(e.client_x() as f64);
                    },
                    on:click=move |e: web_sys::MouseEvent| {
                        e.stop_propagation();
                        on_scrubber_pointer(e.client_x() as f64);
                    },
                    on:mousemove=move |e: web_sys::MouseEvent| {
                        if is_scrubbing.get() {
                            on_scrubber_pointer(e.client_x() as f64);
                        }
                    },
                    on:mouseup=move |_| is_scrubbing.set(false),
                    on:touchstart=move |e: web_sys::TouchEvent| {
                        if let Some(touch) = e.touches().item(0) {
                            is_scrubbing.set(true);
                            on_scrubber_pointer(touch.client_x() as f64);
                        }
                    },
                    on:touchmove=move |e: web_sys::TouchEvent| {
                        if let Some(touch) = e.touches().item(0) {
                            on_scrubber_pointer(touch.client_x() as f64);
                        }
                    },
                    on:touchend=move |_| is_scrubbing.set(false),
                ) {
                    div(class="video-scrubber-fill", style=fill_style()) {
                        div(class="video-scrubber-thumb")
                    }
                }
                div(class="video-controls-row") {
                    div(class="video-controls-left") {
                        button(
                            r#type="button",
                            class="video-btn video-play-toggle",
                            on:click=move |e: web_sys::MouseEvent| {
                                e.stop_propagation();
                                toggle_play();
                            },
                        ) {
                            (move || if is_playing.get() {
                                view! {
                                    svg(class="icon-pause", viewBox="0 0 24 24", fill="currentColor") {
                                        rect(x="6", y="4", width="4", height="16")
                                        rect(x="14", y="4", width="4", height="16")
                                    }
                                }
                            } else {
                                view! {
                                    svg(class="icon-play", viewBox="0 0 24 24", fill="currentColor") {
                                        polygon(points="5 3 19 12 5 21 5 3")
                                    }
                                }
                            })
                        }
                        div(class="video-time-display") {
                            span(class="video-time-current") { (cur_time_str()) }
                            span(class="video-time-divider") { "/" }
                            span(class="video-time-duration") { (dur_time_str()) }
                        }
                    }
                    div(class="video-controls-right") {
                        button(
                            r#type="button",
                            class="video-btn video-volume-toggle",
                            on:click=move |e: web_sys::MouseEvent| {
                                e.stop_propagation();
                                toggle_mute();
                            },
                        ) {
                            (move || if is_muted.get() {
                                view! {
                                    svg(class="icon-volume-muted", viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                                        polygon(points="11 5 6 9 2 9 2 15 6 15 11 19 11 5")
                                        line(x1="23", y1="9", x2="17", y2="15")
                                        line(x1="17", y1="9", x2="23", y2="15")
                                    }
                                }
                            } else {
                                view! {
                                    svg(class="icon-volume-on", viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                                        polygon(points="11 5 6 9 2 9 2 15 6 15 11 19 11 5")
                                        path(d="M19.07 4.93a10 10 0 0 1 0 14.14M15.54 8.46a5 5 0 0 1 0 7.07")
                                    }
                                }
                            })
                        }
                        button(
                            r#type="button",
                            class="video-btn video-fullscreen-toggle",
                            on:click=open_fullscreen,
                        ) {
                            svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                                polyline(points="15 3 21 3 21 9")
                                polyline(points="9 21 3 21 3 15")
                                line(x1="21", y1="3", x2="14", y2="10")
                                line(x1="3", y1="21", x2="10", y2="14")
                            }
                        }
                        a(
                            class="video-btn video-download-link",
                            href=download_url,
                            download=download_name,
                            target="_blank",
                            rel="noopener noreferrer",
                        ) {
                            svg(viewBox="0 0 24 24", width="14", height="14", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                                path(d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4")
                                polyline(points="7 10 12 15 17 10")
                                line(x1="12", y1="15", x2="12", y2="3")
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CyberFileKind {
    Pdf,
    Code,
    Archive,
    Audio,
    Sheet,
    Doc,
    Exe,
    Image,
    Generic,
}

impl CyberFileKind {
    pub fn class_name(self) -> &'static str {
        match self {
            Self::Pdf => "kind-pdf",
            Self::Code => "kind-code",
            Self::Archive => "kind-archive",
            Self::Audio => "kind-audio",
            Self::Sheet => "kind-sheet",
            Self::Doc => "kind-doc",
            Self::Exe => "kind-exe",
            Self::Image => "kind-image",
            Self::Generic => "kind-generic",
        }
    }
}

fn classify_file_kind(ext: &str) -> (CyberFileKind, String) {
    let ext_lower = ext.trim().to_ascii_lowercase();
    let display_ext = if ext_lower.is_empty() {
        "FILE".to_string()
    } else if ext_lower.len() <= 4 {
        ext_lower.to_ascii_uppercase()
    } else {
        match ext_lower.as_str() {
            "archive" => "ZIP".to_string(),
            "document" => "DOC".to_string(),
            "picture" => "IMG".to_string(),
            _ => ext_lower.chars().take(4).collect::<String>().to_ascii_uppercase(),
        }
    };

    let kind = match ext_lower.as_str() {
        "pdf" => CyberFileKind::Pdf,
        "rs" | "ts" | "js" | "py" | "c" | "cpp" | "cs" | "go" | "html" | "css" | "json"
        | "toml" | "yaml" | "yml" | "sql" | "sh" | "bash" | "zsh" | "lua" | "php" | "rb"
        | "java" | "kt" | "swift" | "vue" | "jsx" | "tsx" | "wasm" => CyberFileKind::Code,
        "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | "iso" | "tgz" | "7zip" => {
            CyberFileKind::Archive
        }
        "mp3" | "wav" | "flac" | "ogg" | "m4a" | "aac" | "opus" | "mid" | "midi" | "wma" => {
            CyberFileKind::Audio
        }
        "xls" | "xlsx" | "csv" | "tsv" | "ods" => CyberFileKind::Sheet,
        "doc" | "docx" | "txt" | "rtf" | "odt" | "md" | "log" | "tex" => CyberFileKind::Doc,
        "exe" | "msi" | "apk" | "app" | "deb" | "rpm" | "bin" | "bat" | "cmd" | "ps1" => {
            CyberFileKind::Exe
        }
        "png" | "jpg" | "jpeg" | "webp" | "gif" | "svg" | "bmp" | "ico" | "psd" | "ai"
        | "tiff" => CyberFileKind::Image,
        _ => CyberFileKind::Generic,
    };

    (kind, display_ext)
}

/// Нативная карточка файла в чате
#[component(inline_props)]
pub fn FileAttachmentCard(
    name: String,
    file_id_str: String,
    url: String,
    #[prop(default = false)]
    is_pending: bool,
    #[prop(default = false)]
    is_failed: bool,
) -> View {
    let fid_for_open = file_id_str.clone();
    let name_for_open = name.clone();
    let url_for_open = url.clone();

    let is_uploading = !is_failed && (is_pending || url.starts_with("blob:"));

    let fid_for_check = file_id_str.clone();
    let is_cached = !is_uploading
        && !is_failed
        && !fid_for_check.is_empty()
        && crate::components::common::media::is_file_cached(&fid_for_check);

    let on_file_click = {
        let fid = fid_for_open.clone();
        let fname = name_for_open.clone();
        let furl = url_for_open.clone();
        move |e: web_sys::MouseEvent| {
            e.prevent_default();
            e.stop_propagation();
            if is_uploading || is_failed {
                return;
            }
            let fid_c = fid.clone();
            let fname_c = fname.clone();
            let furl_c = furl.clone();
            wasm_bindgen_futures::spawn_local(async move {
                open_file_card_async(&fid_c, &fname_c, &furl_c).await;
            });
        }
    };

    let on_file_keydown = {
        let fid = fid_for_open.clone();
        let fname = name_for_open.clone();
        let furl = url_for_open.clone();
        move |e: web_sys::KeyboardEvent| {
            if e.key() == "Enter" || e.key() == " " {
                e.prevent_default();
                e.stop_propagation();
                if is_uploading || is_failed {
                    return;
                }
                let fid_c = fid.clone();
                let fname_c = fname.clone();
                let furl_c = furl.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    open_file_card_async(&fid_c, &fname_c, &furl_c).await;
                });
            }
        }
    };

    let card_class = if is_failed {
        "chat-message-file is-failed"
    } else if is_uploading {
        "chat-message-file is-uploading"
    } else if is_cached {
        "chat-message-file is-cached"
    } else {
        "chat-message-file"
    };

    let display_name = name.clone();
    let ext = gilvave_core::files::get_file_extension(&name, "");
    let (kind, display_ext) = classify_file_kind(&ext);
    let icon_class = format!("chat-message-file-icon cyber-file-shard {}", kind.class_name());

    let glyph_view = match kind {
        CyberFileKind::Pdf => view! {
            svg(viewBox="0 0 24 24", width="15", height="15", fill="none", stroke="currentColor", stroke-width="2.2", stroke-linecap="round", stroke-linejoin="round") {
                path(d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z")
                polyline(points="14 2 14 8 20 8")
                line(x1="9", y1="13", x2="15", y2="13")
                line(x1="9", y1="17", x2="13", y2="17")
            }
        },
        CyberFileKind::Code => view! {
            svg(viewBox="0 0 24 24", width="15", height="15", fill="none", stroke="currentColor", stroke-width="2.2", stroke-linecap="round", stroke-linejoin="round") {
                polyline(points="16 18 22 12 16 6")
                polyline(points="8 6 2 12 8 18")
            }
        },
        CyberFileKind::Archive => view! {
            svg(viewBox="0 0 24 24", width="15", height="15", fill="none", stroke="currentColor", stroke-width="2.2", stroke-linecap="round", stroke-linejoin="round") {
                polyline(points="21 8 21 21 3 21 3 8")
                rect(x="1", y="3", width="22", height="5", rx="1")
                line(x1="10", y1="12", x2="14", y2="12")
            }
        },
        CyberFileKind::Audio => view! {
            svg(viewBox="0 0 24 24", width="15", height="15", fill="none", stroke="currentColor", stroke-width="2.2", stroke-linecap="round", stroke-linejoin="round") {
                line(x1="4", y1="10", x2="4", y2="14")
                line(x1="8", y1="6", x2="8", y2="18")
                line(x1="12", y1="3", x2="12", y2="21")
                line(x1="16", y1="8", x2="16", y2="16")
                line(x1="20", y1="11", x2="20", y2="13")
            }
        },
        CyberFileKind::Sheet => view! {
            svg(viewBox="0 0 24 24", width="15", height="15", fill="none", stroke="currentColor", stroke-width="2.2", stroke-linecap="round", stroke-linejoin="round") {
                rect(x="3", y="3", width="18", height="18", rx="2")
                line(x1="3", y1="9", x2="21", y2="9")
                line(x1="3", y1="15", x2="21", y2="15")
                line(x1="9", y1="3", x2="9", y2="21")
                line(x1="15", y1="3", x2="15", y2="21")
            }
        },
        CyberFileKind::Doc => view! {
            svg(viewBox="0 0 24 24", width="15", height="15", fill="none", stroke="currentColor", stroke-width="2.2", stroke-linecap="round", stroke-linejoin="round") {
                path(d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z")
                polyline(points="14 2 14 8 20 8")
                line(x1="8", y1="13", x2="16", y2="13")
                line(x1="8", y1="17", x2="14", y2="17")
            }
        },
        CyberFileKind::Exe => view! {
            svg(viewBox="0 0 24 24", width="15", height="15", fill="none", stroke="currentColor", stroke-width="2.2", stroke-linecap="round", stroke-linejoin="round") {
                polygon(points="13 2 3 14 12 14 11 22 21 10 12 10 13 2")
            }
        },
        CyberFileKind::Image => view! {
            svg(viewBox="0 0 24 24", width="15", height="15", fill="none", stroke="currentColor", stroke-width="2.2", stroke-linecap="round", stroke-linejoin="round") {
                rect(x="3", y="3", width="18", height="18", rx="2")
                circle(cx="8.5", cy="8.5", r="1.5")
                polyline(points="21 15 16 10 5 21")
            }
        },
        CyberFileKind::Generic => view! {
            svg(viewBox="0 0 24 24", width="15", height="15", fill="none", stroke="currentColor", stroke-width="2.2", stroke-linecap="round", stroke-linejoin="round") {
                polygon(points="12 2 2 12 12 22 22 12")
                circle(cx="12", cy="12", r="3")
            }
        },
    };

    view! {
        div(
            class=card_class,
            role="button",
            tabindex="0",
            data-file-id=file_id_str,
            data-filename=name,
            data-file-url=url,
            on:click=on_file_click,
            on:keydown=on_file_keydown,
        ) {
            span(class=icon_class) {
                span(class="cyber-corner-notch")
                span(class="cyber-glyph") {
                    (glyph_view)
                }
                span(class="cyber-ext-label") {
                    (display_ext)
                }
            }
            span(class="chat-message-file-info") {
                span(class="chat-message-file-name") { (display_name) }
                (if is_failed {
                    view! {
                        span(class="chat-message-file-action is-failed") {
                            svg(class="failed-alert-icon", viewBox="0 0 24 24", width="12", height="12", fill="none", stroke="currentColor", stroke-width="2.5", stroke-linecap="round", stroke-linejoin="round") {
                                circle(cx="12", cy="12", r="10")
                                line(x1="12", y1="8", x2="12", y2="12")
                                line(x1="12", y1="16", x2="12.01", y2="16")
                            }
                            "Не удалось отправить"
                        }
                    }
                } else if is_uploading {
                    view! {
                        span(class="chat-message-file-action is-uploading") {
                            "Загрузка файла..."
                        }
                    }
                } else if is_cached {
                    view! {
                        span(class="chat-message-file-action cached-action") {
                            svg(class="cached-check-icon", viewBox="0 0 24 24", width="12", height="12", fill="none", stroke="currentColor", stroke-width="2.5", stroke-linecap="round", stroke-linejoin="round") {
                                polyline(points="20 6 9 17 4 12")
                            }
                            "Файл сохранён · Нажмите для открытия"
                        }
                    }
                } else {
                    view! {
                        span(class="chat-message-file-action") {
                            "Нажмите для скачивания"
                        }
                    }
                })
            }
            button(
                r#type="button",
                class="chat-message-file-download",
                aria-label=if is_cached { "Открыть файл" } else { "Скачать файл" },
                on:click={
                    let on_file_click = on_file_click.clone();
                    move |e: web_sys::MouseEvent| {
                        on_file_click(e);
                    }
                },
            ) {
                (if is_failed {
                    view! {
                        svg(viewBox="0 0 24 24", width="18", height="18", fill="none", stroke="currentColor", stroke-width="2.2", stroke-linecap="round", stroke-linejoin="round") {
                            circle(cx="12", cy="12", r="10")
                            line(x1="12", y1="8", x2="12", y2="12")
                            line(x1="12", y1="16", x2="12.01", y2="16")
                        }
                    }
                } else if is_uploading {
                    view! {
                        svg(class="spin-loader-icon", viewBox="0 0 24 24", width="18", height="18", fill="none", stroke="currentColor", stroke-width="2.5", stroke-linecap="round") {
                            circle(cx="12", cy="12", r="9", stroke-opacity="0.25")
                            path(d="M12 3a9 9 0 0 1 9 9")
                        }
                    }
                } else if is_cached {
                    view! {
                        svg(viewBox="0 0 24 24", width="18", height="18", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                            path(d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6")
                            polyline(points="15 3 21 3 21 9")
                            line(x1="10", y1="14", x2="21", y2="3")
                        }
                    }
                } else {
                    view! {
                        svg(viewBox="0 0 24 24", width="18", height="18", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                            path(d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4")
                            polyline(points="7 10 12 15 17 10")
                            line(x1="12", y1="15", x2="12", y2="3")
                        }
                    }
                })
            }
        }
    }
}

/// Корневой модальный полноэкранный видеоплеер (Театр)
#[component(inline_props)]
pub fn FullscreenVideoModal(
    active: crate::components::common::media::ActiveFullscreenVideo,
) -> View {
    let modal_ctx = use_context::<UiModalContext>();

    let video_ref = create_node_ref();
    let scrubber_ref = create_node_ref();

    let is_playing = create_signal(true);
    let is_muted = create_signal(false);
    let current_time = create_signal(active.initial_time);
    let duration = create_signal(0.0f64);
    let is_scrubbing = create_signal(false);
    let is_closing = create_signal(false);
    let is_controls_visible = create_signal(true);

    let timer_cell: std::rc::Rc<std::cell::RefCell<Option<gloo_timers::callback::Timeout>>> =
        std::rc::Rc::new(std::cell::RefCell::new(None));

    let reset_activity = {
        let timer_cell = timer_cell.clone();
        move || {
            if is_closing.get() {
                return;
            }
            is_controls_visible.set(true);
            if let Ok(mut cell) = timer_cell.try_borrow_mut() {
                *cell = None;
                if is_playing.get() && !is_scrubbing.get() {
                    let cell_inner = timer_cell.clone();
                    *cell = Some(gloo_timers::callback::Timeout::new(2500, move || {
                        if !is_closing.get() && is_playing.get() && !is_scrubbing.get() {
                            is_controls_visible.set(false);
                        }
                        if let Ok(mut c) = cell_inner.try_borrow_mut() {
                            *c = None;
                        }
                    }));
                }
            }
        }
    };

    {
        let timer_cell = timer_cell.clone();
        on_cleanup(move || {
            if let Ok(mut cell) = timer_cell.try_borrow_mut() {
                *cell = None;
            }
        });
    }

    let close_modal = {
        let timer_cell = timer_cell.clone();
        move || {
            if is_closing.get() {
                return;
            }
            is_closing.set(true);

            if let Ok(mut cell) = timer_cell.try_borrow_mut() {
                *cell = None;
            }

            if let Some(video) = video_ref
                .try_get()
                .and_then(|n| n.dyn_into::<web_sys::HtmlVideoElement>().ok())
            {
                let _ = video.pause();
            }

            modal_ctx.active_fullscreen_video.set(None);
        }
    };

    let end_scrub = {
        let reset_activity = reset_activity.clone();
        move || {
            if is_scrubbing.get() {
                is_scrubbing.set(false);
                reset_activity();
            }
        }
    };

    let toggle_play = {
        let reset_activity = reset_activity.clone();
        move || {
            reset_activity();
            if let Some(video) = video_ref
                .try_get()
                .and_then(|n| n.dyn_into::<web_sys::HtmlVideoElement>().ok())
            {
                if is_playing.get() {
                    let _ = video.pause();
                } else {
                    let _ = video.play();
                }
            }
        }
    };

    let toggle_mute = {
        let reset_activity = reset_activity.clone();
        move || {
            reset_activity();
            if let Some(video) = video_ref
                .try_get()
                .and_then(|n| n.dyn_into::<web_sys::HtmlVideoElement>().ok())
            {
                let next_muted = !video.muted();
                video.set_muted(next_muted);
                is_muted.set(next_muted);
            }
        }
    };

    let seek_to_ratio = move |ratio: f64| {
        let video_elem = video_ref
            .try_get()
            .and_then(|n| n.dyn_into::<web_sys::HtmlVideoElement>().ok());
        let mut dur = duration.get();
        if (dur <= 0.0 || dur.is_nan() || dur.is_infinite()) && let Some(v) = &video_elem {
            let vd = v.duration();
            if vd > 0.0 && !vd.is_nan() && !vd.is_infinite() {
                dur = vd;
                duration.set(vd);
            }
        }
        if dur > 0.0 && !dur.is_nan() && !dur.is_infinite() {
            let target_time = (ratio * dur).clamp(0.0, dur);
            current_time.set(target_time);
            if let Some(video) = video_elem {
                video.set_current_time(target_time);
            }
        }
    };

    let on_scrubber_pointer = {
        let reset_activity = reset_activity.clone();
        move |client_x: f64| {
            reset_activity();
            if let Some(scrubber) = scrubber_ref
                .try_get()
                .and_then(|n| n.dyn_into::<web_sys::Element>().ok())
            {
                let rect = scrubber.get_bounding_client_rect();
                let width = rect.width();
                if width > 0.0 {
                    let click_x = (client_x - rect.left()).clamp(0.0, width);
                    let ratio = click_x / width;
                    seek_to_ratio(ratio);
                }
            }
        }
    };

    // Слушатель Escape для закрытия полноэкранного режима
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            let close_esc = close_modal.clone();
            let on_keydown = wasm_bindgen::closure::Closure::<dyn Fn(web_sys::KeyboardEvent)>::wrap(
                Box::new(move |e: web_sys::KeyboardEvent| {
                    if e.key() == "Escape" {
                        close_esc();
                    }
                }) as Box<dyn Fn(web_sys::KeyboardEvent)>,
            );
            let _ = window.add_event_listener_with_callback(
                "keydown",
                on_keydown.as_ref().unchecked_ref(),
            );
            let win = window.clone();
            on_cleanup(move || {
                let _ = win.remove_event_listener_with_callback(
                    "keydown",
                    on_keydown.as_ref().unchecked_ref(),
                );
            });
        }
    }

    let fill_style = move || {
        let dur = duration.get();
        let cur = current_time.get();
        let pct = if dur > 0.0 && !dur.is_nan() && !dur.is_infinite() {
            (cur / dur * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        };
        format!("width: {pct:.1}%;")
    };

    let cur_time_str = move || format_time(current_time.get());
    let dur_time_str = move || format_time(duration.get());

    let download_url = active.url.clone();
    let download_name = active.title.clone();

    let overlay_class = move || {
        let mut cls = String::from("fullscreen-video-theater-overlay");
        if is_closing.get() {
            cls.push_str(" is-closing");
        } else if !is_controls_visible.get() {
            cls.push_str(" is-controls-hidden");
        }
        cls
    };

    let toggle_play_video = toggle_play.clone();
    let toggle_play_center = toggle_play.clone();
    let toggle_play_btn = toggle_play.clone();

    view! {
        div(
            class=overlay_class(),
            on:click={
                let close_modal = close_modal.clone();
                move |e: web_sys::MouseEvent| {
                    if is_scrubbing.get() {
                        return;
                    }
                    if let Some(target) = e.target()
                        && let Ok(el) = target.dyn_into::<web_sys::Element>()
                    {
                        let cls = el.class_name();
                        if cls.contains("fullscreen-video-theater-overlay")
                            || cls.contains("fullscreen-video-stage")
                        {
                            close_modal();
                        }
                    }
                }
            },
            on:mousemove={
                let reset_activity = reset_activity.clone();
                let on_scrubber_pointer = on_scrubber_pointer.clone();
                move |e: web_sys::MouseEvent| {
                    reset_activity();
                    if is_scrubbing.get() {
                        on_scrubber_pointer(e.client_x() as f64);
                    }
                }
            },
            on:mouseup={
                let end_scrub = end_scrub.clone();
                move |_| end_scrub()
            },
            on:touchstart={
                let reset_activity = reset_activity.clone();
                let on_scrubber_pointer = on_scrubber_pointer.clone();
                move |e: web_sys::TouchEvent| {
                    reset_activity();
                    if let Some(touch) = e.touches().item(0)
                        && is_scrubbing.get()
                    {
                        on_scrubber_pointer(touch.client_x() as f64);
                    }
                }
            },
            on:touchmove={
                let reset_activity = reset_activity.clone();
                let on_scrubber_pointer = on_scrubber_pointer.clone();
                move |e: web_sys::TouchEvent| {
                    reset_activity();
                    if let Some(touch) = e.touches().item(0)
                        && is_scrubbing.get()
                    {
                        on_scrubber_pointer(touch.client_x() as f64);
                    }
                }
            },
            on:touchend={
                let end_scrub = end_scrub.clone();
                move |_| end_scrub()
            },
        ) {
            // Верхняя плашка с названием и кнопкой закрытия
            div(
                class="video-fullscreen-topbar",
                on:click={
                    let reset_activity = reset_activity.clone();
                    move |e: web_sys::MouseEvent| {
                        e.stop_propagation();
                        reset_activity();
                    }
                },
                on:mousemove={
                    let reset_activity = reset_activity.clone();
                    move |_| reset_activity()
                },
            ) {
                div(class="video-topbar-info") {
                    svg(class="video-topbar-icon", viewBox="0 0 24 24", width="20", height="20", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                        rect(x="2", y="2", width="20", height="20", rx="2.18", ry="2.18")
                        line(x1="7", y1="2", x2="7", y2="22")
                        line(x1="17", y1="2", x2="17", y2="22")
                        line(x1="2", y1="12", x2="22", y2="12")
                        line(x1="2", y1="7", x2="7", y2="7")
                        line(x1="2", y1="17", x2="7", y2="17")
                        line(x1="17", y1="17", x2="22", y2="17")
                        line(x1="17", y1="7", x2="22", y2="7")
                    }
                    span(class="video-topbar-title") { (active.title) }
                }
                button(
                    r#type="button",
                    class="video-fullscreen-close-btn",
                    aria-label="Закрыть плеер",
                    title="Закрыть плеер (Esc)",
                    on:click={
                        let close_modal = close_modal.clone();
                        move |e: web_sys::MouseEvent| {
                            e.stop_propagation();
                            close_modal();
                        }
                    },
                ) {
                    svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                        line(x1="18", y1="6", x2="6", y2="18")
                        line(x1="6", y1="6", x2="18", y2="18")
                    }
                }
            }

            // Центральная сцена с видео
            div(class="fullscreen-video-stage") {
                video(
                    ref=video_ref,
                    class="fullscreen-theater-video",
                    src=active.url,
                    autoplay=true,
                    playsinline=true,
                    preload="auto",
                    on:play={
                        let reset_activity = reset_activity.clone();
                        move |_| {
                            if !is_closing.get() {
                                is_playing.set(true);
                                reset_activity();
                            }
                        }
                    },
                    on:pause=move |_| {
                        if !is_closing.get() {
                            is_playing.set(false);
                            is_controls_visible.set(true);
                        }
                    },
                    on:ended=move |_| {
                        if !is_closing.get() {
                            is_playing.set(false);
                            is_controls_visible.set(true);
                        }
                    },
                    on:timeupdate=move |_| {
                        if !is_closing.get()
                            && let Some(v) = video_ref
                                .try_get()
                                .and_then(|n| n.dyn_into::<web_sys::HtmlVideoElement>().ok())
                        {
                            let d = v.duration();
                            if duration.get() <= 0.0 && d > 0.0 && !d.is_nan() && !d.is_infinite() {
                                duration.set(d);
                            }
                            if !is_scrubbing.get() && !v.seeking() {
                                current_time.set(v.current_time());
                            }
                        }
                    },
                    on:loadedmetadata=move |_| {
                        if !is_closing.get()
                            && let Some(v) = video_ref
                                .try_get()
                                .and_then(|n| n.dyn_into::<web_sys::HtmlVideoElement>().ok())
                        {
                            let d = v.duration();
                            if d > 0.0 && !d.is_nan() && !d.is_infinite() {
                                duration.set(d);
                            }
                            if active.initial_time > 0.0 {
                                v.set_current_time(active.initial_time);
                            }
                            let _ = v.play();
                        }
                    },
                    on:durationchange=move |_| {
                        if !is_closing.get()
                            && let Some(v) = video_ref
                                .try_get()
                                .and_then(|n| n.dyn_into::<web_sys::HtmlVideoElement>().ok())
                        {
                            let d = v.duration();
                            if d > 0.0 && !d.is_nan() && !d.is_infinite() {
                                duration.set(d);
                            }
                        }
                    },
                    on:canplay=move |_| {
                        if !is_closing.get()
                            && let Some(v) = video_ref
                                .try_get()
                                .and_then(|n| n.dyn_into::<web_sys::HtmlVideoElement>().ok())
                        {
                            let d = v.duration();
                            if duration.get() <= 0.0 && d > 0.0 && !d.is_nan() && !d.is_infinite() {
                                duration.set(d);
                            }
                        }
                    },
                    on:click=move |e: web_sys::MouseEvent| {
                        e.stop_propagation();
                        toggle_play_video();
                    },
                )
                (if !is_playing.get() {
                    let toggle_play = toggle_play_center.clone();
                    view! {
                        button(
                            r#type="button",
                            class="video-center-play-btn fullscreen-center-btn",
                            aria-label="Воспроизвести",
                            on:click=move |e: web_sys::MouseEvent| {
                                e.stop_propagation();
                                toggle_play();
                            },
                        ) {
                            svg(class="play-icon", viewBox="0 0 24 24", fill="currentColor") {
                                polygon(points="5 3 19 12 5 21 5 3")
                            }
                        }
                    }
                } else {
                    view! {}
                })
            }

            // Плавающая панель управления снизу
            div(
                class="video-custom-controls fullscreen-controls-bar",
                on:click={
                    let reset_activity = reset_activity.clone();
                    move |e: web_sys::MouseEvent| {
                        e.stop_propagation();
                        reset_activity();
                    }
                },
                on:mousemove={
                    let reset_activity = reset_activity.clone();
                    let on_scrubber_pointer = on_scrubber_pointer.clone();
                    move |e: web_sys::MouseEvent| {
                        reset_activity();
                        if is_scrubbing.get() {
                            on_scrubber_pointer(e.client_x() as f64);
                        }
                    }
                },
                on:mouseup={
                    let end_scrub = end_scrub.clone();
                    move |_| end_scrub()
                },
                on:touchend={
                    let end_scrub = end_scrub.clone();
                    move |_| end_scrub()
                },
            ) {
                div(
                    ref=scrubber_ref,
                    class="video-scrubber-track",
                    on:mousedown={
                        let on_scrubber_pointer = on_scrubber_pointer.clone();
                        move |e: web_sys::MouseEvent| {
                            e.stop_propagation();
                            is_scrubbing.set(true);
                            on_scrubber_pointer(e.client_x() as f64);
                        }
                    },
                    on:click={
                        let on_scrubber_pointer = on_scrubber_pointer.clone();
                        move |e: web_sys::MouseEvent| {
                            e.stop_propagation();
                            on_scrubber_pointer(e.client_x() as f64);
                        }
                    },
                    on:mousemove={
                        let on_scrubber_pointer = on_scrubber_pointer.clone();
                        move |e: web_sys::MouseEvent| {
                            if is_scrubbing.get() {
                                on_scrubber_pointer(e.client_x() as f64);
                            }
                        }
                    },
                    on:mouseup={
                        let end_scrub = end_scrub.clone();
                        move |_| end_scrub()
                    },
                    on:touchstart={
                        let on_scrubber_pointer = on_scrubber_pointer.clone();
                        move |e: web_sys::TouchEvent| {
                            if let Some(touch) = e.touches().item(0) {
                                is_scrubbing.set(true);
                                on_scrubber_pointer(touch.client_x() as f64);
                            }
                        }
                    },
                    on:touchmove={
                        let on_scrubber_pointer = on_scrubber_pointer.clone();
                        move |e: web_sys::TouchEvent| {
                            if let Some(touch) = e.touches().item(0) {
                                on_scrubber_pointer(touch.client_x() as f64);
                            }
                        }
                    },
                    on:touchend={
                        let end_scrub = end_scrub.clone();
                        move |_| end_scrub()
                    },
                ) {
                    div(class="video-scrubber-fill", style=fill_style()) {
                        div(class="video-scrubber-thumb")
                    }
                }
                div(class="video-controls-row") {
                    div(class="video-controls-left") {
                        button(
                            r#type="button",
                            class="video-btn video-play-toggle",
                            aria-label=if is_playing.get() { "Пауза" } else { "Воспроизвести" },
                            on:click=move |e: web_sys::MouseEvent| {
                                e.stop_propagation();
                                toggle_play_btn();
                            },
                        ) {
                            (move || if is_playing.get() {
                                view! {
                                    svg(class="icon-pause", viewBox="0 0 24 24", fill="currentColor") {
                                        rect(x="6", y="4", width="4", height="16")
                                        rect(x="14", y="4", width="4", height="16")
                                    }
                                }
                            } else {
                                view! {
                                    svg(class="icon-play", viewBox="0 0 24 24", fill="currentColor") {
                                        polygon(points="5 3 19 12 5 21 5 3")
                                    }
                                }
                            })
                        }
                        div(class="video-time-display") {
                            span(class="video-time-current") { (cur_time_str()) }
                            span(class="video-time-divider") { "/" }
                            span(class="video-time-duration") { (dur_time_str()) }
                        }
                    }
                    div(class="video-controls-right") {
                        button(
                            r#type="button",
                            class="video-btn video-volume-toggle",
                            aria-label=if is_muted.get() { "Включить звук" } else { "Выключить звук" },
                            on:click={
                                let toggle_mute = toggle_mute.clone();
                                move |e: web_sys::MouseEvent| {
                                    e.stop_propagation();
                                    toggle_mute();
                                }
                            },
                        ) {
                            (move || if is_muted.get() {
                                view! {
                                    svg(class="icon-volume-muted", viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                                        polygon(points="11 5 6 9 2 9 2 15 6 15 11 19 11 5")
                                        line(x1="23", y1="9", x2="17", y2="15")
                                        line(x1="17", y1="9", x2="23", y2="15")
                                    }
                                }
                            } else {
                                view! {
                                    svg(class="icon-volume-on", viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                                        polygon(points="11 5 6 9 2 9 2 15 6 15 11 19 11 5")
                                        path(d="M19.07 4.93a10 10 0 0 1 0 14.14M15.54 8.46a5 5 0 0 1 0 7.07")
                                    }
                                }
                            })
                        }
                        button(
                            r#type="button",
                            class="video-btn video-fullscreen-toggle",
                            aria-label="Свернуть полноэкранный режим",
                            on:click={
                                let close_modal = close_modal.clone();
                                move |e: web_sys::MouseEvent| {
                                    e.stop_propagation();
                                    close_modal();
                                }
                            },
                        ) {
                            svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                                polyline(points="4 14 10 14 10 20")
                                polyline(points="20 10 14 10 14 4")
                                line(x1="14", y1="10", x2="21", y2="3")
                                line(x1="3", y1="21", x2="10", y2="14")
                            }
                        }
                        a(
                            class="video-btn video-download-link",
                            href=download_url,
                            download=download_name,
                            target="_blank",
                            rel="noopener noreferrer",
                            aria-label="Скачать видео",
                        ) {
                            svg(viewBox="0 0 24 24", width="14", height="14", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                                path(d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4")
                                polyline(points="7 10 12 15 17 10")
                                line(x1="12", y1="15", x2="12", y2="3")
                            }
                        }
                    }
                }
            }
        }
    }
}
