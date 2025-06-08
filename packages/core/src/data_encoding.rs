use std::{
    io::{self, Read, Seek, Write},
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

pub trait ReadHeterogeneousData: Read + Seek {
    fn read_u8(&mut self) -> Result<u8, io::Error>;
    fn read_u16_le(&mut self) -> Result<u16, io::Error>;
    fn read_u16_be(&mut self) -> Result<u16, io::Error>;
    fn read_null_terminated_string(&mut self) -> Result<String, io::Error>;
}

impl<T: Read + Seek> ReadHeterogeneousData for T {
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
        let mut bytes: Vec<u8> = Vec::new();
        loop {
            let byte = self.read_u8()?;
            if byte == 0 {
                break;
            }
            bytes.push(byte);
        }
        Ok(String::from_utf8(bytes)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?)
    }
}

pub trait WriteHeterogeneousData: Write + Seek {
    fn write_u8(&mut self, value: u8) -> Result<(), io::Error>;
    fn write_u16_le(&mut self, value: u16) -> Result<(), io::Error>;
    fn write_u16_be(&mut self, value: u16) -> Result<(), io::Error>;
    fn write_null_terminated_string(&mut self, value: &str) -> Result<(), io::Error>;
}

impl<T: Write + Seek> WriteHeterogeneousData for T {
    fn write_u8(&mut self, value: u8) -> Result<(), io::Error> {
        self.write(&[value])?;
        Ok(())
    }

    fn write_u16_le(&mut self, value: u16) -> Result<(), io::Error> {
        let bytes = encode_uint16le(value);
        self.write(&bytes)?;
        Ok(())
    }

    fn write_u16_be(&mut self, value: u16) -> Result<(), io::Error> {
        let bytes = encode_uint16be(value);
        self.write(&bytes)?;
        Ok(())
    }

    fn write_null_terminated_string(&mut self, value: &str) -> Result<(), io::Error> {
        let bytes = value.as_bytes();
        self.write(bytes)?;
        self.write(&[0])?;
        Ok(())
    }
}
