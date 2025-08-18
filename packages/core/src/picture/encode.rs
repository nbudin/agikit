use bitstream_io::{BigEndian, BitWrite, BitWriter};
#[cfg(feature = "js")]
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

#[cfg(feature = "js")]
use crate::buffer::Buffer;
use crate::{
    compression::bitstreams::EncodeBitstream,
    data_encoding::WriteHeterogeneousData,
    picture::{
        AbsoluteLinePictureCommand, FillPictureCommand, Picture, PictureCommand,
        PictureCommandOpcode, PictureCoordinate, PictureCornerStep, PlotWithPenPictureCommand,
        RelativeLinePictureCommand,
    },
    resources::{
        ResourceType,
        encode::{Encode, EncodeResource, EncodingError},
    },
};

impl EncodeBitstream<'_> for PictureCoordinate {
    type Options = ();

    fn encode_bitstream<Out: BitWrite>(
        &self,
        out: &mut Out,
        _options: Self::Options,
    ) -> Result<(), EncodingError> {
        out.write::<8, u8>(self.x)?;
        out.write::<8, u8>(self.y)?;

        Ok(())
    }
}

impl EncodeBitstream<'_> for Vec<PictureCornerStep> {
    type Options = ();

    fn encode_bitstream<Out: BitWrite>(
        &self,
        out: &mut Out,
        _options: Self::Options,
    ) -> Result<(), EncodingError> {
        for step in self.iter() {
            out.write::<8, u8>(step.position)?;
        }

        Ok(())
    }
}

impl EncodeBitstream<'_> for AbsoluteLinePictureCommand {
    type Options = ();

    fn encode_bitstream<Out: BitWrite>(
        &self,
        out: &mut Out,
        _options: Self::Options,
    ) -> Result<(), EncodingError> {
        for point in self.points.iter() {
            point.encode_bitstream(out, ())?;
        }

        Ok(())
    }
}

impl EncodeBitstream<'_> for RelativeLinePictureCommand {
    type Options = ();

    fn encode_bitstream<Out: BitWrite>(
        &self,
        out: &mut Out,
        _options: Self::Options,
    ) -> Result<(), EncodingError> {
        self.start_position.encode_bitstream(out, ())?;

        for point in self.relative_points.iter() {
            out.write::<8, u8>(point.into_bits())?;
        }

        Ok(())
    }
}

impl EncodeBitstream<'_> for FillPictureCommand {
    type Options = ();

    fn encode_bitstream<Out: BitWrite>(
        &self,
        out: &mut Out,
        _options: Self::Options,
    ) -> Result<(), EncodingError> {
        for start_position in self.start_positions.iter() {
            start_position.encode_bitstream(out, ())?;
        }

        Ok(())
    }
}

impl EncodeBitstream<'_> for PlotWithPenPictureCommand {
    type Options = ();

    fn encode_bitstream<Out: BitWrite>(
        &self,
        out: &mut Out,
        _options: Self::Options,
    ) -> Result<(), EncodingError> {
        for point in self.points.iter() {
            if let Some(texture) = point.texture {
                out.write::<8, u8>(texture)?;
            }

            point.position.encode_bitstream(out, ())?;
        }

        Ok(())
    }
}

impl EncodeBitstream<'_> for PictureCommand {
    type Options = bool; // compress color numbers

    fn encode_bitstream<Out: BitWrite>(
        &self,
        out: &mut Out,
        compress_color_numbers: Self::Options,
    ) -> Result<(), EncodingError> {
        let color_bits = if compress_color_numbers { 4 } else { 8 };

        out.write::<8, u8>(self.opcode().into())?;
        match self {
            PictureCommand::SetPictureColor(command) => {
                out.write_var(color_bits, command.color_number)?;
            }
            PictureCommand::DisablePictureDraw(_) => {}
            PictureCommand::SetPriorityColor(command) => {
                out.write_var(color_bits, command.color_number)?;
            }
            PictureCommand::DisablePriorityDraw(_) => {}
            PictureCommand::DrawYCorner(command) => {
                command.start_position.encode_bitstream(out, ())?;
                command.steps.encode_bitstream(out, ())?;
            }
            PictureCommand::DrawXCorner(command) => {
                command.start_position.encode_bitstream(out, ())?;
                command.steps.encode_bitstream(out, ())?;
            }
            PictureCommand::AbsoluteLine(command) => command.encode_bitstream(out, ())?,
            PictureCommand::RelativeLine(command) => command.encode_bitstream(out, ())?,
            PictureCommand::Fill(command) => command.encode_bitstream(out, ())?,
            PictureCommand::ChangePen(command) => {
                out.write::<8, u8>(command.settings.into_bits().into())?;
            }
            PictureCommand::PlotWithPen(command) => command.encode_bitstream(out, ())?,
            PictureCommand::End(_) => {}
        }

        Ok(())
    }
}

impl EncodeBitstream<'_> for Picture {
    type Options = bool; // compress color numbers

    fn encode_bitstream<Out: BitWrite>(
        &self,
        out: &mut Out,
        compress_color_numbers: Self::Options,
    ) -> Result<(), EncodingError> {
        for command in self.commands.iter() {
            command.encode_bitstream(out, compress_color_numbers)?;
        }

        out.write::<8, u8>(PictureCommandOpcode::End.into())?;

        Ok(())
    }
}

impl Encode<'_> for Picture {
    type Options = bool;

    fn encode<Out: WriteHeterogeneousData>(
        &self,
        out: Out,
        options: bool,
    ) -> Result<(), EncodingError> {
        let mut out_bitstream = BitWriter::endian(out, BigEndian);
        self.encode_bitstream(&mut out_bitstream, options)
    }
}

impl EncodeResource<'_> for Picture {
    fn resource_type(&self) -> ResourceType {
        ResourceType::PIC
    }
}

#[cfg(feature = "js")]
#[wasm_bindgen(js_name = "buildPicture")]
pub fn build_picture(
    picture_resource: Picture,
    compress_color_numbers: bool,
) -> Result<Buffer, JsValue> {
    use bitstream_io::{BigEndian, BitWriter};

    let mut encoded: Vec<u8> = vec![];
    picture_resource
        .encode_bitstream(
            &mut BitWriter::endian(&mut encoded, BigEndian),
            compress_color_numbers,
        )
        .map_err(|e| JsValue::from_str(format!("{}", e).as_str()))?;
    Ok(Buffer::from(encoded))
}
