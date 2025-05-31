use crate::data_encoding::DecodingError;

#[derive(Debug, Clone)]
pub enum EncodingError {
    InvalidOptions(String),
}

pub trait Encode {
    type Options;

    fn encode(&self, options: Self::Options) -> Result<Vec<u8>, EncodingError>;
}

pub trait Decode {
    type Options;

    fn decode<'a, Data: Iterator<Item = u8> + 'a>(
        data: &'a mut Data,
        options: Self::Options,
    ) -> Result<Self, DecodingError>
    where
        Self: Sized;
}

pub trait Resource: Encode + Decode {}
impl<T> Resource for T where T: Encode + Decode {}
