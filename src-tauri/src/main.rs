#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    app_lib::run();
}
