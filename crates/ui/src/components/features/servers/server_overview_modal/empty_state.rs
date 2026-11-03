//! Empty states for the server overview modal (no servers in tab, or search query without results).

use sycamore::prelude::*;
use crate::components::{
    common::OverviewTab,
    ui::icons::{GlobeIcon, PlusIcon, SearchIcon, UserIcon},
};

#[component(inline_props)]
pub fn ServerOverviewEmptyState<F1, F2>(
    query: Signal<String>,
    tab: Signal<OverviewTab>,
    on_open_create: F1,
    on_open_join: F2,
) -> View
where
    F1: Fn(web_sys::MouseEvent) + Copy + 'static,
    F2: Fn(web_sys::MouseEvent) + Copy + 'static,
{
    view! {
        (move || {
            let q = query.get_clone();
            let current_tab = tab.get();
            if q.trim().is_empty() {
                match current_tab {
                    OverviewTab::My => view! {
                        div(class="server-overview-empty") {
                            div(class="server-overview-empty-icon") { UserIcon() }
                            h3 { "У вас пока нет своих серверов" }
                            p { "Создайте собственный сервер, чтобы общаться с друзьями и настраивать свои каналы." }
                            button(
                                class="server-overview-empty-btn",
                                on:click=on_open_create,
                            ) {
                                PlusIcon()
                                span { "Создать сервер" }
                            }
                        }
                    },
                    OverviewTab::Saved => view! {
                        div(class="server-overview-empty") {
                            div(class="server-overview-empty-icon") { GlobeIcon() }
                            h3 { "У вас пока нет сохранённых серверов" }
                            p { "Присоединяйтесь к публичным серверам или примите приглашение от сообщества." }
                            button(
                                class="server-overview-empty-btn",
                                on:click=on_open_join,
                            ) {
                                PlusIcon()
                                span { "Найти серверы" }
                            }
                        }
                    },
                }
            } else {
                view! {
                    div(class="server-overview-empty") {
                        div(class="server-overview-empty-icon") { SearchIcon() }
                        h3 { "Ничего не найдено" }
                        p { (format!("По запросу «{}» серверов не найдено.", q)) }
                        button(
                            class="server-overview-empty-btn",
                            on:click=move |_| query.set(String::new()),
                        ) {
                            "Сбросить поиск"
                        }
                    }
                }
            }
        })
    }
}
