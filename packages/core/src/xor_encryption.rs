use std::str::FromStr;

use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
use web_sys::{
    console,
    js_sys::{JsString, Uint8Array},
};

use crate::wasm_utils::Buffer;

const AGI_ENCRYPTION_KEY: &str = "Avis Durgan";

#[wasm_bindgen(js_name = getXorEncryptionKey)]
pub fn get_xor_encryption_key() -> Buffer {
    Buffer::from_str(
        &JsString::from_str(AGI_ENCRYPTION_KEY).unwrap(),
        &JsString::from_str("ascii").unwrap(),
    )
}

pub struct XorEncryptionIterator<'a> {
    input: &'a mut dyn Iterator<Item = u8>,
    key: &'a [u8],
    key_index: usize,
}

impl<'a> XorEncryptionIterator<'a> {
    pub fn new(input: &'a mut dyn Iterator<Item = u8>, key: &'a [u8]) -> Self {
        Self {
            input,
            key,
            key_index: 0,
        }
    }
}

impl<'a> Iterator for XorEncryptionIterator<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let next_byte = self.input.next()?;
        let key_byte = self.key[self.key_index];
        self.key_index = (self.key_index + 1) % self.key.len();
        Some(next_byte ^ key_byte)
    }
}

#[wasm_bindgen(js_name = xorBuffer)]
pub fn xor_buffer(
    input: Buffer,
    #[wasm_bindgen(js_name = encryptionKey)] encryption_key: Buffer,
) -> Buffer {
    console::log_1(&JsValue::from_str("xor_buffer called"));
    let input_array = Uint8Array::new(&input);
    console::log_2(
        &JsValue::from_str("Input array length:"),
        &JsValue::from(input_array.length()),
    );
    let key_array = Uint8Array::new(&encryption_key);
    console::log_2(
        &JsValue::from_str("Key array length:"),
        &JsValue::from(key_array.length()),
    );

    let input_bytes = input_array.to_vec();
    let key_bytes = key_array.to_vec();
    let mut input_iter = input_bytes.into_iter();

    let xor_iter = XorEncryptionIterator::new(&mut input_iter, key_bytes.as_slice());

    let xored_bytes: Vec<u8> = xor_iter.collect();
    let array_buffer = Uint8Array::new_with_length(xored_bytes.len() as u32);
    array_buffer.copy_from(&xored_bytes);
    Buffer::from_array_buffer(&array_buffer.buffer())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xor_encryption() {
        let input = "Hello world!";

        let mut input_iter = input.as_bytes().iter().copied();
        let xored = XorEncryptionIterator::new(&mut input_iter, AGI_ENCRYPTION_KEY.as_bytes())
            .collect::<Vec<u8>>();

        let expected_xored = vec![
            0x09, 0x13, 0x05, 0x1f, 0x4f, 0x64, 0x02, 0x1d, 0x15, 0x0d, 0x0a, 0x60,
        ];
        assert_eq!(xored, expected_xored);
    }
}
