use sycamore::prelude::*;

#[component]
pub fn UserAvatar() -> View {
    view! {
        svg(
            class="avatar",
            xmlns="http://www.w3.org/2000/svg",
            viewBox="0 0 100 100",
        ) {
            // Темный фон
            circle(cx="50", cy="50", r="50", fill="#2C2C2E")
            // Силуэт (светло-серый)
            circle(cx="50", cy="36", r="16", fill="#636366")
            ellipse(cx="50", cy="80", rx="30", ry="16", fill="#636366")
        }
    }
}
