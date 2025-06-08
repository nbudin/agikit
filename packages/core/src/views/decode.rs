use crate::{
    data_encoding::ReadHeterogeneousData,
    resource::{Decode, DecodingError},
};
use std::io::SeekFrom;

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

impl<Data: ReadHeterogeneousData + Clone> Decode<'_, Data> for ViewCel {
    type Options = ViewCelDecodeOptions;

    fn decode<'a>(data: &mut Data, options: Self::Options) -> Result<Self, DecodingError> {
        let width = data.read_u8()?;
        let height = data.read_u8()?;
        let transparency_mirroring_byte = TransparencyMirroringByte::from_bits(data.read_u8()?);

        let data = if transparency_mirroring_byte.is_mirrored()
            && transparency_mirroring_byte.mirrored_from_loop_number() != options.loop_number
        {
            let loop_number = transparency_mirroring_byte.mirrored_from_loop_number();
            ViewCelData::Mirrored(MirroredViewCelData { loop_number })
        } else {
            let pixel_count = width as usize * height as usize;
            let mut pixels = Vec::with_capacity(pixel_count);
            let mut bytes_iterator = data.clone().bytes().map(|b| b.unwrap_or(0));
            pixels.extend(
                ViewRLEDecoder::new(
                    &mut bytes_iterator,
                    width.into(),
                    height.into(),
                    transparency_mirroring_byte.transparent_color(),
                )
                .take(pixel_count),
            );

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

pub struct ViewLoopDecodeOptions {
    pub loop_number: u8,
}

impl<Data: ReadHeterogeneousData + Clone> Decode<'_, Data> for ViewLoop {
    type Options = ViewLoopDecodeOptions;

    fn decode<'a>(data: &'a mut Data, options: Self::Options) -> Result<Self, DecodingError> {
        let loop_offset = data.stream_position()?;
        let cel_count = data.read_u8()?;
        let mut cels = Vec::with_capacity(cel_count as usize);
        let mut cel_offsets = Vec::with_capacity(cel_count as usize);
        for _ in 0..cel_count {
            cel_offsets.push(data.read_u16_le()?);
        }

        for (cel_number, &cel_offset) in cel_offsets.iter().enumerate() {
            data.seek(SeekFrom::Start(loop_offset + cel_offset as u64))?;

            let cel = ViewCel::decode(
                data,
                ViewCelDecodeOptions {
                    loop_number: options.loop_number,
                    cel_number: cel_number as u8,
                },
            )?;

            cels.push(cel);
        }

        Ok(ViewLoop {
            loop_number: options.loop_number,
            cels,
        })
    }
}

impl<Data: ReadHeterogeneousData + Clone> Decode<'_, Data> for AGIView {
    type Options = ();

    fn decode<'a>(data: &'a mut Data, _: Self::Options) -> Result<Self, DecodingError> {
        // AGI Spec says the purpose of the first 2 bytes is unknown :/
        // http://agiwiki.sierrahelp.com/index.php?title=AGI_Specifications:_Chapter_8_-_View_Resources#ss8.1
        data.read_u8()?;
        data.read_u8()?;

        let loop_count = data.read_u8()?;
        let mut loops = Vec::with_capacity(loop_count as usize);
        let description_offset = data.read_u16_le()?;
        let mut loop_offsets = Vec::with_capacity(loop_count as usize);
        for _ in 0..loop_count {
            loop_offsets.push(data.read_u16_le()?);
        }

        let description = if description_offset > 0 {
            data.seek(SeekFrom::Start(description_offset as u64))?;
            Some(data.read_null_terminated_string()?)
        } else {
            None
        };

        for (loop_number, &loop_offset) in loop_offsets.iter().enumerate() {
            data.seek(SeekFrom::Start(loop_offset as u64))?;
            let loop_ = ViewLoop::decode(
                data,
                ViewLoopDecodeOptions {
                    loop_number: loop_number as u8,
                },
            )?;

            loops.push(loop_);
        }

        // Placeholder for actual implementation
        Ok(AGIView { description, loops })
    }
}
