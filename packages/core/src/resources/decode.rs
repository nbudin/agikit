use std::fmt::Display;

use crate::{
    data_encoding::ReadHeterogeneousData,
    resources::{resource_collection::RESOURCE_SIGNATURE, ResourceNumber, ResourceType},
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
}

impl From<std::io::Error> for DecodingError {
    fn from(error: std::io::Error) -> Self {
        DecodingError::IoError(error)
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
        }
    }
}

pub trait Decode<'opt, Data: ReadHeterogeneousData> {
    type Options: 'opt;

    fn decode<'a>(data: &'a mut Data, options: Self::Options) -> Result<Self, DecodingError>
    where
        Self: Sized;
}
