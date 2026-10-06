use sycamore::prelude::*;

#[component]
pub fn Spinner() -> View {
    view! {
        div(class="spinner-overlay") {
            div(class="spinner") {
                div(class="spinner-ring")
                div(class="spinner-ring")
                div(class="spinner-ring")
                div(class="spinner-core")
            }
        }
    }
}
