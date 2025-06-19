use std::{
    collections::HashMap,
    fmt::Display,
    hash::Hash,
    io::{Cursor, Read, Seek, SeekFrom},
};

use crate::{
    compression::bitstreams::{ReadBitstream, WriteBitstream},
    data_encoding::ReadHeterogeneousData,
};

pub const START_OVER_CODE: u32 = 256;
pub const END_RESOURCE_CODE: u32 = 257;

// AGI's LZW variant reserves 256 and 257 as "start over" and "end resource"
pub const STARTING_SIZE: u32 = 257;

pub trait LZWDictionary<KeyType: Eq + Hash, ValueType> {
    fn get_mapping(&self) -> &HashMap<KeyType, ValueType>;
    fn get_mapping_mut(&mut self) -> &mut HashMap<KeyType, ValueType>;
    fn get_size(&self) -> u32;

    fn has(&self, key: &KeyType) -> bool {
        self.get_mapping().contains_key(key)
    }

    fn get<'a>(&'a self, key: &'a KeyType) -> Option<&'a ValueType> {
        self.get_mapping().get(key)
    }

    fn code_length(&self) -> usize {
        let size = self.get_size() as f64;
        let calculated_size = (size + 1.0).log2().ceil() as usize;
        calculated_size.min(11)
    }

    fn is_full(&self) -> bool {
        self.get_size() >= 2047
    }
}

pub struct LZWBitstreamWriter {
    bytes: Vec<u8>,
    current_byte: u8,
    current_byte_offset: usize,
}

impl LZWBitstreamWriter {
    pub fn new() -> Self {
        LZWBitstreamWriter {
            bytes: Vec::new(),
            current_byte: 0,
            current_byte_offset: 0,
        }
    }
}

impl WriteBitstream for LZWBitstreamWriter {
    fn current_byte_offset(&self) -> usize {
        self.current_byte_offset
    }

    fn current_byte(&self) -> u8 {
        self.current_byte
    }

    fn get_data(&self) -> &[u8] {
        &self.bytes
    }

    fn flush_current_byte(&mut self) {
        self.bytes.push(self.current_byte);
        self.current_byte = 0;
        self.current_byte_offset = 0;
    }

    fn write_code(&mut self, code: u32, bit_length: usize) {
        // AGIv3's LZW implementation writes the low-order bits first
        let mut working_code = code;
        let mut remaining_bits = bit_length;

        while remaining_bits > 0 {
            let written_length = (8 - self.current_byte_offset).min(remaining_bits);
            let mask = 2u32.pow(written_length as u32) - 1;
            let contribution = (working_code & mask) << self.current_byte_offset;

            working_code >>= written_length;

            self.current_byte |= contribution as u8;
            self.current_byte_offset += written_length;
            remaining_bits -= written_length;

            if self.current_byte_offset == 8 {
                self.flush_current_byte();
            }
        }
    }
}

#[derive(Debug)]
pub enum CompressionError {
    KeyNotFound(Vec<u8>),
}

pub struct CompressionDictionary {
    mapping: HashMap<Vec<u8>, u32>,
    size: u32,
}

impl CompressionDictionary {
    pub fn new() -> Self {
        let mut mapping = HashMap::new();
        for i in 0u8..=255 {
            mapping.insert(vec![i], i as u32);
        }

        CompressionDictionary {
            mapping,
            size: STARTING_SIZE,
        }
    }

    pub fn get(&self, word: &[u8]) -> Result<u32, CompressionError> {
        self.mapping
            .get(word)
            .copied()
            .ok_or(CompressionError::KeyNotFound(word.to_vec()))
    }

    pub fn add(&mut self, word: &[u8]) {
        self.size += 1;
        self.mapping.insert(word.to_vec(), self.size);
    }
}

impl LZWDictionary<Vec<u8>, u32> for CompressionDictionary {
    fn get_mapping(&self) -> &HashMap<Vec<u8>, u32> {
        &self.mapping
    }

    fn get_mapping_mut(&mut self) -> &mut HashMap<Vec<u8>, u32> {
        &mut self.mapping
    }

    fn get_size(&self) -> u32 {
        self.size
    }
}

