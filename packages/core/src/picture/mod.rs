pub mod decode;
pub mod encode;
mod picture;
pub mod render;

pub use picture::*;

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use bitstream_io::{BigEndian, BitReader, BitWriter};
    use image::{ImageBuffer, ImageFormat, Pixel, Rgba, RgbaImage};
    use similar_asserts::assert_eq;

    use crate::{
        color_palettes::{ColorPalette, ega_palette},
        compression::bitstreams::{DecodeBitstream, EncodeBitstream},
        picture::{Picture, render::PixelBuffer},
        resources::ResourceType,
        test_data::{kq4demo, uriquest},
        test_utils::write_and_edit_bytes,
    };

    fn render_image(
        pixel_buffer: &PixelBuffer<u8>,
        palette: &ColorPalette,
    ) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let mut image = RgbaImage::new(pixel_buffer.width as u32, pixel_buffer.height as u32);
        for (index, pixel) in pixel_buffer.buffer.iter().enumerate() {
            image.put_pixel(
                (index % pixel_buffer.width) as u32,
                (index / pixel_buffer.height) as u32,
                *Rgba::from_slice(&palette.colors[*pixel as usize]),
            );
        }

        image
    }

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
        let rendered = pic.render();
        let image = render_image(&rendered.visual_buffer, &ega_palette());
        let mut bytes: Vec<u8> = Vec::new();
        image
            .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
            .unwrap();
        write_and_edit_bytes(".png", &bytes);
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

        let rendered = pic.render();
        let image = render_image(&rendered.visual_buffer, &ega_palette());
        let mut bytes: Vec<u8> = Vec::new();
        image
            .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
            .unwrap();
        write_and_edit_bytes(".png", &bytes);
    }
}
