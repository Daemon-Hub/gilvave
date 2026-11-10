#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

/// Чтение изображения в Data URL
#[allow(dead_code)]
#[cfg(target_arch = "wasm32")]
pub async fn read_image_file_as_data_url(file: web_sys::File) -> Result<(String, String), String> {
    let file_name = file.name();
    let file_type = file.type_();

    if file_type == "image/svg+xml" || file_type == "image/gif" {
        let raw_data_url = read_file_blob_as_data_url(&file).await?;
        return Ok((file_name, raw_data_url));
    }

    if file.size() <= 81_920.0 {
        let raw_data_url = read_file_blob_as_data_url(&file).await?;
        return Ok((file_name, raw_data_url));
    }

    match optimize_image_with_canvas(&file).await {
        Ok(opt_url) => Ok((file_name, opt_url)),
        Err(_) => {
            let fallback_url = read_file_blob_as_data_url(&file).await?;
            Ok((file_name, fallback_url))
        }
    }
}

/// Чтение изображения в Data URL
#[allow(dead_code)]
#[cfg(not(target_arch = "wasm32"))]
pub async fn read_image_file_as_data_url(_file: web_sys::File) -> Result<(String, String), String> {
    Ok((String::new(), String::new()))
}

#[cfg(target_arch = "wasm32")]
async fn read_file_blob_as_data_url(blob: &web_sys::Blob) -> Result<String, String> {
    let fr = web_sys::FileReader::new().map_err(|e| format!("{e:?}"))?;
    let fr_clone = fr.clone();
    let (tx, rx) = futures_channel::oneshot::channel::<Result<String, String>>();
    let mut tx_opt = Some(tx);

    let onload = wasm_bindgen::closure::Closure::once(move || {
        if let Ok(res) = fr_clone.result() {
            if let Some(s) = res.as_string() {
                if let Some(tx) = tx_opt.take() {
                    let _ = tx.send(Ok(s));
                }
                return;
            }
        }
        if let Some(tx) = tx_opt.take() {
            let _ = tx.send(Err("Failed to read file as string".to_string()));
        }
    });

    fr.set_onload(Some(onload.as_ref().unchecked_ref()));
    fr.read_as_data_url(blob).map_err(|e| format!("{e:?}"))?;

    rx.await.map_err(|e| format!("{e:?}"))?
}

#[cfg(target_arch = "wasm32")]
async fn optimize_image_with_canvas(file: &web_sys::File) -> Result<String, String> {
    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;

    let object_url =
        web_sys::Url::create_object_url_with_blob(file.as_ref()).map_err(|e| format!("{e:?}"))?;

    let img = web_sys::HtmlImageElement::new().map_err(|e| format!("{e:?}"))?;
    let img_clone = img.clone();
    let object_url_clone = object_url.clone();

    let (tx, rx) = futures_channel::oneshot::channel::<Result<(), String>>();
    let mut tx_opt = Some(tx);

    let onload = wasm_bindgen::closure::Closure::once(move || {
        if let Some(tx) = tx_opt.take() {
            let _ = tx.send(Ok(()));
        }
    });

    img.set_onload(Some(onload.as_ref().unchecked_ref()));
    img.set_src(&object_url);

    let load_res = rx.await.map_err(|e| format!("{e:?}"))?;
    let _ = web_sys::Url::revoke_object_url(&object_url_clone);
    load_res?;

    let orig_w = img_clone.natural_width();
    let orig_h = img_clone.natural_height();
    if orig_w == 0 || orig_h == 0 {
        return Err("Invalid image dimensions".to_string());
    }

    // Scale to max 1280x1280
    const MAX_DIM: f64 = 1280.0;
    let mut target_w = orig_w as f64;
    let mut target_h = orig_h as f64;

    if target_w > MAX_DIM || target_h > MAX_DIM {
        if target_w > target_h {
            target_h = (target_h * MAX_DIM / target_w).round();
            target_w = MAX_DIM;
        } else {
            target_w = (target_w * MAX_DIM / target_h).round();
            target_h = MAX_DIM;
        }
    }

    let canvas_el = document
        .create_element("canvas")
        .map_err(|e| format!("{e:?}"))?;
    let canvas: web_sys::HtmlCanvasElement = canvas_el.dyn_into().map_err(|e| format!("{e:?}"))?;
    canvas.set_width(target_w as u32);
    canvas.set_height(target_h as u32);

    let ctx_obj = canvas
        .get_context("2d")
        .map_err(|e| format!("{e:?}"))?
        .ok_or("No 2d context")?;
    let ctx: web_sys::CanvasRenderingContext2d =
        ctx_obj.dyn_into().map_err(|e| format!("{e:?}"))?;

    ctx.draw_image_with_html_image_element_and_dw_and_dh(&img_clone, 0.0, 0.0, target_w, target_h)
        .map_err(|e| format!("{e:?}"))?;

    // Try webp first; fallback to jpeg if webp is unsupported
    let data_url = canvas
        .to_data_url_with_type_and_encoder_options(
            "image/webp",
            &wasm_bindgen::JsValue::from_f64(0.82),
        )
        .unwrap_or_else(|_| {
            canvas
                .to_data_url_with_type_and_encoder_options(
                    "image/jpeg",
                    &wasm_bindgen::JsValue::from_f64(0.82),
                )
                .unwrap_or_default()
        });

    if data_url.is_empty() {
        return Err("Canvas toDataURL returned empty".to_string());
    }

    Ok(data_url)
}