pub fn agi_lzw_compress(compressed: &[u8]) -> Result<Vec<u8>, CompressionError> {
    let mut writer = LZWBitstreamWriter::new();
    let mut dictionary = CompressionDictionary::new();
    let mut word: Vec<u8> = vec![];

    writer.write_code(START_OVER_CODE, 9);

    for &byte in compressed {
        let joined_word = word
            .iter()
            .chain(std::iter::once(&byte))
            .cloned()
            .collect::<Vec<u8>>();

        if dictionary.mapping.contains_key(&joined_word) {
            word = joined_word;
        } else {
            let word_code = dictionary.get(&word)?;
            writer.write_code(word_code, dictionary.code_length());

            if dictionary.is_full() {
                // dictionary overflow!  write a start over code and reset the dictionary
                writer.write_code(START_OVER_CODE, dictionary.code_length());
                dictionary = CompressionDictionary::new();
            } else {
                dictionary.add(&joined_word);
            }

            word = vec![byte];
        }
    }

    if !word.is_empty() {
        let word_code = dictionary.get(&word)?;
        writer.write_code(word_code, dictionary.code_length());
    }

    writer.write_code(END_RESOURCE_CODE, dictionary.code_length());
    Ok(writer.finish())
}

pub struct LZWBitstreamReader<'a, Data: Read + Seek> {
    bitstream: &'a mut Data,
    bit_offset: usize,
    input_bit_buffer: u32,
    input_bit_count: usize,
    bitstream_bit_length: usize,
}

impl<'a, Data: Read + Seek> LZWBitstreamReader<'a, Data> {
    pub fn new(bitstream: &'a mut Data) -> Self {
        let bitstream_bit_length = bitstream.seek(SeekFrom::End(0)).unwrap_or(0) * 8; // Convert byte length to bit length
        bitstream.seek(SeekFrom::Start(0)).unwrap();

        LZWBitstreamReader {
            bitstream,
            bit_offset: 0,
            input_bit_buffer: 0,
            input_bit_count: 0,
            bitstream_bit_length: bitstream_bit_length as usize,
        }
    }
}

impl<'a, Data: Read + Seek> ReadBitstream for LZWBitstreamReader<'a, Data> {
    fn bit_offset(&self) -> usize {
        self.bit_offset
    }

    fn read_code(&mut self, bit_length: usize) -> Result<u32, std::io::Error> {
        while self.input_bit_count <= 24 && self.bit_offset < self.bitstream_bit_length {
            self.bitstream
                .seek(SeekFrom::Start(self.byte_offset() as u64))?;
            let byte = self.bitstream.read_u8()?;
            self.input_bit_buffer |= (byte as u32) << self.input_bit_count;
            self.input_bit_count += 8;
            self.bit_offset += 8;
        }

        let code = (self.input_bit_buffer & 0x7fff) % (1 << bit_length) as u32;
        self.input_bit_buffer >>= bit_length;
        self.input_bit_count = self.input_bit_count.saturating_sub(bit_length);

        Ok(code)
    }

    fn seek_bits(&mut self, bits: isize) -> Result<(), std::io::Error> {
        self.bit_offset = self
            .bit_offset
            .checked_add(bits as usize)
            .ok_or(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Bit offset overflow",
            ))?;
        Ok(())
    }

    fn done(&self) -> bool {
        self.bit_offset >= self.bitstream_bit_length && self.input_bit_count < 8
    }
}

#[derive(Debug)]
pub enum DecompressionError {
    IoError(std::io::Error),
    UnexpectedCode { expected: u32, actual: u32 },
}

impl From<std::io::Error> for DecompressionError {
    fn from(err: std::io::Error) -> Self {
        DecompressionError::IoError(err)
    }
}

impl Display for DecompressionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecompressionError::IoError(err) => err.fmt(f),
            DecompressionError::UnexpectedCode { expected, actual } => {
                write!(f, "Unexpected code: expected {}, got {}", expected, actual)
            }
        }
    }
}

pub struct DecompressionDictionary {
    mapping: HashMap<u32, Vec<u8>>,
    size: u32,
}

impl DecompressionDictionary {
    pub fn new() -> Self {
        let mut mapping = HashMap::new();
        for i in 0u8..=255 {
            mapping.insert(i as u32, vec![i]);
        }

        DecompressionDictionary {
            mapping,
            size: STARTING_SIZE,
        }
    }

    pub fn add(&mut self, word: &[u8]) {
        self.mapping.insert(self.size, word.to_vec());
        self.size += 1;
    }
}

impl LZWDictionary<u32, Vec<u8>> for DecompressionDictionary {
    fn get_mapping(&self) -> &HashMap<u32, Vec<u8>> {
        &self.mapping
    }

    fn get_mapping_mut(&mut self) -> &mut HashMap<u32, Vec<u8>> {
        &mut self.mapping
    }

    fn get_size(&self) -> u32 {
        self.size
    }
}

