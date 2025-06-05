use std::{
    io::{self, Read, Seek},
    mem::MaybeUninit,
};

use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(js_name = encodeUInt16LE)]
pub fn encode_uint16le(value: u16) -> Vec<u8> {
    Vec::from([(value & 0xff) as u8, ((value & 0xff00) >> 8) as u8])
}

#[wasm_bindgen(js_name = encodeUInt16BE)]
pub fn encode_uint16be(value: u16) -> Vec<u8> {
    Vec::from([((value & 0xff00) >> 8) as u8, (value & 0xff) as u8])
}

pub trait ReadHeterogeneousData: Read + Seek + Clone {
    fn read_u8(&mut self) -> Result<u8, io::Error>;
    fn read_u16_le(&mut self) -> Result<u16, io::Error>;
    fn read_u16_be(&mut self) -> Result<u16, io::Error>;
    fn read_null_terminated_string(&mut self) -> Result<String, io::Error>;
}

impl<T: Read + Seek + Clone> ReadHeterogeneousData for T {
    fn read_u8(&mut self) -> Result<u8, io::Error> {
        let mut buffer = MaybeUninit::<[u8; 1]>::uninit();
        let bytes = self.read(unsafe { &mut *buffer.as_mut_ptr() })?;
        if bytes != 1 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "End of data"));
        }
        let bytes = unsafe { buffer.assume_init() };
        Ok(bytes[0])
    }

    fn read_u16_le(&mut self) -> Result<u16, io::Error> {
        let mut buffer = MaybeUninit::<[u8; 2]>::uninit();
        let bytes = self.read(unsafe { &mut *buffer.as_mut_ptr() })?;
        if bytes != 2 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "End of data"));
        }
        let bytes = unsafe { buffer.assume_init() };
        Ok(bytes[0] as u16 | ((bytes[1] as u16) << 8))
    }

    fn read_u16_be(&mut self) -> Result<u16, io::Error> {
        let mut buffer = MaybeUninit::<[u8; 2]>::uninit();
        let bytes = self.read(unsafe { &mut *buffer.as_mut_ptr() })?;
        if bytes != 2 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "End of data"));
        }
        let bytes = unsafe { buffer.assume_init() };
        Ok(bytes[1] as u16 | ((bytes[0] as u16) << 8))
    }

    fn read_null_terminated_string(&mut self) -> Result<String, io::Error> {
        let mut string = String::new();
        loop {
            let byte = self.read_u8()?;
            if byte == 0 {
                break;
            }
            string.push(byte as char);
        }
        Ok(string)
    }
}