/// Прикрепляемый файл, ожидающий подтверждения отправки пользователем
#[derive(Clone, PartialEq)]
pub struct PendingAttachment {
    pub id: String,
    pub file: Option<web_sys::File>,
    pub name: String,
    pub size_bytes: u64,
    pub is_previewable: bool,
    pub is_video: bool,
    pub preview_url: Option<String>,
    pub existing_file_id: Option<gilvave_core::ids::FileId>,
    pub existing_url: Option<String>,
}

impl PendingAttachment {
    pub fn new(file: web_sys::File) -> Self {
        let name = file.name();
        let size_bytes = file.size() as u64;
        let mime = file.type_();
        let is_image = mime.starts_with("image/");
        let is_video = mime.starts_with("video/");
        let is_previewable = is_image || is_video;
        let preview_url = web_sys::Url::create_object_url_with_blob(&file).ok();
        let id = gilvave_core::ids::FileId::default().to_string();

        Self {
            id,
            file: Some(file),
            name,
            size_bytes,
            is_previewable,
            is_video,
            preview_url,
            existing_file_id: None,
            existing_url: None,
        }
    }

    pub fn from_existing(
        name: String,
        url: String,
        is_previewable: bool,
        is_video: bool,
        file_id: Option<gilvave_core::ids::FileId>,
    ) -> Self {
        let preview_url = Some(url.clone());
        let id = file_id
            .map(|f| f.to_string())
            .unwrap_or_else(|| gilvave_core::ids::FileId::default().to_string());

        Self {
            id,
            file: None,
            name,
            size_bytes: 0,
            is_previewable,
            is_video,
            preview_url,
            existing_file_id: file_id,
            existing_url: Some(url),
        }
    }

    pub fn formatted_size(&self) -> String {
        if self.size_bytes == 0 {
            if self.is_previewable {
                if self.is_video {
                    "Видео".to_string()
                } else {
                    "Фото".to_string()
                }
            } else {
                "Файл".to_string()
            }
        } else if self.size_bytes < 1024 * 1024 {
            format!("{:.1} КБ", self.size_bytes as f64 / 1024.0)
        } else {
            format!("{:.1} МБ", self.size_bytes as f64 / (1024.0 * 1024.0))
        }
    }

    pub fn file_extension(&self) -> String {
        self.name
            .rsplit('.')
            .next()
            .unwrap_or("FILE")
            .to_ascii_uppercase()
    }

    pub fn revoke(&self) {
        if let Some(url) = &self.preview_url
            && url.starts_with("blob:")
        {
            let _ = web_sys::Url::revoke_object_url(url);
        }
    }
}

/// Проверка на источник видео
pub fn is_video_source(src: &str, alt: &str) -> bool {
    if src.starts_with("data:image/") {
        return false;
    }
    if src.starts_with("data:video/") {
        return true;
    }
    let check = |s: &str| {
        let lower = s.to_ascii_lowercase();
        lower.contains("/videos/")
            || lower.ends_with(".mp4")
            || lower.ends_with(".webm")
            || lower.ends_with(".mov")
            || lower.ends_with(".ogg")
            || lower.ends_with(".ogv")
            || lower.ends_with(".avi")
            || lower.ends_with(".mkv")
            || lower.ends_with(".m4v")
    };
    check(src) || check(alt)
}

