use sycamore::prelude::*;

use crate::components::common::{ActiveScreen, ScreenWrapper};
use crate::components::ui::tab_button::TabButton;

#[component]
pub fn FormTabs() -> View {
    let screen_wrapper = use_context::<ScreenWrapper>();
    let switch_form = move || {
        let screen_wrapper = use_context::<ScreenWrapper>();
        match screen_wrapper.get() {
            ActiveScreen::Login => screen_wrapper.set(ActiveScreen::Register),
            ActiveScreen::Register | ActiveScreen::RegisterConfirm => {
                screen_wrapper.set(ActiveScreen::Login)
            }
            _ => {}
        }
    };

    view! {
        div(class="form-header") {
            TabButton(
                on:click=move |_| if screen_wrapper.is_register() || screen_wrapper.is_register_confirm() { switch_form() },
                is_active=(move || screen_wrapper.is_login()).into(),
            ) { "Вход" }

            TabButton(
                on:click=move |_| if screen_wrapper.is_login() { switch_form() },
                is_active=(move || screen_wrapper.is_register() || screen_wrapper.is_register_confirm()).into(),
            ) { "Регистрация" }
        }
    }
}
