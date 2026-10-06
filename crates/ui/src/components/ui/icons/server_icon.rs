use sycamore::prelude::*;

use crate::components::common::classes;

#[component(inline_props)]
pub fn ServerIcon(
    server_name: Signal<String>,
    icon_url: Signal<String>,
    #[prop(default = false.into())] is_active: MaybeDyn<bool>,
    #[prop(attributes(html, div))] attributes: Attributes,
) -> View {
    let is_plus = server_name.with(|s| s == "+");
    let class = classes(vec![
        "server-icon".into(),
        ("active", is_active).into(),
        ("new", is_plus.into()).into(),
    ]);

    view! {
        div(class=class, ..attributes) {
            (if is_plus {
                view! {
                    svg(
                        class="server-icon-plus-svg",
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
            } else if icon_url.with(|s| s.is_empty()) {
                let first_char = server_name.with(|s| {
                    s.chars().next().unwrap_or('?').to_uppercase().to_string()
                });
                view! { span { (first_char) } }
            } else {
                let icon_str = icon_url.get_clone();
                view! { img(src=icon_str) }
            })
        }
    }
}