/// Парсинг FileId из URL или ссылки
pub fn parse_file_id_from_url(url: &str) -> Option<gilvave_core::ids::FileId> {
    if let Some(id_str) = url.strip_prefix("file:") {
        let clean = id_str.trim();
        let uuid_part = if clean.len() >= 36 { &clean[..36] } else { clean };
        return uuid_part.parse::<gilvave_core::ids::FileId>().ok();
    }
    if let Some(id_str) = url.strip_prefix("file_id:") {
        let clean = id_str.trim();
        let uuid_part = if clean.len() >= 36 { &clean[..36] } else { clean };
        return uuid_part.parse::<gilvave_core::ids::FileId>().ok();
    }
    if let Some(pos) = url.rfind("/files/") {
        let part = &url[pos + 7..];
        let stem = part
            .split(['/', '?', '#', '.'])
            .next()
            .unwrap_or(part);
        if stem.len() >= 36 {
            return stem[..36].parse::<gilvave_core::ids::FileId>().ok();
        }
    }
    None
}

#[derive(Clone, Debug, PartialEq)]
pub enum ParsedAttachment {
    Image {
        name: String,
        url: String,
    },
    Video {
        name: String,
        url: String,
    },
    File {
        name: String,
        file_id: Option<gilvave_core::ids::FileId>,
        url: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct ActiveFullscreenVideo {
    pub url: String,
    pub title: String,
    pub initial_time: f64,
}

/// Извлечение вложений и чистого текста из сообщения для нативного отображения
pub fn extract_message_media_and_text(content: &str) -> (String, Vec<ParsedAttachment>) {
    let mut attachments = Vec::new();
    let mut clean_text = String::with_capacity(content.len());
    let mut remaining = content;

    while !remaining.is_empty() {
        // 1. Изображение или видео: ![alt](url)
        if remaining.starts_with("![")
            && let Some(close_bracket) = remaining.find("](")
        {
            let alt = &remaining[2..close_bracket];
            let after = &remaining[close_bracket + 2..];
            if let Some(close_paren) = after.find(')') {
                let url = &after[..close_paren];
                if is_video_source(url, alt) {
                    attachments.push(ParsedAttachment::Video {
                        name: alt.to_string(),
                        url: url.to_string(),
                    });
                } else {
                    attachments.push(ParsedAttachment::Image {
                        name: alt.to_string(),
                        url: url.to_string(),
                    });
                }
                remaining = &after[close_paren + 1..];
                continue;
            }
        }

        // 2. Файловая ссылка: [name](file:id) или [name](http.../files/id)
        if remaining.starts_with('[')
            && let Some(close_bracket) = remaining.find("](")
        {
            let name = &remaining[1..close_bracket];
            let after = &remaining[close_bracket + 2..];
            if let Some(close_paren) = after.find(')') {
                let link = &after[..close_paren];
                if link.starts_with("file:")
                    || link.starts_with("file_id:")
                    || link.contains("/files/")
                    || link.starts_with("blob:")
                {
                    let file_id = parse_file_id_from_url(link);
                    let download_url = if let Some(fid) = link.strip_prefix("file:") {
                        format!("{}/files/{}", gilvave_core::settings::BASE_HTTP_URL, fid.trim())
                    } else if let Some(fid) = link.strip_prefix("file_id:") {
                        format!("{}/files/{}", gilvave_core::settings::BASE_HTTP_URL, fid.trim())
                    } else if link.starts_with("http://") || link.starts_with("https://") || link.starts_with("blob:") {
                        link.to_string()
                    } else {
                        format!("{}{}", gilvave_core::settings::BASE_HTTP_URL, link)
                    };
                    attachments.push(ParsedAttachment::File {
                        name: name.to_string(),
                        file_id,
                        url: download_url,
                    });
                    remaining = &after[close_paren + 1..];
                    continue;
                }
            }
        }

        let next_char = match remaining.chars().next() {
            Some(c) => c,
            None => break,
        };
        clean_text.push(next_char);
        remaining = &remaining[next_char.len_utf8()..];
    }

    (clean_text.trim().to_string(), attachments)
}

/// Извлечение вложений из текста Markdown при редактировании сообщения
pub fn extract_attachments_from_markdown(content: &str) -> (String, Vec<PendingAttachment>) {
    let mut attachments = Vec::new();
    let mut clean_text = String::with_capacity(content.len());
    let mut remaining = content;

    while !remaining.is_empty() {
        // 1. Изображение или видео: ![alt](url)
        if remaining.starts_with("![")
            && let Some(close_bracket) = remaining.find("](")
        {
            let alt = &remaining[2..close_bracket];
            let after = &remaining[close_bracket + 2..];
            if let Some(close_paren) = after.find(')') {
                let url = &after[..close_paren];
                let is_vid = is_video_source(url, alt);
                let file_id = parse_file_id_from_url(url);
                attachments.push(PendingAttachment::from_existing(
                    alt.to_string(),
                    url.to_string(),
                    true,
                    is_vid,
                    file_id,
                ));
                remaining = &after[close_paren + 1..];
                continue;
            }
        }

        // 2. Файловая ссылка: [name](file:id) или [name](http.../files/id)
        if remaining.starts_with('[')
            && let Some(close_bracket) = remaining.find("](")
        {
            let name = &remaining[1..close_bracket];
            let after = &remaining[close_bracket + 2..];
            if let Some(close_paren) = after.find(')') {
                let link = &after[..close_paren];
                if link.starts_with("file:")
                    || link.starts_with("file_id:")
                    || link.contains("/files/")
                {
                    let file_id = parse_file_id_from_url(link);
                    attachments.push(PendingAttachment::from_existing(
                        name.to_string(),
                        link.to_string(),
                        false,
                        false,
                        file_id,
                    ));
                    remaining = &after[close_paren + 1..];
                    continue;
                }
            }
        }

        let next_char = match remaining.chars().next() {
            Some(c) => c,
            None => break,
        };
        clean_text.push(next_char);
        remaining = &remaining[next_char.len_utf8()..];
    }

    (clean_text.trim().to_string(), attachments)
}

/// Отметка файла как локально сохранённого
#[allow(unused_variables, dead_code)]
pub fn mark_file_cached(file_id: &str) {
    #[cfg(target_arch = "wasm32")]
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let mut set: std::collections::HashSet<String> = storage
                .get_item("gilvave_cached_files_v1")
                .ok()
                .flatten()
                .and_then(|json| serde_json::from_str(&json).ok())
                .unwrap_or_default();
            set.insert(file_id.to_string());
            if let Ok(serialized) = serde_json::to_string(&set) {
                let _ = storage.set_item("gilvave_cached_files_v1", &serialized);
            }
        }
    }
}

