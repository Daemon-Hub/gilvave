use sycamore::prelude::*;
use wasm_bindgen::JsCast;

use crate::components::{
    common::{download_json_file, AppTheme, CustomTheme, UiModalContext},
    ui::{
        icons::{BookOpenIcon, CloseSmallIcon, CodeIcon, DownloadIcon, FileTextIcon},
        markdown::MarkdownView,
    },
};

const GUIDE_CONTENT: &str = include_str!("../../../../../../CUSTOM_THEMES_GUIDE.md");

#[component]
pub fn ThemeJsonEditorModal() -> View {
    let modal_context = use_context::<UiModalContext>();
    let current_theme = modal_context.app_theme;
    let custom_themes_sig = modal_context.custom_themes;

    let initial_content = {
        let preset = modal_context.theme_json_editor_content.get_clone();
        if !preset.trim().is_empty() {
            preset
        } else {
            CustomTheme::template_json()
        }
    };

    let json_input = create_signal(initial_content);
    let status_msg = create_signal(String::new());
    let is_error = create_signal(false);
    let is_guide_open = create_signal(false);

    let close_modal = move |_| {
        modal_context.is_theme_json_editor_open.set(false);
    };

    let on_reset_template = move |_| {
        json_input.set(CustomTheme::template_json());
        status_msg.set(String::new());
        is_error.set(false);
    };

    let on_download_template = move |_| {
        let tpl = CustomTheme::template_json();
        download_json_file("template.gilvave-theme.json", &tpl);
    };

    let on_file_upload = move |e: web_sys::Event| {
        if let Some(target) = e.target() {
            if let Ok(input) = target.dyn_into::<web_sys::HtmlInputElement>() {
                if let Some(files) = input.files() {
                    if let Some(file) = files.get(0) {
                        let promise = file.text();
                        wasm_bindgen_futures::spawn_local(async move {
                            if let Ok(js_val) =
                                wasm_bindgen_futures::JsFuture::from(promise).await
                            {
                                if let Some(content) = js_val.as_string() {
                                    json_input.set(content);
                                    status_msg.set(String::new());
                                    is_error.set(false);
                                }
                            }
                        });
                    }
                }
                input.set_value("");
            }
        }
    };

    let handle_apply = move || {
        let raw = json_input.get_clone();
        match CustomTheme::from_json_str(&raw) {
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
                current_theme.set(AppTheme::Custom(new_id));
                is_error.set(false);
                status_msg.set(format!(
                    "✓ Тема «{new_name}» успешно установлена и активирована!"
                ));
            }
            Err(err_msg) => {
                is_error.set(true);
                status_msg.set(err_msg);
            }
        }
    };

    let on_apply_click = move |_| {
        handle_apply();
    };

    let on_keydown = move |e: web_sys::KeyboardEvent| {
        if e.key() == "Escape" {
            modal_context.is_theme_json_editor_open.set(false);
        } else if (e.ctrl_key() || e.meta_key()) && e.key() == "Enter" {
            handle_apply();
        }
    };

    let modal_class = create_memo(move || {
        if is_guide_open.get() {
            "server-modal theme-json-editor-modal with-guide"
        } else {
            "server-modal theme-json-editor-modal"
        }
    });

    view! {
        div(
            class="server-modal-overlay large theme-json-editor-overlay",
            on:click=close_modal,
            on:keydown=on_keydown,
        ) {
            div(
                class=modal_class.get(),
                on:click=move |e: web_sys::MouseEvent| e.stop_propagation(),
            ) {
                div(class="server-modal-header") {
                    span {
                        CodeIcon()
                        span { "JSON-редактор темы" }
                    }
                    button(
                        class="modal-close-icon-btn",
                        on:click=close_modal,
                        title="Закрыть",
                    ) { CloseSmallIcon() }
                    p(class="theme-json-editor-subtitle") {
                        "Создавайте или редактируйте темы в формате JSON. Поддерживаются блок палитры palette (19 ключевых цветов) и прямые CSS-переменные (--color-*, --gradient-*)."
                    }
                }

                div(class="theme-json-editor-body") {
                    div(class="theme-editor-code-col") {
                        div(class="theme-json-editor-toolbar") {
                            button(
                                class="toolbar-btn",
                                on:click=on_reset_template,
                                title="Сбросить содержимое редактора к исходному шаблону",
                            ) {
                                FileTextIcon()
                                span { "Сбросить к шаблону" }
                            }
                            button(
                                class="toolbar-btn",
                                on:click=on_download_template,
                                title="Скачать файл шаблона на диск",
                            ) {
                                DownloadIcon()
                                span { "Скачать шаблон (.json)" }
                            }
                            label(
                                r#for="theme-editor-file-input",
                                class="toolbar-btn",
                                title="Загрузить JSON-файл темы с компьютера",
                            ) {
                                DownloadIcon()
                                span { "Загрузить из файла (.json)" }
                            }
                            input(
                                id="theme-editor-file-input",
                                r#type="file",
                                accept=".json",
                                class="hidden-file-input",
                                style="display: none;",
                                on:change=on_file_upload,
                            )
                            button(
                                class=if is_guide_open.get() { "toolbar-btn active" } else { "toolbar-btn" },
                                on:click=move |_| is_guide_open.set(!is_guide_open.get()),
                                title="Открыть или скрыть руководство по созданию тем",
                            ) {
                                BookOpenIcon()
                                span { (if is_guide_open.get() { "Скрыть справку" } else { "Справка" }) }
                            }
                        }

                        (move || {
                            let msg = status_msg.get_clone();
                            if msg.is_empty() {
                                view! {}
                            } else {
                                let cls = if is_error.get() {
                                    "theme-status-banner error"
                                } else {
                                    "theme-status-banner success"
                                };
                                view! {
                                    div(class=cls) { (msg) }
                                }
                            }
                        })

                        textarea(
                            class="json-theme-textarea",
                            rows="15",
                            placeholder="Вставьте валидный JSON-код темы...",
                            bind:value=json_input,
                        )
                    }

                    (if is_guide_open.get() {
                        view! {
                            div(class="theme-editor-guide-col") {
                                div(class="theme-editor-guide-header") {
                                    div(class="theme-editor-guide-title") {
                                        BookOpenIcon()
                                        span { "Руководство по созданию тем" }
                                    }
                                    button(
                                        class="guide-close-btn",
                                        on:click=move |_| is_guide_open.set(false),
                                        title="Скрыть панель справки",
                                    ) {
                                        CloseSmallIcon()
                                    }
                                }
                                div(class="theme-editor-guide-scroll") {
                                    MarkdownView(content=GUIDE_CONTENT.to_string())
                                }
                            }
                        }
                    } else {
                        view! {}
                    })
                }

                div(class="theme-json-editor-footer modal-footer-buttons") {
                    span(class="theme-json-editor-hint") {
                        "Горячая клавиша: Ctrl+Enter для быстрой установки"
                    }
                    div(class="theme-json-editor-actions") {
                        button(class="back-btn", on:click=close_modal) { "Закрыть" }
                        button(
                            class="submit-btn create-submit",
                            on:click=on_apply_click,
                        ) {
                            span { "Установить и применить" }
                        }
                    }
                }
            }
        }
    }
}
