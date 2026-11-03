//! Appearance tab coordinating workbench settings, custom theme toolbar, built-in themes, and custom themes.

pub mod builtin_cards;
pub mod custom_card;
pub mod theme_toolbar;
pub mod workbench_settings;

use sycamore::prelude::*;

use self::builtin_cards::BuiltinThemeCards;
use self::custom_card::CustomThemeCard;
use self::theme_toolbar::ThemeToolbar;
use self::workbench_settings::WorkbenchSettings;
use crate::components::common::UiModalContext;

#[component(inline_props)]
pub fn AppearanceTab(
    theme_status_msg: Signal<String>,
    theme_status_is_error: Signal<bool>,
) -> View {
    let modal_context = use_context::<UiModalContext>();
    let custom_themes_sig = modal_context.custom_themes;

    view! {
        div(class="tab-pane theme-settings-pane") {
            h3(class="tab-title") { "Внешний вид" }

            // Workbench settings (Theme dropdown, windowed mode, greeting, quick access)
            WorkbenchSettings()

            // Custom themes actions toolbar
            ThemeToolbar(
                theme_status_msg=theme_status_msg,
                theme_status_is_error=theme_status_is_error,
            )

            // Theme Cards Grid
            div(class="theme-cards-grid") {
                // Built-in theme cards: Advanced, Standard, Light
                BuiltinThemeCards()

                // Custom user & community themes
                (move || {
                    let list = custom_themes_sig.get_clone();
                    let cards: Vec<View> = list
                        .into_iter()
                        .map(|theme| {
                            view! {
                                CustomThemeCard(
                                    theme=theme,
                                    theme_status_msg=theme_status_msg,
                                    theme_status_is_error=theme_status_is_error,
                                )
                            }
                        })
                        .collect();
                    cards
                })
            }
        }
    }
}
