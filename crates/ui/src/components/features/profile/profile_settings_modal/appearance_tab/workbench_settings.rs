//! Workbench appearance settings: theme dropdown, windowed mode toggle, and home screen toggles.

use sycamore::prelude::*;

use crate::components::common::{is_mobile_device, AppTheme, UiModalContext};

#[component(inline_props)]
pub fn WorkbenchSettings() -> View {
    let modal_context = use_context::<UiModalContext>();
    let current_theme = modal_context.app_theme;
    let custom_themes_sig = modal_context.custom_themes;
    let is_windowed_sig = modal_context.is_windowed_mode;
    let is_show_greeting_sig = modal_context.is_show_greeting;
    let is_show_quick_actions_sig = modal_context.is_show_quick_actions;

    let is_theme_dropdown_open = create_signal(false);

    let theme_label = move || match current_theme.get_clone() {
        AppTheme::Advanced => "Gilvave Advanced".to_string(),
        AppTheme::Standard => "Gilvave Standard".to_string(),
        AppTheme::Light => "Gilvave Light".to_string(),
        AppTheme::Custom(id) => {
            if let Some(t) = custom_themes_sig
                .get_clone()
                .into_iter()
                .find(|t| t.id == id)
            {
                format!("{} (by @{})", t.name, t.author)
            } else {
                format!("Пользовательская тема ({id})")
            }
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
                button(
                    r#type="button",
                    class=move || if is_theme_dropdown_open.get() {
                        "custom-select-trigger is-open"
                    } else {
                        "custom-select-trigger"
                    },
                    on:click=move |_| {
                        is_theme_dropdown_open.set(!is_theme_dropdown_open.get());
                    },
                ) {
                    span(class="select-label") { (theme_label()) }
                    svg(
                        class=move || if is_theme_dropdown_open.get() { "select-chevron is-rotated" } else { "select-chevron" },
                        viewBox="0 0 24 24",
                        fill="none",
                        stroke="currentColor",
                        stroke-width="2",
                        stroke-linecap="round",
                        stroke-linejoin="round",
                    ) {
                        polyline(points="6 9 12 15 18 9")
                    }
                }
                (move || if is_theme_dropdown_open.get() {
                    let active = current_theme.get_clone();

                    view! {
                        div(
                            class="custom-select-backdrop",
                            on:click=move |_| is_theme_dropdown_open.set(false),
                        )
                        div(class="custom-select-dropdown") {
                            div(
                                class=if active == AppTheme::Advanced {
                                    "custom-select-option is-selected"
                                } else {
                                    "custom-select-option"
                                },
                                on:click=move |_| {
                                    current_theme.set(AppTheme::Advanced);
                                    is_theme_dropdown_open.set(false);
                                },
                            ) {
                                span(class="option-title") { "Gilvave Advanced" }
                                span(class="option-desc") { "Cosmic Icon Palette — неоновые градиенты и фирменный стиль" }
                            }
                            div(
                                class=if active == AppTheme::Standard {
                                    "custom-select-option is-selected"
                                } else {
                                    "custom-select-option"
                                },
                                on:click=move |_| {
                                    current_theme.set(AppTheme::Standard);
                                    is_theme_dropdown_open.set(false);
                                },
                            ) {
                                span(class="option-title") { "Gilvave Standard" }
                                span(class="option-desc") { "Classic Dark — спокойная тёмная палитра" }
                            }
                            div(
                                class=if active == AppTheme::Light {
                                    "custom-select-option is-selected"
                                } else {
                                    "custom-select-option"
                                },
                                on:click=move |_| {
                                    current_theme.set(AppTheme::Light);
                                    is_theme_dropdown_open.set(false);
                                },
                            ) {
                                span(class="option-title") { "Gilvave Light" }
                                span(class="option-desc") { "Clean Modern Light — светлая контрастная палитра" }
                            }
                            (custom_themes_sig.get_clone().into_iter().map(|t| {
                                let theme_id = t.id.clone();
                                let is_sel = active == AppTheme::Custom(t.id.clone());
                                let name_str = t.name.clone();
                                let author_str = format!("Автор: @{}", t.author);
                                view! {
                                    div(
                                        class=if is_sel { "custom-select-option is-selected" } else { "custom-select-option" },
                                        on:click=move |_| {
                                            current_theme.set(AppTheme::Custom(theme_id.clone()));
                                            is_theme_dropdown_open.set(false);
                                        },
                                    ) {
                                        span(class="option-title") { (name_str) }
                                        span(class="option-desc") { (author_str) }
                                    }
                                }
                            }).collect::<Vec<View>>())
                        }
                    }
                } else {
                    view! {}
                })
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