pub fn agi_lzw_decompress(compressed: &[u8]) -> Result<Vec<u8>, DecompressionError> {
    let mut dictionary = DecompressionDictionary::new();
    let mut cursor = Cursor::new(compressed);
    let mut reader = LZWBitstreamReader::new(&mut cursor);

    let mut word = dictionary
        .get(&reader.read_code(dictionary.code_length())?)
        .cloned()
        .unwrap_or_default();
    let mut result = word.clone();
    let mut entry: Vec<u8>;

    while !reader.done() {
        let code = reader.read_code(dictionary.code_length())?;

        if code == END_RESOURCE_CODE {
            break;
        }

        if code == START_OVER_CODE {
            dictionary = DecompressionDictionary::new();
            word = dictionary
                .get(&reader.read_code(dictionary.code_length())?)
                .cloned()
                .unwrap_or_default();
            dictionary.add(&word);
            result.extend_from_slice(&word);
        } else {
            let dictionary_word = dictionary.get(&code);

            match dictionary_word {
                Some(w) => {
                    entry = w.clone();
                }
                None => {
                    if code == dictionary.get_size() {
                        entry = word.clone();
                        entry.push(word[0]);
                    } else {
                        return Err(DecompressionError::UnexpectedCode {
                            expected: dictionary.size,
                            actual: code,
                        });
                    }
                }
            }

            result.extend_from_slice(&entry);
            let mut new_word = word.clone();
            new_word.push(entry[0]);
            dictionary.add(&new_word);
            word = entry;
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    const KNOWN_STRING: &[u8] = b"TOBEORNOTTOBEORTOBEORNOT";
    const KNOWN_COMPRESSED: &[u32] = &[
        //   T   O   B   E   O   R   N   O   T   TO   BE   OR   TOB  EO   RN   OT   [eof]
        256, 84, 79, 66, 69, 79, 82, 78, 79, 84, 258, 260, 262, 267, 261, 263, 265, 257,
    ];

    #[test]
    fn smoke_test_bitstream() {
        let codes: [u32; 4] = [1, 2, 3, 4];
        let mut writer = LZWBitstreamWriter::new();

        for code in codes {
            writer.write_code(code, 9);
        }

        let encoded = writer.finish();
        let mut cursor = Cursor::new(encoded);
        let mut reader = LZWBitstreamReader::new(&mut cursor);
        let mut decoded: Vec<u32> = vec![];

        while !reader.done() {
            let code = reader.read_code(9).expect("Failed to read code");
            decoded.push(code);
        }

        assert_eq!(decoded, codes);
    }

    #[test]
    fn test_compress_known_string() {
        let compressed = agi_lzw_compress(KNOWN_STRING).expect("Compression failed");

        let mut cursor = Cursor::new(compressed);
        let mut bitstream_reader = LZWBitstreamReader::new(&mut cursor);
        let mut codes = vec![];
        while !bitstream_reader.done() {
            let code = bitstream_reader.read_code(9).expect("Failed to read code");
            codes.push(code);
        }

        assert_eq!(codes, KNOWN_COMPRESSED);
    }

    #[test]
    fn test_decompress_known_string() {
        let mut bitstream_writer = LZWBitstreamWriter::new();
        for code in KNOWN_COMPRESSED {
            bitstream_writer.write_code(*code, 9);
        }
        let bitstream = bitstream_writer.finish();

        let compressed = agi_lzw_decompress(&bitstream).expect("Decompression failed");
        assert_eq!(compressed, KNOWN_STRING);
    }
}

#[cfg(feature = "js")]
pub mod js {
    use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

    use crate::{
        buffer::Buffer,
        compression::lzw::{agi_lzw_compress, agi_lzw_decompress},
    };

    #[wasm_bindgen(js_name = "agiLzwCompress")]
    pub fn js_agi_lzw_compress(uncompressed: Buffer) -> Result<Buffer, JsValue> {
        let uncompressed_bytes: Vec<u8> = uncompressed.into();
        let compressed = agi_lzw_compress(&uncompressed_bytes)
            .map_err(|e| JsValue::from_str(&format!("Compression error: {:?}", e)))?;
        Ok(compressed.into())
    }

    #[wasm_bindgen(js_name = "agiLzwDecompress")]
    pub fn js_agi_lzw_decompress(compressed: Buffer) -> Result<Buffer, JsValue> {
        let compressed_bytes: Vec<u8> = compressed.into();
        let decompressed = agi_lzw_decompress(&compressed_bytes)
            .map_err(|e| JsValue::from_str(&format!("Decompression error: {:?}", e)))?;
        Ok(decompressed.into())
    }
}
