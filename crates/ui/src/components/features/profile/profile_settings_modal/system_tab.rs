//! System preferences tab (Close-to-tray desktop toggle).

use sycamore::prelude::*;
use crate::components::common::UiModalContext;

#[component]
pub fn SystemTab() -> View {
    let modal_context = use_context::<UiModalContext>();
    let is_close_to_tray_sig = modal_context.is_close_to_tray;

    view! {
                                div(class="tab-pane system-settings-pane") {
                                    h3(class="tab-title") { "Системные настройки" }

                                    (if !crate::components::common::is_mobile_device() {
                                        view! {
                                            div(class="vscode-setting-item") {
                                                div(class="vscode-setting-header") {
                                                    span(class="vscode-setting-category") { "Workbench › System: " }
                                                    span(class="vscode-setting-name") { "Close to Tray" }
                                                }
                                                p(class="vscode-setting-desc") {
                                                    "Сворачивать окно в системный трей при нажатии на кнопку закрытия (крестик). Если отключено, приложение полностью завершает работу при закрытии."
                                                }
                                                div(
                                                    class="vscode-toggle-row",
                                                    on:click=move |_| {
                                                        is_close_to_tray_sig.set(!is_close_to_tray_sig.get());
                                                    },
                                                ) {
                                                    div(
                                                        class=move || {
                                                            if is_close_to_tray_sig.get() {
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
                                                            if is_close_to_tray_sig.get() {
                                                                "Сворачивать в системный трей при закрытии"
                                                            } else {
                                                                "Полностью закрывать приложение при нажатии на крестик"
                                                            }
                                                        })
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                        view! {
                                            div(class="vscode-setting-item") {
                                                div(class="vscode-setting-header") {
                                                    span(class="vscode-setting-category") { "Workbench › System: " }
                                                    span(class="vscode-setting-name") { "Системный трей" }
                                                }
                                                p(class="vscode-setting-desc") {
                                                    "Настройки системного трея и поведения окон доступны в десктопной версии приложения."
                                                }
                                            }
                                        }
                                    })
                                }

    }
}
