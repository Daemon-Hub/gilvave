//! Grid items for the server overview modal (server cards and add tile).

use sycamore::prelude::*;
use gilvave_core::dto::servers::ServerSmallPart;
use crate::components::ui::icons::PlusIcon;

pub fn get_server_initials(name: &str) -> String {
    let words: Vec<&str> = name.split_whitespace().collect();
    if words.len() >= 2 {
        let first = words[0].chars().next().unwrap_or('?');
        let second = words[1].chars().next().unwrap_or(' ');
        if second != ' ' {
            format!("{first}{second}").to_uppercase()
        } else {
            first.to_uppercase().to_string()
        }
    } else {
        name.chars()
            .next()
            .unwrap_or('?')
            .to_uppercase()
            .to_string()
    }
}

pub fn get_server_grad_class(name: &str) -> &'static str {
    let sum: usize = name.chars().map(|c| c as usize).sum();
    match (name.len() + sum) % 6 {
        0 => "overview-grad-0",
        1 => "overview-grad-1",
        2 => "overview-grad-2",
        3 => "overview-grad-3",
        4 => "overview-grad-4",
        _ => "overview-grad-5",
    }
}

#[component(inline_props)]
pub fn ServerCard<F>(server: ServerSmallPart, is_active: bool, on_click: F) -> View
where
    F: Fn() + Copy + 'static,
{
    let s_id = server.id;
    let s_name = server.name;
    let s_alt = s_name.clone();
    let s_icon = server.icon_url;
    let initials = get_server_initials(&s_name);
    let has_icon = !s_icon.is_empty();

    let icon_box_class = if has_icon {
        "server-overview-icon-box has-icon".to_string()
    } else {
        let grad_cls = get_server_grad_class(&s_name);
        format!("server-overview-icon-box {grad_cls}")
    };

    let card_class = format!("server-overview-item{}", if is_active { " active" } else { "" });

    view! {
        div(
            class=card_class,
            on:click=move |_| on_click(),
            data-server-id=s_id.to_string(),
        ) {
            div(class=icon_box_class) {
                (if has_icon {
                    view! {
                        img(src=s_icon, alt=s_alt, class="server-overview-icon-img")
                    }
                } else {
                    view! {
                        span(class="server-overview-initials") { (initials) }
                    }
                })
            }
            span(class="server-overview-name") { (s_name) }
        }
    }
}

#[component(inline_props)]
pub fn AddServerTile<F>(on_click: F) -> View
where
    F: Fn(web_sys::MouseEvent) + Copy + 'static,
{
    view! {
        div(
            class="server-overview-item add-tile",
            on:click=on_click,
        ) {
            div(class="server-overview-icon-box add-box") {
                PlusIcon()
            }
            span(class="server-overview-name") { "Добавить" }
        }
    }
}
