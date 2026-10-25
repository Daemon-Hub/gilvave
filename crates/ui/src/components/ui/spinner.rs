use sycamore::prelude::*;

#[component(inline_props)]
pub fn Spinner(#[prop(default)] label: Option<&'static str>) -> View {
    view! {
        div(class="spinner-overlay") {
            div(class="spinner") {
                div(class="spinner-ring")
                div(class="spinner-ring")
                div(class="spinner-ring")
                div(class="spinner-core")
            }
            (if let Some(text) = label {
                view! {
                    p(class="spinner-label") { (text) }
                }
            } else {
                view! {}
            })
        }
    }
}
