//! Internal SVG icons for context menu items.

use sycamore::prelude::*;

#[component(inline_props)]
pub(crate) fn MenuIcon(name: &'static str) -> View {
    match name {
        "reply" => view! {
            svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                polyline(points="9 17 4 12 9 7")
                path(d="M20 18v-2a4 4 0 0 0-4-4H4")
            }
        },
        "user_plus" => view! {
            svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                path(d="M16 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2")
                circle(cx="8.5", cy="7", r="4")
                line(x1="20", y1="8", x2="20", y2="14")
                line(x1="23", y1="11", x2="17", y2="11")
            }
        },
        "message" => view! {
            svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                path(d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z")
            }
        },
        "user" => view! {
            svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                path(d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2")
                circle(cx="12", cy="7", r="4")
            }
        },
        "forward" => view! {
            svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                polyline(points="15 17 20 12 15 7")
                path(d="M4 18v-2a4 4 0 0 1 4-4h12")
            }
        },
        "quote" => view! {
            svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                path(d="M3 21c3 0 7-1 7-8V5c0-1.25-.756-2.017-2-2H4c-1.25 0-2 .75-2 1.972V11c0 1.25.75 2 2 2 1 0 1 0 1 1v1c0 1-1 2-2 2s-1 .008-1 1.031V20c0 1 0 1 1 1z")
                path(d="M15 21c3 0 7-1 7-8V5c0-1.25-.757-2.017-2-2h-4c-1.25 0-2 .75-2 1.972V11c0 1.25.75 2 2 2 1 0 1 0 1 1v1c0 1-1 2-2 2s-1 .008-1 1.031V20c0 1 0 1 1 1z")
            }
        },
        "devtools" => view! {
            svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                polyline(points="16 18 22 12 16 6")
                polyline(points="8 6 2 12 8 18")
            }
        },
        "copy" => view! {
            svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                rect(x="9", y="9", width="13", height="13", rx="2", ry="2")
                path(d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1")
            }
        },
        "paste" => view! {
            svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                path(d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2")
                rect(x="8", y="2", width="8", height="4", rx="1", ry="1")
            }
        },
        "cut" => view! {
            svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                circle(cx="6", cy="6", r="3")
                circle(cx="6", cy="18", r="3")
                line(x1="20", y1="4", x2="8.12", y2="15.88")
                line(x1="14.47", y1="14.48", x2="20", y2="20")
                line(x1="8.12", y1="8.12", x2="12", y2="12")
            }
        },
        "select_all" => view! {
            svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                rect(x="3", y="3", width="18", height="18", rx="2", ry="2")
                polyline(points="9 11 12 14 22 4")
            }
        },
        "trash" => view! {
            svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                polyline(points="3 6 5 6 21 6")
                path(d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2")
            }
        },
        "edit" => view! {
            svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                path(d="M17 3a2.828 2.828 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z")
            }
        },
        "settings" => view! {
            svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                circle(cx="12", cy="12", r="3")
                path(d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z")
            }
        },
        "channel" => view! {
            svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                line(x1="4", y1="9", x2="20", y2="9")
                line(x1="4", y1="15", x2="20", y2="15")
                line(x1="10", y1="3", x2="8", y2="21")
                line(x1="16", y1="3", x2="14", y2="21")
            }
        },
        "refresh" => view! {
            svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                path(d="M23 4v6h-6")
                path(d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10")
            }
        },
        "theme" => view! {
            svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                path(d="M12 2.69l5.66 5.66a8 8 0 1 1-11.31 0z")
            }
        },
        "info" => view! {
            svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                circle(cx="12", cy="12", r="10")
                line(x1="12", y1="16", x2="12", y2="12")
                line(x1="12", y1="8", x2="12.01", y2="8")
            }
        },
        "download" => view! {
            svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                path(d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4")
                polyline(points="7 10 12 15 17 10")
                line(x1="12", y1="15", x2="12", y2="3")
            }
        },
        "upload" => view! {
            svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                path(d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4")
                polyline(points="17 8 12 3 7 8")
                line(x1="12", y1="3", x2="12", y2="15")
            }
        },
        _ => view! {
            svg(viewBox="0 0 24 24", fill="none", stroke="currentColor", stroke-width="2", stroke-linecap="round", stroke-linejoin="round") {
                circle(cx="12", cy="12", r="1")
                circle(cx="19", cy="12", r="1")
                circle(cx="5", cy="12", r="1")
            }
        },
    }
}

