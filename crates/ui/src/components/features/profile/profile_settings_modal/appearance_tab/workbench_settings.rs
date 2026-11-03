//! Workbench appearance settings: theme dropdown, windowed mode toggle, and home screen toggles.

use sycamore::prelude::*;
use wasm_bindgen::JsCast;

use crate::components::common::{AppTheme, UiModalContext, is_mobile_device};

#[component(inline_props)]
pub fn WorkbenchSettings() -> View {
    let modal_context = use_context::<UiModalContext>();
    let current_theme = modal_context.app_theme;
    let custom_themes_sig = modal_context.custom_themes;
    let is_windowed_sig = modal_context.is_windowed_mode;
    let is_show_greeting_sig = modal_context.is_show_greeting;
    let is_show_quick_actions_sig = modal_context.is_show_quick_actions;

    let on_select_change = move |e: web_sys::Event| {
        if let Some(target) = e.target()
            && let Ok(select) = target.dyn_into::<web_sys::HtmlSelectElement>()
        {
            current_theme.set(AppTheme::from_id(&select.value()));
        }
    };

    view! {
        // Color Theme Select
        div(class="vscode-setting-item") {
            div(class="vscode-setting-header") {
                span(class="vscode-setting-category") { "Workbench › Appearance: " }
                span(class="vscode-setting-name") { "Color Theme" }
            }
            p(class="vscode-setting-desc") {
                "Выберите встроенную или установленную пользовательскую тему интерфейса Gilvave. Как в VS Code, вы можете скачивать темы из каталога или загружать JSON-файлы."
            }
            div(class="vscode-select-wrapper") {
                select(
                    class="vscode-theme-select",
                    on:change=on_select_change,
                ) {
                    option(
                        value="advanced",
                        selected=move || current_theme.get_clone() == AppTheme::Advanced,
                    ) {
                        "Gilvave Advanced (Cosmic Icon Palette)"
                    }
                    option(
                        value="standard",
                        selected=move || current_theme.get_clone() == AppTheme::Standard,
                    ) {
                        "Gilvave Standard (Classic Dark)"
                    }
                    option(
                        value="light",
                        selected=move || current_theme.get_clone() == AppTheme::Light,
                    ) {
                        "Gilvave Light (Clean Modern Light)"
                    }
                    (move || {
                        let list = custom_themes_sig.get_clone();
                        let active = current_theme.get_clone();
                        let opts: Vec<View> = list
                            .into_iter()
                            .map(|t| {
                                let val = t.id.clone();
                                let is_sel = active == AppTheme::Custom(t.id.clone());
                                let label = format!("{} (by @{})", t.name, t.author);
                                view! {
                                    option(value=val, selected=is_sel) { (label) }
                                }
                            })
                            .collect();
                        opts
                    })
                }
            }
        }

        // Windowed Mode vs Frameless Fullbleed Toggle
        div(class="vscode-setting-item") {
            div(class="vscode-setting-header") {
                span(class="vscode-setting-category") { "Workbench › Appearance: " }
                span(class="vscode-setting-name") { "Windowed Mode (Floating Frame)" }
            }
            p(class="vscode-setting-desc") {
                "Отображать отступы и фоновое свечение вокруг рабочей панели. Отключите, чтобы рабочая область заполнила весь экран без зазоров по краям."
            }
            div(
                class="vscode-toggle-row",
                on:click=move |_| {
                    if !is_mobile_device() {
                        is_windowed_sig.set(!is_windowed_sig.get());
                    }
                },
            ) {
                div(
                    class=move || {
                        if is_mobile_device() {
                            "vscode-switch disabled"
                        } else if is_windowed_sig.get() {
                            "vscode-switch active"
                        } else {
                            "vscode-switch"
                        }
                    },
                ) {
                    div(class="vscode-switch-thumb")
                }
                span(class="vscode-switch-label") {
                    (move || {
                        if is_mobile_device() {
                            "Оконный режим автоматически отключён на мобильных устройствах"
                        } else if is_windowed_sig.get() {
                            "Оконный режим включён (с отступами и фоном по краям)"
                        } else {
                            "Оконный режим отключён (панель заполняет всё пространство без зазоров)"
                        }
                    })
                }
            }
        }

        // Show Greeting Banner Toggle
        div(class="vscode-setting-item") {
            div(class="vscode-setting-header") {
                span(class="vscode-setting-category") { "Workbench › Home: " }
                span(class="vscode-setting-name") { "Greeting Banner" }
            }
            p(class="vscode-setting-desc") {
                "Отображать приветственный баннер на главном экране."
            }
            div(
                class="vscode-toggle-row",
                on:click=move |_| {
                    is_show_greeting_sig.set(!is_show_greeting_sig.get());
                },
            ) {
                div(
                    class=move || {
                        if is_show_greeting_sig.get() {
                            "vscode-switch active"
                        } else {
                            "vscode-switch"
                        }
                    },
                ) {
                    div(class="vscode-switch-thumb")
                }
                span(class="vscode-switch-label") {
                    (move || {
                        if is_show_greeting_sig.get() {
                            "Баннер приветствия отображается"
                        } else {
                            "Баннер приветствия скрыт"
                        }
                    })
                }
            }
        }

        // Show Quick Access Toggle
        div(class="vscode-setting-item") {
            div(class="vscode-setting-header") {
                span(class="vscode-setting-category") { "Workbench › Home: " }
                span(class="vscode-setting-name") { "Quick Access Cards" }
            }
            p(class="vscode-setting-desc") {
                "Отображать карточки быстрого доступа (создание сервера, каталог сообществ) на главном экране."
            }
            div(
                class="vscode-toggle-row",
                on:click=move |_| {
                    is_show_quick_actions_sig.set(!is_show_quick_actions_sig.get());
                },
            ) {
                div(
                    class=move || {
                        if is_show_quick_actions_sig.get() {
                            "vscode-switch active"
                        } else {
                            "vscode-switch"
                        }
                    },
                ) {
                    div(class="vscode-switch-thumb")
                }
                span(class="vscode-switch-label") {
                    (move || {
                        if is_show_quick_actions_sig.get() {
                            "Карточки быстрого доступа отображаются"
                        } else {
                            "Карточки быстрого доступа скрыты"
                        }
                    })
                }
            }
        }
    }
}
