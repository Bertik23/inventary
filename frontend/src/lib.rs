pub mod api;
pub mod app;
pub mod barcode;
pub mod components;
pub mod i18n;
pub mod router;

use wasm_bindgen::prelude::*;

pub fn format_quantity(q: f64) -> String {
    let s = format!("{:.2}", q);
    let s = s.strip_suffix(".00").unwrap_or(&s);
    let s = s.strip_suffix("0").filter(|v| v.contains('.')).unwrap_or(s);
    s.to_string()
}

#[wasm_bindgen(start)]
pub fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();

    yew::Renderer::<app::App>::new().render();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_quantity() {
        assert_eq!(format_quantity(1.0), "1");
        assert_eq!(format_quantity(1.5), "1.5");
        assert_eq!(format_quantity(1.50), "1.5");
        assert_eq!(format_quantity(1.51), "1.51");
        assert_eq!(format_quantity(0.0), "0");
    }
}
