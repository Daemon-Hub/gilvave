#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

/// Чтение изображения в Data URL
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
