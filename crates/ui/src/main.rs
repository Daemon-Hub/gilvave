mod app;
pub mod cache;
mod components;
pub mod gateway;
pub mod http;
pub mod security;
pub mod utils;

use app::App;

fn main() {
    console_error_panic_hook::set_once();
    components::ui::markdown::setup_custom_video_players();
    sycamore::render(App);
}