/// Проверка, сохранён ли файл локально в кэше
#[allow(unused_variables)]
pub fn is_file_cached(file_id: &str) -> bool {
    #[cfg(target_arch = "wasm32")]
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Some(json) = storage.get_item("gilvave_cached_files_v1").ok().flatten() {
                if let Ok(set) = serde_json::from_str::<std::collections::HashSet<String>>(&json) {
                    return set.contains(file_id);
                }
            }
        }
    }
    false
}

/// Кэширование загруженного пользователем файла в локальное хранилище
#[cfg(target_arch = "wasm32")]
pub async fn cache_uploaded_file_storage(file_id: &str, _file_name: &str, file: &web_sys::File) {
    mark_file_cached(file_id);

    // В desktop окружении файлы сохраняются нативно на диск, не нагружая память WebView2
    if crate::utils::is_tauri() {
        return;
    }

    // В Web / CacheStorage сохраняем блоб в фоновом кэше браузера без блокирующих сериализаций
    let Some(window) = web_sys::window() else { return; };
    let Ok(caches_val) = js_sys::Reflect::get(&window, &wasm_bindgen::JsValue::from_str("caches")) else { return; };
    if caches_val.is_undefined() || caches_val.is_null() { return; }

    let Ok(open_val) = js_sys::Reflect::get(&caches_val, &wasm_bindgen::JsValue::from_str("open")) else { return; };
    let Ok(open_fn) = open_val.dyn_into::<js_sys::Function>() else { return; };

    let Ok(promise) = js_sys::Reflect::apply(
        &open_fn,
        &caches_val,
        &js_sys::Array::of1(&wasm_bindgen::JsValue::from_str("gilvave-files-v1")),
    ) else { return; };

    let Ok(cache_js) = wasm_bindgen_futures::JsFuture::from(js_sys::Promise::from(promise)).await else { return; };
    let Ok(put_val) = js_sys::Reflect::get(&cache_js, &wasm_bindgen::JsValue::from_str("put")) else { return; };
    let Ok(put_fn) = put_val.dyn_into::<js_sys::Function>() else { return; };

    if let Ok(resp) = web_sys::Response::new_with_opt_blob(Some(file.as_ref())) {
        let rel_url = format!("/files/{}", file_id);
        let _ = js_sys::Reflect::apply(
            &put_fn,
            &cache_js,
            &js_sys::Array::of2(&wasm_bindgen::JsValue::from_str(&rel_url), &resp),
        );

        let full_url = format!("{}/files/{}", gilvave_core::settings::BASE_HTTP_URL, file_id);
        if let Ok(resp_full) = web_sys::Response::new_with_opt_blob(Some(file.as_ref())) {
            let _ = js_sys::Reflect::apply(
                &put_fn,
                &cache_js,
                &js_sys::Array::of2(&wasm_bindgen::JsValue::from_str(&full_url), &resp_full),
            );
        }
    }
}

