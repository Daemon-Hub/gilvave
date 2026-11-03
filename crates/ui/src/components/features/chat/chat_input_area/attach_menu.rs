//! Attachment popover menu for uploading media and files.

use sycamore::prelude::*;
use web_sys::Event;

#[component(inline_props)]
pub(crate) fn AttachMenu<F>(is_open: Signal<bool>, on_file_selected: F) -> View
where
    F: Fn(Event) + Copy + 'static,
{
    view! {
        // Hidden file inputs for media and files
        input(
            id="chat-attach-media-input",
            r#type="file",
            accept="image/*,video/*",
            multiple=true,
            class="chat-hidden-file-input",
            on:change=on_file_selected,
        )
        input(
            id="chat-attach-file-input",
            r#type="file",
            multiple=true,
            class="chat-hidden-file-input",
            on:change=on_file_selected,
        )

        // Attachment popup menu (bottom-left above paperclip button)
        (move || if is_open.get() {
            view! {
                div(
                    class="chat-attach-menu",
                    on:click=move |e: web_sys::MouseEvent| e.stop_propagation(),
                ) {
                    label(
                        r#for="chat-attach-media-input",
                        class="chat-attach-menu-item",
                    ) {
                        span(class="attach-item-icon media") {
                            svg(viewBox="0 0 24 24") {
                                rect(x="3", y="3", width="18", height="18", rx="3", ry="3")
                                circle(cx="8.5", cy="8.5", r="1.5")
                                polyline(points="21 15 16 10 5 21")
                            }
                        }
                        span(class="attach-item-label") { "Фото или видео" }
                    }
                    label(
                        r#for="chat-attach-file-input",
                        class="chat-attach-menu-item",
                    ) {
                        span(class="attach-item-icon file") {
                            svg(viewBox="0 0 24 24") {
                                path(d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z")
                                polyline(points="14 2 14 8 20 8")
                            }
                        }
                        span(class="attach-item-label") { "Файл" }
                    }
                }
            }
        } else {
            view! {}
        })
    }
}
