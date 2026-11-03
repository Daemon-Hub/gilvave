//! Standalone modal for browsing and installing themes from the backend server catalog.

use sycamore::{futures::spawn_local_scoped, prelude::*};
use wasm_bindgen::JsCast;

use crate::components::common::{AppTheme, CustomTheme, UiModalContext, download_json_file};
use crate::components::ui::ShowMoreBanner;
use crate::components::ui::icons::{
    ArrowLeftIcon, CatalogIcon, CheckIcon, CloseSmallIcon, DownloadIcon,
};
use crate::http::api::Api;

#[component]
pub fn ThemeCatalogModal() -> View {
    let modal_context = use_context::<UiModalContext>();
    let current_theme = modal_context.app_theme;
    let custom_themes_sig = modal_context.custom_themes;

    let search_query = create_signal(String::new());
    let server_themes = create_signal::<Vec<CustomTheme>>(vec![]);
    let current_page = create_signal::<i64>(1);
    let has_more = create_signal::<bool>(false);
    let is_loading = create_signal::<bool>(false);
    let is_at_bottom = create_signal::<bool>(false);
    let body_ref = create_node_ref();

    let is_banner_visible = create_signal(false);
    create_effect(move || {
        let visible = has_more.get() && (is_at_bottom.get() || is_loading.get());
        is_banner_visible.set(visible);
    });

    // Keep preview CSS synced for any themes returned from the server catalog
    create_effect(move || {
        let mut combined = custom_themes_sig.get_clone();
        for theme in server_themes.get_clone() {
            if !combined.iter().any(|t| t.id == theme.id) {
                combined.push(theme);
            }
        }
        CustomTheme::sync_preview_styles(&combined);
    });

    // Fetch initial public themes from the server
    is_loading.set(true);
    spawn_local_scoped(async move {
        if let Ok((themes, more)) = Api::get_themes_catalog(1).await {
            let customs: Vec<CustomTheme> = themes.into_iter().map(CustomTheme::from).collect();
            if modal_context.is_theme_catalog_open.get() {
                server_themes.set(customs);
                has_more.set(more);
            }
        }
        is_loading.set(false);
    });

    let check_bottom = move || {
        if let Some(node) = body_ref.try_get() {
            let el = node.unchecked_into::<web_sys::HtmlElement>();
            let top = el.scroll_top();
            let sh = el.scroll_height();
            let ch = el.client_height();
            let distance = sh - ch - top;
            is_at_bottom.set(distance <= 64);
        }
    };

    let on_scroll = move |_| {
        check_bottom();
    };

    let handle_load_more = move |_| {
        if is_loading.get() || !has_more.get() {
            return;
        }
        is_loading.set(true);
        let next_page = current_page.get() + 1;
        spawn_local_scoped(async move {
            if let Ok((new_themes, more)) = Api::get_themes_catalog(next_page).await {
                let new_customs: Vec<CustomTheme> =
                    new_themes.into_iter().map(CustomTheme::from).collect();
                let mut current = server_themes.get_clone();
                current.extend(new_customs);
                server_themes.set(current);
                current_page.set(next_page);
                has_more.set(more);
                is_at_bottom.set(false);
            }
            is_loading.set(false);
        });
    };

    let close_catalog = move |_| {
        modal_context.is_theme_catalog_open.set(false);
    };

    view! {
        div(
            class="server-modal-overlay large theme-catalog-overlay",
            on:click=close_catalog,
        ) {
            div(
                class="server-modal join-modal theme-catalog-modal",
                on:click=move |e: web_sys::MouseEvent| e.stop_propagation(),
            ) {
                div(class="server-modal-header") {
                    span { CatalogIcon()
                                            span { "Каталог тем" } }
                    button(
                        class="modal-close-icon-btn",
                        on:click=close_catalog,
                        title="Закрыть",
                    ) { CloseSmallIcon() }
                    div(class="join-modal-search theme-catalog-search") {
                        span(class="theme-catalog-search-icon") {
                            svg(viewBox="0 0 24 24") {
                                circle(cx="11", cy="11", r="7")
                                line(x1="20", y1="20", x2="16.35", y2="16.35")
                            }
                        }
                        input(
                            r#type="text",
                            placeholder="Поиск тем по названию, автору или описанию...",
                            bind:value=search_query,
                        )
                    }
                }

                div(
                    class="join-modal-body theme-catalog-body",
                    r#ref=body_ref,
                    on:scroll=on_scroll,
                ) {
                    (move || {
                        let query = search_query.get_clone().trim().to_lowercase();
                        let filtered: Vec<CustomTheme> = server_themes
                            .get_clone()
                            .into_iter()
                            .filter(|t| {
                                if query.is_empty() {
                                    true
                                } else {
                                    t.name.to_lowercase().contains(&query)
                                        || t.author.to_lowercase().contains(&query)
                                        || t.description.to_lowercase().contains(&query)
                                }
                            })
                            .collect();

                        if filtered.is_empty() {
                            view! {
                                div(class="join-modal-empty") {
                                    span { "По вашему запросу темы не найдены" }
                                }
                            }
                        } else {
                            let cards: Vec<View> = filtered
                                .into_iter()
                                .map(|cat_theme| {
                                    let tid = cat_theme.id.clone();
                                    let tid_check = cat_theme.id.clone();
                                    let theme_for_install = cat_theme.clone();
                                    let theme_for_download = cat_theme.clone();
                                    let name = cat_theme.name.clone();
                                    let author = format!("by @{}", cat_theme.author);
                                    let desc = cat_theme.description.clone();

                                    view! {
                                        div(
                                            class="theme-preview-card custom",
                                            data-custom-theme=tid,
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
                                                div(class="theme-card-actions") {
                                                    button(
                                                        class="theme-card-btn install",
                                                        on:click=move |_| {
                                                            let new_theme = theme_for_install.clone();
                                                            let new_id = new_theme.id.clone();
                                                            let mut list = custom_themes_sig.get_clone();
                                                            if let Some(existing) = list.iter_mut().find(|t| t.id == new_id) {
                                                                *existing = new_theme;
                                                            } else {
                                                                list.push(new_theme);
                                                            }
                                                            CustomTheme::save_all(&list);
                                                            custom_themes_sig.set(list);
                                                            current_theme.set(AppTheme::Custom(new_id));
                                                        },
                                                    ) {
                                                        (move || {
                                                            let installed = custom_themes_sig
                                                                .get_clone()
                                                                .iter()
                                                                .any(|t| t.id == tid_check);
                                                            if installed {
                                                                view! { CheckIcon() span { "Установлена (Применить)" } }
                                                            } else {
                                                                view! { DownloadIcon() span { "Установить тему" } }
                                                            }
                                                        })
                                                    }
                                                    button(
                                                        class="theme-card-btn",
                                                        on:click=move |_| {
                                                            let filename = format!(
                                                                "{}.gilvave-theme.json",
                                                                theme_for_download.id
                                                            );
                                                            let content = theme_for_download.to_pretty_json();
                                                            download_json_file(&filename, &content);
                                                        },
                                                    ) {
                                                                                        DownloadIcon()
                                                                                        span { "Скачать .json" }
                                                                                    }
                                                }
                                            }
                                        }
                                    }
                                })
                                .collect();

                            view! {
                                div(class="theme-cards-grid") {
                                    (cards)
                                }
                            }
                        }
                    })

                }

                ShowMoreBanner(
                    is_visible=is_banner_visible,
                    is_loading=is_loading,
                    on_click=handle_load_more,
                )

                div(class="join-modal-footer") {
                    button(class="back-btn", on:click=close_catalog) {
                        ArrowLeftIcon()
                        span { "Назад" }
                    }
                }
            }
        }
    }
}
