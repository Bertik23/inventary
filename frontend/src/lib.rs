mod app;
mod components;
mod api;
mod barcode;
mod router;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();
    
    yew::Renderer::<app::App>::new().render();
}
