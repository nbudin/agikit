use std::{
    io::{Read, Seek, Write},
    str::FromStr,
};

use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::js_sys::{JsString, Uint8Array};

use crate::buffer::Buffer;

pub const AGI_ENCRYPTION_KEY: &str = "Avis Durgan";

#[wasm_bindgen(js_name = getXorEncryptionKey)]
pub fn get_xor_encryption_key() -> Buffer {
    Buffer::from_str(
        &JsString::from_str(AGI_ENCRYPTION_KEY).unwrap(),
        &JsString::from_str("ascii").unwrap(),
    )
}

pub trait ReadXor<T: Read + Seek> {
    fn get_xor_encryption_key(&self) -> &[u8];
    fn get_data(&mut self) -> &mut T;

    fn read_xor(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        let data = self.get_data();
        let offset = data.stream_position()? as usize;
        let size = data.read(buf)?;

        let key = self.get_xor_encryption_key();
        for i in 0..size {
            let key_byte = key[(offset + i) % key.len()];
            let output_byte = buf[i] ^ key_byte;
            buf[i] = output_byte;
        }

        Ok(size)
    }
}

pub trait WriteXor<T: Write + Seek> {
    fn get_xor_encryption_key(&self) -> &[u8];
    fn get_data(&mut self) -> &mut T;

    fn write_xor(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        let offset = self.get_data().stream_position()? as usize;
        let key = self.get_xor_encryption_key();

        let xor_buf = buf
            .iter()
            .enumerate()
            .map(|(i, &byte)| {
                let key_byte = key[(offset + i) % key.len()];
                byte ^ key_byte
            })
            .collect::<Vec<u8>>();

        self.get_data().write_all(&xor_buf)?;
        Ok(xor_buf.len())
    }
}

#[derive(Debug)]
pub struct XorCursor<'a, T> {
    data: &'a mut T,
    key: &'a [u8],
}

impl<'a, T> XorCursor<'a, T> {
    pub fn new(data: &'a mut T, key: &'a [u8]) -> Self {
        Self { data, key }
    }
}

impl<'a, T: Read + Seek> ReadXor<T> for XorCursor<'a, T> {
    fn get_xor_encryption_key(&self) -> &[u8] {
        self.key
    }

    fn get_data(&mut self) -> &mut T {
        self.data
    }
}

impl<'a, T: Read + Seek> Read for XorCursor<'a, T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.read_xor(buf)
    }
}

impl<'a, T: Seek> Seek for XorCursor<'a, T> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> Result<u64, std::io::Error> {
        Seek::seek(self.data, pos)
    }
}

impl<'a, T: Read + Write + Seek> Write for XorCursor<'a, T> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.write_xor(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        T::flush(self.data)
    }
}

impl<'a, T: Read + Write + Seek> WriteXor<T> for XorCursor<'a, T> {
    fn get_xor_encryption_key(&self) -> &[u8] {
        self.key
    }

    fn get_data(&mut self) -> &mut T {
        self.data
    }
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
    let input_array = Uint8Array::new(&input);
    let key_array = Uint8Array::new(&encryption_key);

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
