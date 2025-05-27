pub mod color_palettes;
mod data_encoding;
mod wasm_utils;
mod xor_encryption;

use color_palettes::ega_palette;
use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen]
pub fn greet() {
    let palette = ega_palette();
    let greeting: String = format!("{:?}", palette.colors);
    console::log_1(&JsValue::from(&greeting));
}
