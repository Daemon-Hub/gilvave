//! Profile edit tab with live preview card and input fields.

use sycamore::prelude::*;

#[component(inline_props)]
pub fn ProfileTab<F1, F2>(
    name_input: Signal<String>,
    avatar_input: Signal<String>,
    banner_input: Signal<String>,
    bio_input: Signal<String>,
    on_cancel: F1,
    on_save: F2,
) -> View
where
    F1: Fn() + Copy + 'static,
    F2: Fn() + Copy + 'static,
{
    let handle_save = move |_| on_save();
    let close = move |_| on_cancel();

    view! {
                                div(class="tab-pane") {
                                    h3(class="tab-title") { "Мой профиль" }

                                    // Live preview card
                                    div(class="profile-card-preview") {
                                        div(class="profile-preview-banner") {
                                            (if banner_input.with(|b| !b.is_empty()) {
                                                let b_url = banner_input.get_clone();
                                                view! { img(src=b_url, alt="") }
                                            } else {
                                                view! { div(class="default-banner") }
                                            })
                                        }

                                        div(class="profile-preview-header") {
                                            div(class="profile-preview-avatar") {
                                                (if avatar_input.with(|a| !a.is_empty()) {
                                                    let a_url = avatar_input.get_clone();
                                                    view! { img(src=a_url, alt="") }
                                                } else {
                                                    let initial = name_input.with(|n| n.chars().next().unwrap_or('?').to_uppercase().to_string());
                                                    view! { span { (initial) } }
                                                })
                                                div(class="status-dot online")
                                            }
                                            div(class="profile-preview-names") {
                                                span(class="preview-name") { (name_input.get_clone()) }
                                                span(class="preview-tag") { "@" (name_input.with(|n| n.to_lowercase())) }
                                            }
                                        }

                                        (if bio_input.with(|b| !b.is_empty()) {
                                            let bio = bio_input.get_clone();
                                            view! {
                                                div(class="profile-preview-bio") {
                                                    span(class="bio-label") { "О СЕБЕ" }
                                                    p { (bio) }
                                                }
                                            }
                                        } else {
                                            view! {}
                                        })
                                    }

                                    div(class="input-group") {
                                        label { "ОТОБРАЖАЕМОЕ ИМЯ" }
                                        input(r#type="text", placeholder="Ваше имя", bind:value=name_input)
                                    }

                                    div(class="input-group") {
                                        label { "URL АВАТАРКИ" }
                                        input(r#type="text", placeholder="https://example.com/avatar.png", bind:value=avatar_input)
                                    }

                                    div(class="input-group") {
                                        label { "URL БАННЕРА" }
                                        input(r#type="text", placeholder="https://example.com/banner.png", bind:value=banner_input)
                                    }

                                    div(class="input-group") {
                                        label { "О СЕБЕ" }
                                        textarea(rows="2", placeholder="Расскажите немного о себе...", bind:value=bio_input)
                                    }

                                    div(class="create-form-actions modal-footer-buttons") {
                                        button(class="back-btn", on:click=close) { "Отмена" }
                                        button(class="submit-btn create-submit", on:click=handle_save) { "Сохранить изменения" }
                                    }
                                }

    }
}
