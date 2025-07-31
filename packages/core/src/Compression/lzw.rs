use std::{
    collections::HashMap,
    fmt::Display,
    hash::Hash,
    io::{Cursor, ErrorKind},
};

use bitstream_io::{BitRead, BitReader, BitWrite, BitWriter, LittleEndian};

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

    fn code_length(&self) -> u32 {
        let size = self.get_size() as f64;
        let calculated_size = (size + 1.0).log2().ceil() as u32;
        calculated_size.min(11)
    }

    fn is_full(&self) -> bool {
        self.get_size() >= 2047
    }
}

#[derive(Debug)]
pub enum CompressionError {
    KeyNotFound(Vec<u8>),
    IoError(std::io::Error),
}

impl From<std::io::Error> for CompressionError {
    fn from(value: std::io::Error) -> Self {
        CompressionError::IoError(value)
    }
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
    let mut output: Vec<u8> = vec![];
    let mut writer = BitWriter::endian(&mut output, LittleEndian);
    let mut dictionary = CompressionDictionary::new();
    let mut word: Vec<u8> = vec![];

    writer.write::<9, _>(START_OVER_CODE)?;

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
            writer.write_var(dictionary.code_length(), word_code)?;

            if dictionary.is_full() {
                // dictionary overflow!  write a start over code and reset the dictionary
                writer.write_var(dictionary.code_length(), START_OVER_CODE)?;
                dictionary = CompressionDictionary::new();
            } else {
                dictionary.add(&joined_word);
            }

            word = vec![byte];
        }
    }

    if !word.is_empty() {
        let word_code = dictionary.get(&word)?;
        writer.write_var(dictionary.code_length(), word_code)?;
    }

    writer.write_var(dictionary.code_length(), END_RESOURCE_CODE)?;
    writer.byte_align()?;
    writer.flush()?;
    Ok(output)
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
    let mut reader = BitReader::endian(&mut cursor, LittleEndian);

    let mut word = dictionary
        .get(&reader.read_var(dictionary.code_length())?)
        .cloned()
        .unwrap_or_default();
    let mut result = word.clone();
    let mut entry: Vec<u8>;

    loop {
        let read_result = reader.read_var::<u32>(dictionary.code_length());
        let code = match read_result {
            Ok(code) => code,
            Err(err) => match err.kind() {
                ErrorKind::UnexpectedEof => {
                    break;
                }
                _ => return Err(err.into()),
            },
        };
        if code == END_RESOURCE_CODE {
            break;
        }

        if code == START_OVER_CODE {
            dictionary = DecompressionDictionary::new();
            word = dictionary
                .get(&reader.read_var(dictionary.code_length())?)
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
        let mut encoded: Vec<u8> = vec![];
        let mut writer = BitWriter::endian(&mut encoded, LittleEndian);

        for code in codes {
            writer.write::<9, _>(code).unwrap();
        }

        writer.byte_align().unwrap();
        writer.flush().unwrap();

        let mut cursor = Cursor::new(encoded);
        let mut reader = BitReader::endian(&mut cursor, LittleEndian);
        let mut decoded: Vec<u32> = vec![];

        while let Ok(code) = reader.read::<9, u32>() {
            decoded.push(code);
        }

        assert_eq!(decoded, codes);
    }

    #[test]
    fn test_compress_known_string() {
        let compressed = agi_lzw_compress(KNOWN_STRING).expect("Compression failed");

        let mut cursor = Cursor::new(compressed);
        let mut bitstream_reader = BitReader::endian(&mut cursor, LittleEndian);
        let mut codes = vec![];
        while let Ok(code) = bitstream_reader.read::<9, u32>() {
            codes.push(code);
        }

        assert_eq!(codes, KNOWN_COMPRESSED);
    }

    #[test]
    fn test_decompress_known_string() {
        let mut bitstream: Vec<u8> = vec![];
        let mut bitstream_writer = BitWriter::endian(&mut bitstream, LittleEndian);
        for code in KNOWN_COMPRESSED {
            bitstream_writer.write::<9, _>(*code).unwrap();
        }

        let compressed = agi_lzw_decompress(&bitstream).expect("Decompression failed");
        assert_eq!(compressed, KNOWN_STRING);
    }
}

#[cfg(feature = "js")]
pub mod js {
    use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

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
