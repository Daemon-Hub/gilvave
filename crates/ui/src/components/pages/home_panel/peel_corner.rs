//! Interactive 3D paper corner peel and razor slit cut line in the server sidebar.

use sycamore::prelude::*;
use crate::components::common::UiModalContext;

#[component]
pub(crate) fn SidebarCornerPeel() -> View {
    let ui_modal_context = use_context::<UiModalContext>();
    let is_peel_hovered = create_signal(false);

    let peel_cut_line_d = move || {
        if is_peel_hovered.get() {
            "M 14.80 3.00 L 45.00 3.00 L 45.00 4.60 L 14.80 4.60 A 0.80 0.80 0 0 1 14.80 3.00 Z"
        } else {
            "M 14.80 3.00 L 58.40 3.00 L 58.40 4.60 L 14.80 4.60 A 0.80 0.80 0 0 1 14.80 3.00 Z"
        }
    };
    let peel_void_d = move || {
        if is_peel_hovered.get() {
            "M 45.00 3.65 C 59.58 3.35 72.00 13.35 72.00 25.50 L 72.00 3.00 L 45.00 3.00 Z"
        } else {
            "M 58.40 3.65 C 65.74 3.35 72.00 8.15 72.00 14.20 L 72.00 3.00 L 58.40 3.00 Z"
        }
    };
    let peel_shadow_d = move || {
        if is_peel_hovered.get() {
            "M 45.00 3.65 L 72.00 25.50 C 64.90 23.93 54.37 29.97 43.92 27.50 C 40.87 18.73 47.08 9.72 45.00 3.65 Z"
        } else {
            "M 58.40 3.65 L 72.00 14.20 C 68.42 13.42 63.12 16.75 57.86 15.52 C 56.32 11.15 59.45 6.67 58.40 3.65 Z"
        }
    };
    let peel_flap_d = move || {
        if is_peel_hovered.get() {
            "M 45.00 3.65 C 59.58 3.35 72.00 13.35 72.00 25.50 C 66.60 22.35 58.50 26.15 50.94 23.57 Q 48.78 23.00 48.21 20.84 C 45.00 14.90 48.78 8.15 45.00 3.65 Z"
        } else {
            "M 58.40 3.65 C 65.74 3.35 72.00 8.15 72.00 14.20 C 69.28 12.63 65.20 14.85 61.44 13.58 Q 60.30 13.28 60.00 12.14 C 58.40 9.25 60.30 5.89 58.40 3.65 Z"
        }
    };
    let peel_bevel_d = move || {
        if is_peel_hovered.get() {
            "M 45.00 3.65 C 48.78 8.15 45.00 14.90 48.21 20.84 Q 48.78 23.00 50.94 23.57 C 58.50 26.15 66.60 22.35 72.00 25.50"
        } else {
            "M 58.40 3.65 C 60.30 5.89 58.40 9.25 60.00 12.14 Q 60.30 13.28 61.44 13.58 C 65.20 14.85 69.28 12.63 72.00 14.20"
        }
    };
    let peel_crease_d = move || {
        if is_peel_hovered.get() {
            "M 45.00 3.65 C 59.58 3.35 72.00 13.35 72.00 25.50"
        } else {
            "M 58.40 3.65 C 65.74 3.35 72.00 8.15 72.00 14.20"
        }
    };
    let peel_hit_d = move || {
        if is_peel_hovered.get() {
            "M 44.00 0.00 L 72.00 0.00 L 72.00 27.00 L 44.00 27.00 Z"
        } else {
            "M 54.00 0.00 L 72.00 0.00 L 72.00 16.00 L 54.00 16.00 Z"
        }
    };

    view! {
        div(class="sidebar-corner-peel") {
            svg(
                class="corner-peel-svg",
                viewBox="0 0 72 32",
                fill="none",
                xmlns="http://www.w3.org/2000/svg",
            ) {
                defs {
                    linearGradient(id="cornerUnderlayerGrad", x1="14.0", y1="3.0", x2="72.0", y2="3.0", gradientUnits="userSpaceOnUse") {
                        stop(offset="0%", stop-color="var(--color-server-separator-stop-1, #22d3ee)")
                        stop(offset="50%", stop-color="var(--color-server-separator-stop-2, #818cf8)")
                        stop(offset="100%", stop-color="var(--color-server-separator-stop-3, #f59e0b)")
                    }
                    linearGradient(id="gilvaveFlapGrad", x1="14.0", y1="3.0", x2="72.0", y2="3.0", gradientUnits="userSpaceOnUse") {
                        stop(offset="0%", stop-color="var(--color-server-flap-stop-1, #0e7490)")
                        stop(offset="50%", stop-color="var(--color-server-flap-stop-2, #3730a3)")
                        stop(offset="100%", stop-color="var(--color-server-flap-stop-3, #78350f)")
                    }
                    filter(id="cornerShadowBlur", x="-40%", y="-40%", width="180%", height="180%") {
                        feGaussianBlur(stdDeviation="1.0")
                    }
                }

                path(
                    class="corner-cut-line",
                    d=peel_cut_line_d,
                    fill="url(#cornerUnderlayerGrad)",
                )

                g(
                    class="peel-corner-hit",
                    on:mouseenter=move |_| is_peel_hovered.set(true),
                    on:mouseleave=move |_| is_peel_hovered.set(false),
                    on:click=move |_| ui_modal_context.is_server_overview_open.set(true),
                ) {
                    title { "Все серверы" }
                    path(
                        class="peel-hit-pad",
                        d=peel_hit_d,
                        fill="transparent",
                    )
                    path(
                        class="corner-void",
                        d=peel_void_d,
                        fill="url(#cornerUnderlayerGrad)",
                    )
                    path(
                        class="corner-shadow",
                        d=peel_shadow_d,
                        fill="rgba(0, 0, 0, 0.75)",
                        filter="url(#cornerShadowBlur)",
                    )
                    path(
                        class="corner-flap",
                        d=peel_flap_d,
                        fill="url(#gilvaveFlapGrad)",
                        stroke="var(--color-server-flap-stroke, #3b1124)",
                        stroke-width="0.3",
                    )
                    path(
                        class="corner-bevel",
                        d=peel_bevel_d,
                        stroke="rgba(255, 255, 255, 0.65)",
                        stroke-width="0.5",
                        fill="none",
                        stroke-linecap="round",
                    )
                    path(
                        class="corner-crease",
                        d=peel_crease_d,
                        stroke="rgba(129, 140, 248, 0.4)",
                        stroke-width="0.4",
                        fill="none",
                        stroke-linecap="round",
                    )
                }
            }
        }
    }
}
