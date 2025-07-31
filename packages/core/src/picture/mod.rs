pub mod decode;
pub mod encode;
mod picture;
pub mod render;

pub use picture::*;

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use bitstream_io::{BigEndian, BitReader, BitWriter};
    use similar_asserts::assert_eq;

    use crate::{
        compression::bitstreams::{DecodeBitstream, EncodeBitstream},
        picture::Picture,
        resources::ResourceType,
        test_data::{kq4demo, uriquest},
    };

    #[test]
    fn smoke_test_agiv2() {
        let pic_data = uriquest()
            .read_resource_data(ResourceType::PIC, 10)
            .unwrap();
        let mut cursor = Cursor::new(&pic_data.data);
        let pic = Picture::decode_bitstream(
            &mut BitReader::endian(&mut cursor, BigEndian),
            pic_data.is_compressed_pic,
        )
        .unwrap();

        let mut reencoded_data: Vec<u8> = vec![];
        let mut cursor = Cursor::new(&mut reencoded_data);
        pic.encode_bitstream(
            &mut BitWriter::endian(&mut cursor, BigEndian),
            pic_data.is_compressed_pic,
        )
        .unwrap();

        assert_eq!(pic_data.data, reencoded_data);
    }

    #[test]
    fn smoke_test_agiv3() {
        let pic_data = kq4demo().read_resource_data(ResourceType::PIC, 1).unwrap();
        let mut cursor = Cursor::new(&pic_data.data);
        let pic = Picture::decode_bitstream(
            &mut BitReader::endian(&mut cursor, BigEndian),
            pic_data.is_compressed_pic,
        )
        .unwrap();

        let mut reencoded_data: Vec<u8> = vec![];
        let mut cursor = Cursor::new(&mut reencoded_data);
        pic.encode_bitstream(
            &mut BitWriter::endian(&mut cursor, BigEndian),
            pic_data.is_compressed_pic,
        )
        .unwrap();

        assert_eq!(pic_data.data, reencoded_data);
    }
}
