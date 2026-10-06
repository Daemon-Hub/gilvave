use sycamore::prelude::*;

use crate::components::{
    common::{ServerContext, UiModalContext, classes},
    ui::icons::{CloseSmallIcon, GlobeIcon, LockIcon},
};

#[component]
pub fn ServerSettingsModal() -> View {
    let modal_context = use_context::<UiModalContext>();
    let server_context = use_context::<ServerContext>();

    let name = create_signal(String::new());
    let description = create_signal(String::new());
    let icon_url = create_signal(String::new());
    let cover_url = create_signal(String::new());
    let is_public = create_signal(true);

    // Sync from server_context when opened or changed
    create_effect(move || {
        if let Some(server) = server_context.current.get_clone() {
            name.set(server.name);
            description.set(server.description);
            icon_url.set(server.icon_url);
            cover_url.set(server.cover);
            is_public.set(server.is_public);
        }
    });

    let close = move |_| {
        modal_context.is_server_settings_open.set(false);
    };

    let handle_save = move |_| {
        if let Some(mut server) = server_context.current.get_clone() {
            name.with(|new_name| {
                if !new_name.trim().is_empty() {
                    server.name = new_name.clone();
                }
            });
            server.description = description.get_clone();
            server.icon_url = icon_url.get_clone();
            server.cover = cover_url.get_clone();
            server.is_public = is_public.get();

            let sid = server.id;
            let s_name = server.name.clone();
            let s_icon = server.icon_url.clone();
            server_context.current.set(Some(server));

            // Also update small part in server_context.list
            server_context.list.update(|list| {
                for item in list.iter_mut() {
                    if item.id == sid {
                        item.name = s_name.clone();
                        item.icon_url = s_icon.clone();
                    }
                }
            });
        }
        modal_context.is_server_settings_open.set(false);
    };

    let handle_leave = move |_| {
        server_context.current.set(None);
        modal_context.is_server_settings_open.set(false);
    };

    view! {
        div(
            class="server-modal-overlay",
            on:click=close,
        ) {
            div(
                class="server-modal server-settings-modal",
                on:click=move |e: web_sys::MouseEvent| e.stop_propagation(),
            ) {
                div(class="server-modal-header") {
                    span { "Настройки сервера" }
                    button(class="modal-close-icon-btn", on:click=close, title="Закрыть") { CloseSmallIcon() }
                }

                div(class="server-settings-body") {
                    // Live preview banner + icon
                    div(class="server-preview-banner") {
                        (if cover_url.with(|c| !c.is_empty()) {
                            let c_url = cover_url.get_clone();
                            view! { img(class="server-banner-img", src=c_url, alt="") }
                        } else {
                            view! { div(class="server-banner-placeholder") }
                        })

                        div(class="server-preview-avatar") {
                            (if icon_url.with(|i| !i.is_empty()) {
                                let i_url = icon_url.get_clone();
                                view! { img(src=i_url, alt="") }
                            } else {
                                let initial = name.with(|n| n.chars().next().unwrap_or('?').to_uppercase().to_string());
                                view! { span { (initial) } }
                            })
                        }
                    }

                    div(class="input-group") {
                        label { "НАЗВАНИЕ СЕРВЕРА" }
                        input(
                            r#type="text",
                            placeholder="Название сервера",
                            bind:value=name,
                        )
                    }

                    div(class="input-group") {
                        label { "ОПИСАНИЕ СЕРВЕРА" }
                        textarea(
                            rows="3",
                            placeholder="Расскажите о вашем сервере...",
                            bind:value=description,
                        )
                    }

                    div(class="input-group") {
                        label { "URL ИКОНКИ" }
                        input(
                            r#type="text",
                            placeholder="https://example.com/icon.png",
                            bind:value=icon_url,
                        )
                    }

                    div(class="input-group") {
                        label { "URL ОБЛОЖКИ (БАННЕРА)" }
                        input(
                            r#type="text",
                            placeholder="https://example.com/banner.png",
                            bind:value=cover_url,
                        )
                    }

                    div(class="create-form-tabs") {
                        div(class="toggle-wrapper") {
                            div(class="toggle-tabs") {
                                div(
                                    class=classes(vec![
                                        "toggle-tab".into(),
                                        ("active", { !is_public.get() }.into()).into(),
                                    ]),
                                    on:click=move |_| is_public.set(false),
                                ) {
                                    LockIcon()
                                    span { "Приватный" }
                                }
                                div(
                                    class=classes(vec![
                                        "toggle-tab".into(),
                                        ("active", { is_public.get() }.into()).into(),
                                    ]),
                                    on:click=move |_| is_public.set(true),
                                ) {
                                    GlobeIcon()
                                    span { "Публичный" }
                                }
                                div(
                                    class=classes(vec![
                                        "floating-bg".into(),
                                        ("public", { is_public.get() }.into()).into(),
                                    ]),
                                )
                            }
                        }
                    }

                    div(class="server-settings-danger") {
                        button(class="danger-btn", on:click=handle_leave) { "Покинуть сервер" }
                    }

                    div(class="create-form-actions modal-footer-buttons") {
                        button(class="back-btn", on:click=close) { "Отмена" }
                        button(
                            class="submit-btn create-submit",
                            on:click=handle_save,
                        ) { "Сохранить" }
                    }
                }
            }
        }
    }
}
