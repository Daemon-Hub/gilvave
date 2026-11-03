//! Built-in theme cards: Advanced (Cosmic Icon), Standard (Classic Dark), and Light.

use sycamore::prelude::*;

use crate::components::common::{AppTheme, UiModalContext};
use crate::components::ui::icons::CheckIcon;

#[component(inline_props)]
pub fn BuiltinThemeCards() -> View {
    let modal_context = use_context::<UiModalContext>();
    let current_theme = modal_context.app_theme;

    view! {
        // Advanced Theme Card
        div(
            class=move || {
                if current_theme.get_clone() == AppTheme::Advanced {
                    "theme-preview-card advanced active"
                } else {
                    "theme-preview-card advanced"
                }
            },
            on:click=move |_| current_theme.set(AppTheme::Advanced),
        ) {
            div(class="theme-mockup advanced-mockup") {
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
                    span(class="theme-card-name") { "Advanced" }
                    span(class="theme-card-badge advanced-badge") { "Cosmic Icon" }
                    (move || if current_theme.get_clone() == AppTheme::Advanced {
                        view! { span(class="theme-active-check") { CheckIcon() span { "Активна" } } }
                    } else {
                        view! {}
                    })
                }
                p(class="theme-card-desc") {
                    "Глубокий космический индиго, неоновое кольцо (циан, индиго, маджента) и тёплая импульсная волна (коралл, оранжевый, янтарь) с иконки Gilvave."
                }
                div(class="theme-swatches") {
                    span(class="swatch adv-1")
                    span(class="swatch adv-2")
                    span(class="swatch adv-3")
                    span(class="swatch adv-4")
                    span(class="swatch adv-5")
                    span(class="swatch adv-6")
                    span(class="swatch adv-7")
                }
            }
        }

        // Standard Theme Card
        div(
            class=move || {
                if current_theme.get_clone() == AppTheme::Standard {
                    "theme-preview-card standard active"
                } else {
                    "theme-preview-card standard"
                }
            },
            on:click=move |_| current_theme.set(AppTheme::Standard),
        ) {
            div(class="theme-mockup standard-mockup") {
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
                    span(class="theme-card-name") { "Standard" }
                    span(class="theme-card-badge standard-badge") { "Classic Dark" }
                    (move || if current_theme.get_clone() == AppTheme::Standard {
                        view! { span(class="theme-active-check") { CheckIcon() span { "Активна" } } }
                    } else {
                        view! {}
                    })
                }
                p(class="theme-card-desc") {
                    "Классическая тёмно-серая палитра панелей (#36393f / #2f3136 / #202225) со сдержанными лавандово-розовыми акцентами."
                }
                div(class="theme-swatches") {
                    span(class="swatch std-1")
                    span(class="swatch std-2")
                    span(class="swatch std-3")
                    span(class="swatch std-4")
                    span(class="swatch std-5")
                    span(class="swatch std-6")
                    span(class="swatch std-7")
                }
            }
        }

        // Light Theme Card
        div(
            class=move || {
                if current_theme.get_clone() == AppTheme::Light {
                    "theme-preview-card light active"
                } else {
                    "theme-preview-card light"
                }
            },
            on:click=move |_| current_theme.set(AppTheme::Light),
        ) {
            div(class="theme-mockup light-mockup") {
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
                    span(class="theme-card-name") { "Light" }
                    span(class="theme-card-badge light-badge") { "Clean Light" }
                    (move || if current_theme.get_clone() == AppTheme::Light {
                        view! { span(class="theme-active-check") { CheckIcon() span { "Активна" } } }
                    } else {
                        view! {}
                    })
                }
                p(class="theme-card-desc") {
                    "Светлая, чистая и современная тема с мягким контрастом, сине-фиолетовыми акцентами и отличной читаемостью."
                }
                div(class="theme-swatches") {
                    span(class="swatch light-1")
                    span(class="swatch light-2")
                    span(class="swatch light-3")
                    span(class="swatch light-4")
                    span(class="swatch light-5")
                    span(class="swatch light-6")
                    span(class="swatch light-7")
                }
            }
        }
    }
}
