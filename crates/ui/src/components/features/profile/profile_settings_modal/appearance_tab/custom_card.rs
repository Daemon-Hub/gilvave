//! Custom theme card: mockup, preview swatches, and actions (edit, upload, export, delete).

use sycamore::{futures::spawn_local_scoped, prelude::*};

use crate::components::common::{AppTheme, CustomTheme, UiModalContext, download_json_file};
use crate::components::ui::icons::{CheckIcon, DownloadIcon, EditIcon, TrashIcon, UploadIcon};
use crate::http::api::Api;
use gilvave_core::dto::users::CreateThemeInfo;

#[component(inline_props)]
pub fn CustomThemeCard(
    theme: CustomTheme,
    theme_status_msg: Signal<String>,
    theme_status_is_error: Signal<bool>,
) -> View {
    let modal_context = use_context::<UiModalContext>();
    let current_theme = modal_context.app_theme;
    let custom_themes_sig = modal_context.custom_themes;

    let theme_id = theme.id.clone();
    let theme_id_click = theme.id.clone();
    let theme_id_active = theme.id.clone();
    let theme_id_check = theme.id.clone();
    let theme_id_delete = theme.id.clone();
    let theme_name_delete = theme.name.clone();
    let theme_for_export = theme.clone();
    let theme_for_edit = theme.clone();
    let theme_for_upload = theme.clone();
    let name = theme.name.clone();
    let author = format!("by @{}", theme.author);
    let desc = if theme.description.trim().is_empty() {
        format!(
            "Пользовательская тема ({}, v{})",
            theme.base, theme.version
        )
    } else {
        theme.description.clone()
    };

    view! {
        div(
            class=move || {
                if current_theme.get_clone() == AppTheme::Custom(theme_id_active.clone()) {
                    "theme-preview-card custom active"
                } else {
                    "theme-preview-card custom"
                }
            },
            data-custom-theme=theme_id,
            on:click=move |_| {
                current_theme.set(AppTheme::Custom(theme_id_click.clone()));
            },
        ) {
            div(class="theme-mockup custom-mockup") {
                div(class="mockup-sidebar") {
                    div(class="mockup-dot home")
                    div(class="mockup-dot")
                    div(class="mockup-dot plus")
                }
                div(class="mockup-channels") {
                    div(class="mockup-line active")
                    div(class="mockup-line")
                    div(class="mockup-line")
                }
                div(class="mockup-chat") {
                    div(class="mockup-msg") {
                        div(class="mockup-avatar")
                        div(class="mockup-bubble")
                    }
                    div(class="mockup-divider")
                    div(class="mockup-input-row") {
                        div(class="mockup-input")
                        div(class="mockup-send")
                    }
                }
            }
            div(class="theme-card-info") {
                div(class="theme-card-title-row") {
                    span(class="theme-card-name") { (name) }
                    span(class="theme-card-badge custom-badge") { (author) }
                    (move || {
                        if current_theme.get_clone() == AppTheme::Custom(theme_id_check.clone()) {
                            view! { span(class="theme-active-check") { CheckIcon() span { "Активна" } } }
                        } else {
                            view! {}
                        }
                    })
                }
                p(class="theme-card-desc") { (desc) }
                div(class="theme-swatches") {
                    span(class="swatch custom-1")
                    span(class="swatch custom-2")
                    span(class="swatch custom-3")
                    span(class="swatch custom-4")
                    span(class="swatch custom-5")
                    span(class="swatch custom-6")
                    span(class="swatch custom-7")
                }
                div(
                    class="theme-card-actions",
                    on:click=move |e: web_sys::MouseEvent| e.stop_propagation(),
                ) {
                    button(
                        class="theme-card-btn",
                        on:click=move |_| {
                            modal_context.theme_json_editor_content.set(theme_for_edit.to_pretty_json());
                            modal_context.is_theme_json_editor_open.set(true);
                            theme_status_msg.set(String::new());
                        },
                    ) {
                        EditIcon()
                        span { "JSON" }
                    }
                    button(
                        class="theme-card-btn",
                        title="Опубликовать тему в каталоге на сервере",
                        on:click=move |_| {
                            let t_name = theme_for_upload.name.clone();
                            let t_desc = theme_for_upload.description.clone();
                            let config_val = match serde_json::to_value(&theme_for_upload) {
                                Ok(v) => v,
                                Err(e) => {
                                    theme_status_is_error.set(true);
                                    theme_status_msg.set(format!("Ошибка сериализации: {e}"));
                                    return;
                                }
                            };
                            let info = CreateThemeInfo {
                                name: t_name.clone(),
                                description: t_desc,
                                config: config_val,
                            };
                            theme_status_is_error.set(false);
                            theme_status_msg.set(format!("Загрузка темы «{t_name}» на сервер..."));
                            spawn_local_scoped(async move {
                                match Api::create_theme(&info).await {
                                    Ok(()) => {
                                        theme_status_is_error.set(false);
                                        theme_status_msg.set(format!("✓ Тема «{t_name}» успешно опубликована на сервере!"));
                                    }
                                    Err(err) => {
                                        theme_status_is_error.set(true);
                                        theme_status_msg.set(format!("Ошибка загрузки: {}", err.1));
                                    }
                                }
                            });
                        },
                    ) {
                        UploadIcon()
                        span { "В каталог" }
                    }
                    button(
                        class="theme-card-btn",
                        on:click=move |_| {
                            let filename = format!(
                                "{}.gilvave-theme.json",
                                theme_for_export.id
                            );
                            let content = theme_for_export.to_pretty_json();
                            download_json_file(&filename, &content);
                        },
                    ) {
                        DownloadIcon()
                        span { "Экспорт" }
                    }
                    button(
                        class="theme-card-btn delete",
                        title="Удалить тему",
                        on:click=move |_| {
                            let mut list = custom_themes_sig.get_clone();
                            list.retain(|t| t.id != theme_id_delete);
                            CustomTheme::save_all(&list);
                            custom_themes_sig.set(list);
                            if current_theme.get_clone() == AppTheme::Custom(theme_id_delete.clone()) {
                                current_theme.set(AppTheme::Advanced);
                            }
                            theme_status_is_error.set(false);
                            theme_status_msg.set(format!(
                                "Тема «{theme_name_delete}» удалена."
                            ));
                        },
                    ) { TrashIcon() }
                }
            }
        }
    }
}
