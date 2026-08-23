#![cfg_attr(target_family = "wasm", no_main)]

mod app;
mod theme;
mod workspace;

#[cfg(not(target_family = "wasm"))]
fn main() {
    app::run();
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    gpui_platform::web_init();
    app::run();
}
