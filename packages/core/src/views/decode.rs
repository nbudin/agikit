use crate::{
    data_encoding::{DecodingError, HeterogeneousDataReader},
    resource::Decode,
};

use super::{
    cel::{
        MirroredViewCelData, NonMirroredViewCelData, TransparencyMirroringByte, ViewCel,
        ViewCelData,
    },
    rle::ViewRLEDecoder,
    AGIView, ViewLoop,
};

pub struct ViewCelDecodeOptions {
    pub loop_number: u8,
    pub cel_number: u8,
}

impl ViewCelDecodeOptions {
    pub fn new(loop_number: u8, cel_number: u8) -> Self {
        Self {
            loop_number,
            cel_number,
        }
    }
}

impl Decode for ViewCel {
    type Options = ViewCelDecodeOptions;

    fn decode<'a, Data: Iterator<Item = u8> + 'a>(
        data: &mut Data,
        options: Self::Options,
    ) -> Result<Self, DecodingError> {
        let mut cel_reader = HeterogeneousDataReader::new(data);

        let width = cel_reader.next_u8()?;
        let height = cel_reader.next_u8()?;
        let transparency_mirroring_byte =
            TransparencyMirroringByte::from_bits(cel_reader.next_u8()?);

        let data = if transparency_mirroring_byte.is_mirrored()
            && transparency_mirroring_byte.mirrored_from_loop_number() != options.loop_number
        {
            let loop_number = transparency_mirroring_byte.mirrored_from_loop_number();
            ViewCelData::Mirrored(MirroredViewCelData { loop_number })
        } else {
            let pixel_count = width as usize * height as usize;
            let mut pixels = Vec::with_capacity(pixel_count);
            let mut bytes_iterator = cel_reader.iter_bytes();
            pixels.extend(ViewRLEDecoder::new(&mut bytes_iterator).take(pixel_count));

            if pixels.len() < pixel_count {
                let remaining = pixel_count - pixels.len();
                pixels.extend(
                    std::iter::repeat(transparency_mirroring_byte.transparent_color())
                        .take(remaining),
                );
            }

            ViewCelData::NonMirrored(NonMirroredViewCelData { data: pixels })
        };

        Ok(ViewCel {
            cel_number: options.cel_number,
            width,
            height,
            transparent_color: transparency_mirroring_byte.transparent_color(),
            data,
        })
    }
}

impl Decode for AGIView {
    type Options = ();

    fn decode<'a, Data: Iterator<Item = u8> + 'a>(
        data: &'a mut Data,
        _: Self::Options,
    ) -> Result<Self, DecodingError> {
        let mut data = HeterogeneousDataReader::new(data);

        // AGI Spec says the purpose of the first 2 bytes is unknown :/
        // http://agiwiki.sierrahelp.com/index.php?title=AGI_Specifications:_Chapter_8_-_View_Resources#ss8.1
        data.next_u8()?;
        data.next_u8()?;

        let loop_count = data.next_u8()?;
        let mut loops = Vec::with_capacity(loop_count as usize);
        let description_offset = data.next_u16_le()?;
        let mut loop_offsets = Vec::with_capacity(loop_count as usize);
        for _ in 0..loop_count {
            loop_offsets.push(data.next_u16_le()?);
        }

        let header_length = data.offset;
        let rest = data.consume_remaining();

        let description = if description_offset > 0 {
            let mut description_reader = HeterogeneousDataReader::from_offset(
                &rest,
                description_offset as usize - header_length,
            );
            Some(description_reader.next_null_terminated_string()?)
        } else {
            None
        };

        for (loop_number, &loop_offset) in loop_offsets.iter().enumerate() {
            let mut loop_reader =
                HeterogeneousDataReader::from_offset(&rest, loop_offset as usize - header_length);

            let cel_count = loop_reader.next_u8()?;
            let mut cels = Vec::with_capacity(cel_count as usize);
            let mut cel_offsets = Vec::with_capacity(cel_count as usize);
            for _ in 0..cel_count {
                cel_offsets.push(loop_offset + loop_reader.next_u16_le()?);
            }

            for (cel_number, &cel_offset) in cel_offsets.iter().enumerate() {
                let cel_reader = HeterogeneousDataReader::from_offset(
                    &rest,
                    cel_offset as usize - header_length,
                );

                let cel = ViewCel::decode(
                    &mut cel_reader.iter_bytes(),
                    ViewCelDecodeOptions::new(loop_number as u8, cel_number as u8),
                )?;

                cels.push(cel);
            }

            loops.push(ViewLoop {
                loop_number: loop_number as u8,
                cels,
            });
        }

        // Placeholder for actual implementation
        Ok(AGIView { description, loops })
    }
}
