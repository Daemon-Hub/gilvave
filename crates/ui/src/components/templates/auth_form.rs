use crate::components::common::classes;
use sycamore::prelude::*;

use crate::components::{
    common::ScreenWrapper,
    features::auth::form_tabs::*,
    pages::{login_panel::*, register_panel::*},
};

#[component]
pub fn AuthForm() -> View {
    let screen_wrapper = use_context::<ScreenWrapper>();
    let is_auth_screen = MaybeDyn::from(move || screen_wrapper.is_auth());

    view! {
        div(
            class=classes(vec![
                "container".into(),
                ("active", is_auth_screen.clone()).into(),
            ]),
        ) {
            FormTabs()
            LoginPanel(is_active=(move || screen_wrapper.is_login()).into())
            RegisterPanel(is_active=(move || screen_wrapper.is_register()).into())
        }
    }
}
