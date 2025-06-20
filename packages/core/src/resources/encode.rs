use std::{
    fmt::{Debug, Display},
    io::Cursor,
};

use crate::{data_encoding::WriteHeterogeneousData, logic::encode::AssemblyError};

#[derive(Debug)]
pub enum EncodingError {
    InvalidOptions(String),
    UnencodableData(String),
    IoError(std::io::Error),
    AssemblyError(AssemblyError),
}

impl From<std::io::Error> for EncodingError {
    fn from(error: std::io::Error) -> Self {
        EncodingError::IoError(error)
    }
}

impl From<AssemblyError> for EncodingError {
    fn from(error: AssemblyError) -> Self {
        EncodingError::AssemblyError(error)
    }
}

impl Display for EncodingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncodingError::InvalidOptions(msg) => write!(f, "Invalid options: {}", msg),
            EncodingError::UnencodableData(msg) => write!(f, "Unencodable data: {}", msg),
            EncodingError::IoError(err) => Display::fmt(err, f),
            EncodingError::AssemblyError(err) => Display::fmt(err, f),
        }
    }
}

pub trait Encode<'opt> {
    type Options: 'opt;

    fn encode<Out: WriteHeterogeneousData>(
        &self,
        out: Out,
        options: Self::Options,
    ) -> Result<(), EncodingError>;
    fn encode_to_vec(&self, options: Self::Options) -> Result<Vec<u8>, EncodingError> {
        let mut data = Vec::new();
        let cursor = Cursor::new(&mut data);
        self.encode(cursor, options)?;
        Ok(data)
    }
}
