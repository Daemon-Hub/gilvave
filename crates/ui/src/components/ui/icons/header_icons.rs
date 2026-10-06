use sycamore::prelude::*;

#[component(inline_props)]
pub fn MinimizeIcon(#[prop(attributes(html, button))] attributes: Attributes) -> View {
    view! {
        button(class="window-btn", ..attributes) {
            svg(viewBox="0 0 24 24") {
                line(x1="5", y1="12", x2="19", y2="12")
            }
        }
    }
}

#[component(inline_props)]
pub fn MaximizeIcon(#[prop(attributes(html, button))] attributes: Attributes) -> View {
    view! {
        button(class="window-btn", ..attributes) {
            svg(viewBox="0 0 24 24") {
                rect(x="5", y="5", width="14", height="14")
            }
        }
    }
}

#[component(inline_props)]
pub fn CloseIcon(#[prop(attributes(html, button))] attributes: Attributes) -> View {
    view! {
        button(class="window-btn close", ..attributes) {
            svg(viewBox="0 0 24 24") {
                line(x1="6", y1="6", x2="18", y2="18")
                line(x1="18", y1="6", x2="6", y2="18")
            }
        }
    }
}
