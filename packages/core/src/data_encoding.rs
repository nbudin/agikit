use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(js_name = encodeUInt16LE)]
pub fn encode_uint16le(value: u16) -> Vec<u8> {
    Vec::from([(value & 0xff) as u8, ((value & 0xff00) >> 8) as u8])
}

#[wasm_bindgen(js_name = encodeUInt16BE)]
pub fn encode_uint16be(value: u16) -> Vec<u8> {
    Vec::from([((value & 0xff00) >> 8) as u8, (value & 0xff) as u8])
}
