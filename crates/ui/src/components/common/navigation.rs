use strum::{Display, EnumIter, EnumString};
use sycamore::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, EnumIter, Display, EnumString)]
pub enum ActiveScreen {
    Loading,
    Login,
    Register,
    Home,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ScreenWrapper(pub Signal<ActiveScreen>);

impl ScreenWrapper {
    pub fn get(&self) -> ActiveScreen {
        self.0.get()
    }

    pub fn set(&self, value: ActiveScreen) {
        self.0.set(value);
    }

    pub fn is_loading(&self) -> bool {
        self.get() == ActiveScreen::Loading
    }

    pub fn is_login(&self) -> bool {
        self.get() == ActiveScreen::Login
    }

    pub fn is_register(&self) -> bool {
        self.get() == ActiveScreen::Register
    }

    pub fn is_auth(&self) -> bool {
        self.is_login() || self.is_register()
    }

    pub fn is_home(&self) -> bool {
        self.get() == ActiveScreen::Home
    }
}
