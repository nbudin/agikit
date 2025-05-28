use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(js_name = encodeUInt16LE)]
pub fn encode_uint16le(value: u16) -> Vec<u8> {
    Vec::from([(value & 0xff) as u8, ((value & 0xff00) >> 8) as u8])
}

#[wasm_bindgen(js_name = encodeUInt16BE)]
pub fn encode_uint16be(value: u16) -> Vec<u8> {
    Vec::from([((value & 0xff00) >> 8) as u8, (value & 0xff) as u8])
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodingError {
    UnexpectedEndOfData,
}

pub struct HeterogeneousDataReader<'a> {
    data: Box<dyn Iterator<Item = u8> + 'a>,
    pub offset: usize,
}

impl<'a> HeterogeneousDataReader<'a> {
    pub fn new(data: Box<dyn Iterator<Item = u8> + 'a>) -> Self {
        HeterogeneousDataReader { data, offset: 0 }
    }

    pub fn from_offset(data: &'a [u8], offset: usize) -> Self {
        let iterator = data.split_at(offset).1.iter().copied();
        HeterogeneousDataReader {
            data: Box::new(iterator),
            offset,
        }
    }

    pub fn next_u8(&mut self) -> Result<u8, DecodingError> {
        self.offset += 1;
        self.data.next().ok_or(DecodingError::UnexpectedEndOfData)
    }

    pub fn next_u16_le(&mut self) -> Result<u16, DecodingError> {
        let low = self.next_u8()? as u16;
        let high = self.next_u8()? as u16;
        Ok(low | (high << 8))
    }

    pub fn next_null_terminated_string(&mut self) -> Result<String, DecodingError> {
        let mut string = String::new();
        loop {
            let byte = self.next_u8()?;
            if byte == 0 {
                break;
            }
            string.push(byte as char);
        }
        Ok(string)
    }

    pub fn iter_bytes<'i: 'a>(self) -> HeterogeneousDataReaderBytesIterator<'i>
    where
        'a: 'i,
    {
        HeterogeneousDataReaderBytesIterator { reader: self }
    }

    pub fn consume_remaining(self) -> Vec<u8> {
        self.data.collect()
    }
}

pub struct HeterogeneousDataReaderBytesIterator<'a> {
    reader: HeterogeneousDataReader<'a>,
}

impl Iterator for HeterogeneousDataReaderBytesIterator<'_> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        self.reader.next_u8().ok()
    }
}
