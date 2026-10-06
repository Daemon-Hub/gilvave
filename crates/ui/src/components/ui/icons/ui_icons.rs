use sycamore::prelude::*;

#[component]
pub fn PlusIcon() -> View {
    view! {
        svg(
            class="ui-icon ui-icon-plus",
            viewBox="0 0 24 24",
            fill="none",
            stroke="currentColor",
            stroke-width="2.2",
            stroke-linecap="round",
            stroke-linejoin="round",
        ) {
            line(x1="12", y1="5", x2="12", y2="19")
            line(x1="5", y1="12", x2="19", y2="12")
        }
    }
}

#[component]
pub fn HomeIcon() -> View {
    view! {
        svg(
            class="ui-icon ui-icon-home",
            viewBox="0 0 24 24",
            fill="none",
            stroke="currentColor",
            stroke-width="2.2",
            stroke-linecap="round",
            stroke-linejoin="round",
        ) {
            path(d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z")
            polyline(points="9 22 9 12 15 12 15 22")
        }
    }
}

#[component]
pub fn GearIcon() -> View {
    view! {
        svg(
            class="ui-icon ui-icon-gear",
            viewBox="0 0 24 24",
            fill="none",
            stroke="currentColor",
            stroke-width="2.2",
            stroke-linecap="round",
            stroke-linejoin="round",
        ) {
            circle(cx="12", cy="12", r="3")
            path(d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z")
        }
    }
}

#[component]
pub fn GlobeIcon() -> View {
    view! {
        svg(
            class="ui-icon ui-icon-globe",
            viewBox="0 0 24 24",
            fill="none",
            stroke="currentColor",
            stroke-width="2.2",
            stroke-linecap="round",
            stroke-linejoin="round",
        ) {
            circle(cx="12", cy="12", r="10")
            line(x1="2", y1="12", x2="22", y2="12")
            path(d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z")
        }
    }
}

#[component]
pub fn LockIcon() -> View {
    view! {
        svg(
            class="ui-icon ui-icon-lock",
            viewBox="0 0 24 24",
            fill="none",
            stroke="currentColor",
            stroke-width="2.2",
            stroke-linecap="round",
            stroke-linejoin="round",
        ) {
            rect(x="3", y="11", width="18", height="11", rx="2", ry="2")
            path(d="M7 11V7a5 5 0 0 1 10 0v4")
        }
    }
}

#[component]
pub fn ChatBubbleIcon() -> View {
    view! {
        svg(
            class="ui-icon ui-icon-chat",
            viewBox="0 0 24 24",
            fill="none",
            stroke="currentColor",
            stroke-width="2.2",
            stroke-linecap="round",
            stroke-linejoin="round",
        ) {
            path(d="M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z")
        }
    }
}

#[component]
pub fn ArrowRightIcon() -> View {
    view! {
        svg(
            class="ui-icon ui-icon-arrow-right",
            viewBox="0 0 24 24",
            fill="none",
            stroke="currentColor",
            stroke-width="2.2",
            stroke-linecap="round",
            stroke-linejoin="round",
        ) {
            line(x1="5", y1="12", x2="19", y2="12")
            polyline(points="12 5 19 12 12 19")
        }
    }
}

#[component]
pub fn ArrowLeftIcon() -> View {
    view! {
        svg(
            class="ui-icon ui-icon-arrow-left",
            viewBox="0 0 24 24",
            fill="none",
            stroke="currentColor",
            stroke-width="2.2",
            stroke-linecap="round",
            stroke-linejoin="round",
        ) {
            line(x1="19", y1="12", x2="5", y2="12")
            polyline(points="12 19 5 12 12 5")
        }
    }
}

#[component]
pub fn CloseSmallIcon() -> View {
    view! {
        svg(
            class="ui-icon ui-icon-close",
            viewBox="0 0 24 24",
            fill="none",
            stroke="currentColor",
            stroke-width="2.2",
            stroke-linecap="round",
            stroke-linejoin="round",
        ) {
            line(x1="18", y1="6", x2="6", y2="18")
            line(x1="6", y1="6", x2="18", y2="18")
        }
    }
}

#[component]
pub fn UserIcon() -> View {
    view! {
        svg(
            class="ui-icon ui-icon-user",
            viewBox="0 0 24 24",
            fill="none",
            stroke="currentColor",
            stroke-width="2.2",
            stroke-linecap="round",
            stroke-linejoin="round",
        ) {
            path(d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2")
            circle(cx="12", cy="7", r="4")
        }
    }
}

#[component]
pub fn PaletteIcon() -> View {
    view! {
        svg(
            class="ui-icon ui-icon-palette",
            viewBox="0 0 24 24",
            fill="none",
            stroke="currentColor",
            stroke-width="2.2",
            stroke-linecap="round",
            stroke-linejoin="round",
        ) {
            path(d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10c.9 0 1.6-.7 1.6-1.6 0-.4-.2-.8-.4-1.1-.3-.4-.4-.8-.4-1.3 0-.9.7-1.6 1.6-1.6h1.9c4.3 0 7.7-3.4 7.7-7.7C22 5.6 17.5 2 12 2z")
            circle(cx="7.5", cy="11.5", r="1.5", fill="currentColor")
            circle(cx="12", cy="7.5", r="1.5", fill="currentColor")
            circle(cx="16.5", cy="11.5", r="1.5", fill="currentColor")
        }
    }
}

#[component]
pub fn LogOutIcon() -> View {
    view! {
        svg(
            class="ui-icon ui-icon-logout",
            viewBox="0 0 24 24",
            fill="none",
            stroke="currentColor",
            stroke-width="2.2",
            stroke-linecap="round",
            stroke-linejoin="round",
        ) {
            path(d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4")
            polyline(points="16 17 21 12 16 7")
            line(x1="21", y1="12", x2="9", y2="12")
        }
    }
}

#[component]
pub fn DownloadIcon() -> View {
    view! {
        svg(
            class="ui-icon ui-icon-download",
            viewBox="0 0 24 24",
            fill="none",
            stroke="currentColor",
            stroke-width="2.2",
            stroke-linecap="round",
            stroke-linejoin="round",
        ) {
            path(d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4")
            polyline(points="7 10 12 15 17 10")
            line(x1="12", y1="15", x2="12", y2="3")
        }
    }
}

#[component]
pub fn UploadIcon() -> View {
    view! {
        svg(
            class="ui-icon ui-icon-upload",
            viewBox="0 0 24 24",
            fill="none",
            stroke="currentColor",
            stroke-width="2.2",
            stroke-linecap="round",
            stroke-linejoin="round",
        ) {
            path(d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4")
            polyline(points="17 8 12 3 7 8")
            line(x1="12", y1="3", x2="12", y2="15")
        }
    }
}

#[component]
pub fn CodeIcon() -> View {
    view! {
        svg(
            class="ui-icon ui-icon-code",
            viewBox="0 0 24 24",
            fill="none",
            stroke="currentColor",
            stroke-width="2.2",
            stroke-linecap="round",
            stroke-linejoin="round",
        ) {
            polyline(points="16 18 22 12 16 6")
            polyline(points="8 6 2 12 8 18")
        }
    }
}

#[component]
pub fn FileTextIcon() -> View {
    view! {
        svg(
            class="ui-icon ui-icon-file-text",
            viewBox="0 0 24 24",
            fill="none",
            stroke="currentColor",
            stroke-width="2.2",
            stroke-linecap="round",
            stroke-linejoin="round",
        ) {
            path(d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z")
            polyline(points="14 2 14 8 20 8")
            line(x1="16", y1="13", x2="8", y2="13")
            line(x1="16", y1="17", x2="8", y2="17")
        }
    }
}

#[component]
pub fn CheckIcon() -> View {
    view! {
        svg(
            class="ui-icon ui-icon-check",
            viewBox="0 0 24 24",
            fill="none",
            stroke="currentColor",
            stroke-width="2.5",
            stroke-linecap="round",
            stroke-linejoin="round",
        ) {
            polyline(points="20 6 9 17 4 12")
        }
    }
}

#[component]
pub fn EditIcon() -> View {
    view! {
        svg(
            class="ui-icon ui-icon-edit",
            viewBox="0 0 24 24",
            fill="none",
            stroke="currentColor",
            stroke-width="2.2",
            stroke-linecap="round",
            stroke-linejoin="round",
        ) {
            path(d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7")
            path(d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z")
        }
    }
}

#[component]
pub fn TrashIcon() -> View {
    view! {
        svg(
            class="ui-icon ui-icon-trash",
            viewBox="0 0 24 24",
            fill="none",
            stroke="currentColor",
            stroke-width="2.2",
            stroke-linecap="round",
            stroke-linejoin="round",
        ) {
            polyline(points="3 6 5 6 21 6")
            path(d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2")
            line(x1="10", y1="11", x2="10", y2="17")
            line(x1="14", y1="11", x2="14", y2="17")
        }
    }
}

#[component]
pub fn CatalogIcon() -> View {
    view! {
        svg(
            class="ui-icon ui-icon-catalog",
            viewBox="0 0 24 24",
            fill="none",
            stroke="currentColor",
            stroke-width="2.2",
            stroke-linecap="round",
            stroke-linejoin="round",
        ) {
            path(d="M6 2L3 6v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V6l-3-4z")
            line(x1="3", y1="6", x2="21", y2="6")
            path(d="M16 10a4 4 0 0 1-8 0")
        }
    }
}

#[component]
pub fn MicIcon() -> View {
    view! {
        svg(
            class="ui-icon ui-icon-mic",
            viewBox="0 0 24 24",
            fill="none",
            stroke="currentColor",
            stroke-width="2.2",
            stroke-linecap="round",
            stroke-linejoin="round",
        ) {
            path(d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z")
            path(d="M19 10v2a7 7 0 0 1-14 0v-2")
            line(x1="12", y1="19", x2="12", y2="23")
            line(x1="8", y1="23", x2="16", y2="23")
        }
    }
}

#[component]
pub fn MicOffIcon() -> View {
    view! {
        svg(
            class="ui-icon ui-icon-mic-off",
            viewBox="0 0 24 24",
            fill="none",
            stroke="currentColor",
            stroke-width="2.2",
            stroke-linecap="round",
            stroke-linejoin="round",
        ) {
            line(x1="1", y1="1", x2="23", y2="23")
            path(d="M9 9v3a3 3 0 0 0 5.12 2.12M15 9.34V4a3 3 0 0 0-5.94-.6")
            path(d="M17 16.95A7 7 0 0 1 5 12v-2m14 0v2a7 7 0 0 1-.11 1.23")
            line(x1="12", y1="19", x2="12", y2="23")
            line(x1="8", y1="23", x2="16", y2="23")
        }
    }
}

#[component]
pub fn HeadphonesIcon() -> View {
    view! {
        svg(
            class="ui-icon ui-icon-headphones",
            viewBox="0 0 24 24",
            fill="none",
            stroke="currentColor",
            stroke-width="2.2",
            stroke-linecap="round",
            stroke-linejoin="round",
        ) {
            path(d="M3 18v-6a9 9 0 0 1 18 0v6")
            path(d="M21 19a2 2 0 0 1-2 2h-1a2 2 0 0 1-2-2v-3a2 2 0 0 1 2-2h3zM3 19a2 2 0 0 0 2 2h1a2 2 0 0 0 2-2v-3a2 2 0 0 0-2-2H3z")
        }
    }
}

#[component]
pub fn HeadphonesOffIcon() -> View {
    view! {
        svg(
            class="ui-icon ui-icon-headphones-off",
            viewBox="0 0 24 24",
            fill="none",
            stroke="currentColor",
            stroke-width="2.2",
            stroke-linecap="round",
            stroke-linejoin="round",
        ) {
            path(d="M3 18v-6a9 9 0 0 1 18 0v6")
            path(d="M21 19a2 2 0 0 1-2 2h-1a2 2 0 0 1-2-2v-3a2 2 0 0 1 2-2h3zM3 19a2 2 0 0 0 2 2h1a2 2 0 0 0 2-2v-3a2 2 0 0 0-2-2H3z")
            line(x1="2", y1="2", x2="22", y2="22")
        }
    }
}

#[component]
pub fn UsersIcon() -> View {
    view! {
        svg(
            class="ui-icon ui-icon-users",
            viewBox="0 0 24 24",
            fill="none",
            stroke="currentColor",
            stroke-width="2.2",
            stroke-linecap="round",
            stroke-linejoin="round",
        ) {
            path(d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2")
            circle(cx="9", cy="7", r="4")
            path(d="M23 21v-2a4 4 0 0 0-3-3.87")
            path(d="M16 3.13a4 4 0 0 1 0 7.75")
        }
    }
}

#[component]
pub fn ZapIcon() -> View {
    view! {
        svg(
            class="ui-icon ui-icon-zap",
            viewBox="0 0 24 24",
            fill="none",
            stroke="currentColor",
            stroke-width="2.2",
            stroke-linecap="round",
            stroke-linejoin="round",
        ) {
            polygon(points="13 2 3 14 12 14 11 22 21 10 12 10 13 2")
        }
    }
}

#[component]
pub fn Volume2Icon() -> View {
    view! {
        svg(
            class="ui-icon ui-icon-volume-2",
            viewBox="0 0 24 24",
            fill="none",
            stroke="currentColor",
            stroke-width="2.2",
            stroke-linecap="round",
            stroke-linejoin="round",
        ) {
            polygon(points="11 5 6 9 2 9 2 15 6 15 11 19 11 5")
            path(d="M19.07 4.93a10 10 0 0 1 0 14.14M15.54 8.46a5 5 0 0 1 0 7.07")
        }
    }
}

#[component]
pub fn HashIcon() -> View {
    view! {
        svg(
            class="ui-icon ui-icon-hash",
            viewBox="0 0 24 24",
            fill="none",
            stroke="currentColor",
            stroke-width="2.2",
            stroke-linecap="round",
            stroke-linejoin="round",
        ) {
            line(x1="4", y1="9", x2="20", y2="9")
            line(x1="4", y1="15", x2="20", y2="15")
            line(x1="10", y1="3", x2="8", y2="21")
            line(x1="16", y1="3", x2="14", y2="21")
        }
    }
}

#[component]
pub fn BookOpenIcon() -> View {
    view! {
        svg(
            class="ui-icon ui-icon-book-open",
            viewBox="0 0 24 24",
            fill="none",
            stroke="currentColor",
            stroke-width="2.2",
            stroke-linecap="round",
            stroke-linejoin="round",
        ) {
            path(d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z")
            path(d="M22 3h-6a4 4 0 0 1-4 4v14a3 3 0 0 0 3-3h7z")
        }
    }
}

#[component]
pub fn PhoneIcon() -> View {
    view! {
        svg(
            class="ui-icon ui-icon-phone",
            viewBox="0 0 24 24",
            fill="none",
            stroke="currentColor",
            stroke-width="2.2",
            stroke-linecap="round",
            stroke-linejoin="round",
        ) {
            path(d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72 12.84 12.84 0 0 0 .7 2.81 2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45 12.84 12.84 0 0 0 2.81.7A2 2 0 0 1 22 16.92z")
        }
    }
}

#[component]
pub fn PhoneOffIcon() -> View {
    view! {
        svg(
            class="ui-icon ui-icon-phone-off",
            viewBox="0 0 24 24",
            fill="none",
            stroke="currentColor",
            stroke-width="2.2",
            stroke-linecap="round",
            stroke-linejoin="round",
        ) {
            path(d="M10.68 13.31a16 16 0 0 0 3.41 2.6l1.27-1.27a2 2 0 0 1 2.11-.45 12.84 12.84 0 0 0 2.81.7 2 2 0 0 1 1.72 2v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.42 19.42 0 0 1-3.33-2.67m-2.67-3.34a19.79 19.79 0 0 1-3.07-8.63A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72 12.84 12.84 0 0 0 .7 2.81 2 2 0 0 1-.45 2.11L8.09 9.91")
            line(x1="1", y1="1", x2="23", y2="23")
        }
    }
}

#[component]
pub fn ChevronDownIcon() -> View {
    view! {
        svg(
            class="ui-icon ui-icon-chevron-down",
            viewBox="0 0 24 24",
            fill="none",
            stroke="currentColor",
            stroke-width="2.2",
            stroke-linecap="round",
            stroke-linejoin="round",
        ) {
            polyline(points="6 9 12 15 18 9")
        }
    }
}

