// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    set_linux_env();
    gilvave_lib::run()
}

#[cfg(target_os = "linux")]
fn set_linux_env() {
    unsafe {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
        std::env::set_var("GDK_BACKEND", "wayland");
    }
}

#[cfg(not(target_os = "linux"))]
fn set_linux_env() {}
