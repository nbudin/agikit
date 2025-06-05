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
}

pub trait Encode {
    type Options;

    fn encode(&self, options: Self::Options) -> Result<Vec<u8>, EncodingError>;
}

pub trait Decode<'opt> {
    type Options: 'opt;

    fn decode<'a, Data: ReadHeterogeneousData>(
        data: &'a mut Data,
        options: Self::Options,
    ) -> Result<Self, DecodingError>
    where
        Self: Sized;
}

pub trait Resource<'dec>: Encode + Decode<'dec> {}
impl<'dec, T> Resource<'dec> for T where T: Encode + Decode<'dec> {}
