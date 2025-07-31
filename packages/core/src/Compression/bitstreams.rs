use std::io::{Read, Seek};

use bitstream_io::{BitReader, BitWrite, Endianness};

use crate::resources::{decode::DecodingError, encode::EncodingError};

pub trait DecodeBitstream<'opt> {
    type Options: 'opt;

    fn decode_bitstream<'a, R: Read + Seek, E: Endianness>(
        data: &'a mut BitReader<R, E>,
        options: Self::Options,
    ) -> Result<Self, DecodingError>
    where
        Self: Sized;
}

pub trait EncodeBitstream<'opt> {
    type Options: 'opt;

    fn encode_bitstream<Out: BitWrite>(
        &self,
        out: &mut Out,
        options: Self::Options,
    ) -> Result<(), EncodingError>;
}
