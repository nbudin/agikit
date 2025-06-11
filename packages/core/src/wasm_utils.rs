use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::js_sys::{ArrayBuffer, JsString};

// https://github.com/rustwasm/wasm-bindgen/issues/1993#issuecomment-583614609
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Buffer")]
    pub type Buffer;

    #[wasm_bindgen(js_name = "Buffer.from")]
    fn buffer_from_array_buffer(input: &ArrayBuffer) -> Buffer;

    #[wasm_bindgen(js_name = "Buffer.from")]
    fn buffer_from_str(input: &JsString, encoding: &JsString) -> Buffer;

    #[wasm_bindgen(method, getter)]
    pub fn buffer(this: &Buffer) -> ArrayBuffer;

    #[wasm_bindgen(method, getter, js_name = byteOffset)]
    pub fn byte_offset(this: &Buffer) -> u32;

    #[wasm_bindgen(method, getter)]
    pub fn length(this: &Buffer) -> u32;
}

impl Buffer {
    pub fn from_array_buffer(input: &ArrayBuffer) -> Buffer {
        buffer_from_array_buffer(input)
    }

    pub fn from_str(input: &JsString, encoding: &JsString) -> Buffer {
        buffer_from_str(input, encoding)
    }
}

impl From<ArrayBuffer> for Buffer {
    fn from(buffer: ArrayBuffer) -> Self {
        Buffer { obj: buffer.into() }
    }
}

impl Clone for Buffer {
    fn clone(&self) -> Self {
        Buffer {
            obj: self.obj.clone(),
        }
    }
}
