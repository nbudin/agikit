use std::{fmt::Display, io::Cursor};

use crate::{
    compression::lzw::DecompressionError,
    data_encoding::ReadHeterogeneousData,
    logic::decode::DisassemblyError,
    resources::{ResourceNumber, ResourceType, resource_collection::RESOURCE_SIGNATURE},
};

#[derive(Debug)]
pub enum DecodingError {
    IoError(std::io::Error),
    InvalidResourceSignature(u16),
    VolumeNumberMismatch {
        expected: u8,
        actual: u8,
    },
    ResourceNotFound {
        resource_type: ResourceType,
        resource_number: ResourceNumber,
    },
    DecompressionError(DecompressionError),
    DisassemblyError(DisassemblyError),
}

impl std::error::Error for DecodingError {}

impl From<std::io::Error> for DecodingError {
    fn from(error: std::io::Error) -> Self {
        DecodingError::IoError(error)
    }
}

impl From<DecompressionError> for DecodingError {
    fn from(error: DecompressionError) -> Self {
        DecodingError::DecompressionError(error)
    }
}

impl From<DisassemblyError> for DecodingError {
    fn from(error: DisassemblyError) -> Self {
        DecodingError::DisassemblyError(error)
    }
}

impl Display for DecodingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodingError::IoError(error) => error.fmt(f),
            DecodingError::InvalidResourceSignature(signature) => f.write_fmt(format_args!(
                "Invalid resource signature: expected 0x{:04X}, got 0x{:04X}",
                RESOURCE_SIGNATURE, signature
            )),
            DecodingError::VolumeNumberMismatch { expected, actual } => f.write_fmt(format_args!(
                "Volume number mismatch: expected {}, got {}",
                expected, actual
            )),
            DecodingError::ResourceNotFound {
                resource_type,
                resource_number,
            } => f.write_fmt(format_args!(
                "Resource not found: type = {}, number = {}",
                resource_type.as_ref(),
                resource_number
            )),
            DecodingError::DecompressionError(error) => error.fmt(f),
            DecodingError::DisassemblyError(error) => error.fmt(f),
        }
    }
}

pub trait Decode<'opt> {
    type Options: 'opt;

    fn decode<'a, Data: ReadHeterogeneousData>(
        data: &'a mut Data,
        options: Self::Options,
    ) -> Result<Self, DecodingError>
    where
        Self: Sized;

    fn decode_from_bytes(data: &[u8], options: Self::Options) -> Result<Self, DecodingError>
    where
        Self: Sized,
    {
        let mut cursor: Cursor<&[u8]> = Cursor::new(data);
        Self::decode(&mut cursor, options)
    }
}
