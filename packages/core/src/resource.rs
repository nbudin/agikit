use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, EnumString};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::data_encoding::ReadHeterogeneousData;

#[derive(Debug)]
pub enum DecodingError {
    IoError(std::io::Error),
}

impl From<std::io::Error> for DecodingError {
    fn from(error: std::io::Error) -> Self {
        DecodingError::IoError(error)
    }
}

#[derive(Debug)]
pub enum EncodingError {
    InvalidOptions(String),
    UnencodableData(String),
    IoError(std::io::Error),
}

impl From<std::io::Error> for EncodingError {
    fn from(error: std::io::Error) -> Self {
        EncodingError::IoError(error)
    }
}

pub trait Encode {
    type Options;

    fn encode(&self, options: Self::Options) -> Result<Vec<u8>, EncodingError>;
}

pub trait Decode<'opt, Data: ReadHeterogeneousData> {
    type Options: 'opt;

    fn decode<'a>(data: &'a mut Data, options: Self::Options) -> Result<Self, DecodingError>
    where
        Self: Sized;
}

pub trait Resource<'dec, Data: ReadHeterogeneousData>: Encode + Decode<'dec, Data> {}
impl<'dec, T, Data: ReadHeterogeneousData> Resource<'dec, Data> for T where
    T: Encode + Decode<'dec, Data>
{
}

#[derive(Debug, Clone, PartialEq, Eq, EnumString, AsRefStr, Serialize, Deserialize)]
pub enum ResourceType {
    LOGIC,
    PIC,
    VIEW,
    SOUND,
}

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export enum ResourceType {
  LOGIC = 'LOGIC',
  PIC = 'PIC',
  VIEW = 'VIEW',
  SOUND = 'SOUND',
}
"#;
