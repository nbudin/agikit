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

pub trait Decode<'opt, Data: ReadHeterogeneousData> {
    type Options: 'opt;

    fn decode<'a>(data: &'a mut Data, options: Self::Options) -> Result<Self, DecodingError>
    where
        Self: Sized;
}
