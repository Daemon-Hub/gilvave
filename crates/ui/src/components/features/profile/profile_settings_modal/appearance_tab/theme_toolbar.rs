//! Toolbar for custom themes: catalog button, JSON file upload, editor launcher, and template export.

use sycamore::prelude::*;
use wasm_bindgen::JsCast;

use crate::components::common::{CustomTheme, UiModalContext, download_json_file};
use crate::components::ui::icons::{CatalogIcon, CodeIcon, DownloadIcon, FileTextIcon};

#[component(inline_props)]
pub fn ThemeToolbar(
    theme_status_msg: Signal<String>,
    theme_status_is_error: Signal<bool>,
) -> View {
    let modal_context = use_context::<UiModalContext>();
    let current_theme = modal_context.app_theme;
    let custom_themes_sig = modal_context.custom_themes;

    #[allow(unused_variables)]
    let install_custom_theme_from_json = move |raw_json: String| {
        match CustomTheme::from_json_str(&raw_json) {
            Ok(new_theme) => {
                let new_id = new_theme.id.clone();
                let new_name = new_theme.name.clone();
                let mut list = custom_themes_sig.get_clone();
                if let Some(existing) = list.iter_mut().find(|t| t.id == new_id) {
                    *existing = new_theme;
                } else {
                    list.push(new_theme);
                }
                CustomTheme::save_all(&list);
                custom_themes_sig.set(list);
                current_theme.set(crate::components::common::AppTheme::Custom(new_id));
                if modal_context.is_profile_settings_open.get() {
                    theme_status_is_error.set(false);
                    theme_status_msg.set(format!(
                        "✓ Тема «{new_name}» успешно установлена и активирована!"
                    ));
                }
            }
            Err(err_msg) => {
                if modal_context.is_profile_settings_open.get() {
                    theme_status_is_error.set(true);
                    theme_status_msg.set(err_msg);
                }
            }
        }
    };

    let on_file_upload = move |e: web_sys::Event| {
        if let Some(target) = e.target()
            && let Ok(input) = target.dyn_into::<web_sys::HtmlInputElement>()
        {
            if let Some(files) = input.files()
                && let Some(file) = files.get(0)
            {
                #[allow(unused_variables)]
                let promise = file.text();
                #[cfg(target_arch = "wasm32")]
                wasm_bindgen_futures::spawn_local(async move {
                    if let Ok(js_val) = wasm_bindgen_futures::JsFuture::from(promise).await
                        && let Some(content) = js_val.as_string()
                    {
                        install_custom_theme_from_json(content);
                    }
                });
            }
            input.set_value("");
        }
    };

    let on_download_template = move |_| {
        let tpl = CustomTheme::template_json();
        download_json_file("my-theme.gilvave-theme.json", &tpl);
    };

    view! {
        div(class="custom-themes-toolbar") {
            button(
                class="toolbar-btn primary",
                on:click=move |_| {
                    modal_context.is_theme_catalog_open.set(true);
                },
            ) {
                CatalogIcon()
                span { "Каталог тем" }
            }
            label(
                r#for="custom-theme-file-input",
                class="toolbar-btn",
            ) {
                DownloadIcon()
                span { "Загрузить тему (.json)" }
            }
            input(
                id="custom-theme-file-input",
                r#type="file",
                accept=".json",
                class="hidden-file-input",
                on:change=on_file_upload,
            )
            button(
                class="toolbar-btn",
                on:click=move |_| {
                    modal_context.theme_json_editor_content.set(CustomTheme::template_json());
                    modal_context.is_theme_json_editor_open.set(true);
                },
            ) {
                CodeIcon()
                span { "Открыть JSON-редактор" }
            }
            button(
                class="toolbar-btn",
                on:click=on_download_template,
            ) {
                FileTextIcon()
                span { "Скачать шаблон (.json)" }
            }
        }

        // Status / validation banner
        (move || {
            let msg = theme_status_msg.get_clone();
            if msg.is_empty() {
                view! {}
            } else {
                let cls = if theme_status_is_error.get() {
                    "theme-status-banner error"
                } else {
                    "theme-status-banner"
                };
                view! {
                    div(class=cls) { (msg) }
                }
            }
        })
    }
}
