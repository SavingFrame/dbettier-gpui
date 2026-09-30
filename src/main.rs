#![cfg_attr(target_family = "wasm", no_main)]

extern crate gpui_kit as gpui;

mod app;
mod assets;
mod database;
mod runtime;
mod workspace;

#[cfg(not(target_family = "wasm"))]
fn main() {
    app::run();
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    gpui_kit::platform::web_init();
    app::run();
}