/// Открытие файла (в Desktop нативно через ОС opener, в Web через браузер)
#[cfg(target_arch = "wasm32")]
pub async fn open_file_card_async(file_id: &str, filename: &str, url: &str) {
    if crate::utils::is_tauri() {
        let download_url = if url.starts_with("http://") || url.starts_with("https://") {
            Some(url.to_string())
        } else if !file_id.is_empty() {
            Some(format!("{}/files/{}", gilvave_core::settings::BASE_HTTP_URL, file_id))
        } else {
            None
        };

        let cmd = gilvave_core::dto::command::CommandArgs::OpenFile {
            file_id: file_id.to_string(),
            filename: filename.to_string(),
            download_url,
        };
        match crate::utils::invoke_command(cmd.to_json()).await {
            gilvave_core::dto::command::CommandResult::Ok(_) => {
                mark_file_cached(file_id);
            }
            gilvave_core::dto::command::CommandResult::Error(err) => {
                web_sys::console::error_1(
                    &format!("[FILE_CARD] Failed to open file: {}", err.message()).into(),
                );
            }
        }
        return;
    }

    // В обычном браузере (Web)
    let Some(window) = web_sys::window() else { return; };
    let target_url = if url.starts_with("http://") || url.starts_with("https://") || url.starts_with("blob:") {
        url.to_string()
    } else if !file_id.is_empty() {
        format!("{}/files/{}", gilvave_core::settings::BASE_HTTP_URL, file_id)
    } else {
        return;
    };
    let _ = window.open_with_url_and_target(&target_url, "_blank");
}


/// Сохранение файла в системную папку Загрузки
#[cfg(target_arch = "wasm32")]
pub async fn save_file_to_downloads_async(file_id: &str, filename: &str, url: &str) -> Result<String, String> {
    use wasm_bindgen::JsCast;

    if crate::utils::is_tauri() {
        let download_url = if url.starts_with("http://") || url.starts_with("https://") {
            Some(url.to_string())
        } else if !file_id.is_empty() {
            Some(format!("{}/files/{}", gilvave_core::settings::BASE_HTTP_URL, file_id))
        } else {
            None
        };

        let cmd = gilvave_core::dto::command::CommandArgs::SaveFileToDownloads {
            file_id: file_id.to_string(),
            filename: filename.to_string(),
            download_url,
        };
        let res = crate::utils::invoke_command(cmd.to_json()).await;
        match res {
            gilvave_core::dto::command::CommandResult::Ok(gilvave_core::dto::command::CommandResponse::SaveFileToDownloads(p)) => {
                mark_file_cached(file_id);
                Ok(p)
            }
            gilvave_core::dto::command::CommandResult::Error(err) => Err(err.message().to_string()),
            _ => Ok(filename.to_string()),
        }
    } else {
        let window = web_sys::window().ok_or_else(|| "No window".to_string())?;
        let document = window.document().ok_or_else(|| "No document".to_string())?;
        let dl_url = if url.starts_with("http://") || url.starts_with("https://") || url.starts_with("blob:") {
            url.to_string()
        } else if !file_id.is_empty() {
            format!("{}/files/{}", gilvave_core::settings::BASE_HTTP_URL, file_id)
        } else {
            return Err("Empty URL".to_string());
        };

        let a = document.create_element("a").map_err(|e| format!("{e:?}"))?;
        let _ = a.set_attribute("href", &dl_url);
        let _ = a.set_attribute("download", filename);
        let _ = a.set_attribute("style", "display: none;");
        if let Some(body) = document.body() {
            let _ = body.append_child(&a);
            if let Ok(html_a) = a.clone().dyn_into::<web_sys::HtmlElement>() {
                html_a.click();
            }
            let _ = body.remove_child(&a);
        }
        Ok(filename.to_string())
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub async fn save_file_to_downloads_async(_file_id: &str, _filename: &str, _url: &str) -> Result<String, String> {
    Ok(String::new())
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub async fn open_file_card_async(_file_id: &str, _filename: &str, _url: &str) {}

#[cfg(not(target_arch = "wasm32"))]
pub async fn cache_uploaded_file_storage(_file_id: &str, _file_name: &str, _file: &web_sys::File) {}
