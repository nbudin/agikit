use std::fmt::Display;

use crate::{
    compression::bitstreams::{DecodeBitstream, ReadBitstream},
    picture::{
        AbsoluteLinePictureCommand, ChangePenPictureCommand, DisablePictureDrawPictureCommand,
        DisablePriorityDrawPictureCommand, DrawXCornerPictureCommand, DrawYCornerPictureCommand,
        EndPictureCommand, FillPictureCommand, Picture, PictureCommand, PictureCommandOpcodes,
        PictureCoordinate, PictureCornerStep, PictureCornerStepAxis, PicturePenPlotPoint,
        PicturePenSettings, PlotWithPenPictureCommand, RelativeLinePictureCommand,
        RelativeLinePoint, SetPictureColorPictureCommand, SetPriorityColorPictureCommand,
    },
    resources::decode::DecodingError,
};

#[derive(Debug, Clone)]
pub enum PictureDecodingError {
    UnknownOpcode(u8),
}

impl Display for PictureDecodingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PictureDecodingError::UnknownOpcode(opcode) => {
                f.write_fmt(format_args!("Unknown opcode: {:02X}", opcode))
            }
        }
    }
}

impl DecodeBitstream<'_> for PictureCoordinate {
    type Options = ();

    fn decode_bitstream<'a, Data: ReadBitstream>(
        data: &'a mut Data,
        _options: Self::Options,
    ) -> Result<Self, DecodingError>
    where
        Self: Sized,
    {
        let x = data.read_code(8)? as u8;
        let y = data.read_code(8)? as u8;

        Ok(Self { x, y })
    }
}

impl DecodeBitstream<'_> for Vec<PictureCornerStep> {
    type Options = PictureCornerStepAxis; // starting axis

    fn decode_bitstream<'a, Data: ReadBitstream>(
        data: &'a mut Data,
        start_axis: Self::Options,
    ) -> Result<Self, DecodingError>
    where
        Self: Sized,
    {
        let mut steps: Vec<PictureCornerStep> = vec![];
        let mut axis = start_axis;
        loop {
            let current_byte = data.read_code(8)? as u8;
            if current_byte >= 0xf0 {
                data.seek_bits(-8)?;
                break;
            }

            steps.push(PictureCornerStep {
                axis,
                position: current_byte,
            });
            axis = match axis {
                PictureCornerStepAxis::X => PictureCornerStepAxis::Y,
                PictureCornerStepAxis::Y => PictureCornerStepAxis::X,
            };
        }

        Ok(steps)
    }
}

impl DecodeBitstream<'_> for AbsoluteLinePictureCommand {
    type Options = ();

    fn decode_bitstream<'a, Data: ReadBitstream>(
        data: &'a mut Data,
        _options: Self::Options,
    ) -> Result<Self, DecodingError>
    where
        Self: Sized,
    {
        let mut points: Vec<PictureCoordinate> = vec![];

        loop {
            let current_byte = data.read_code(8)? as u8;
            if current_byte >= 0xf0 {
                data.seek_bits(-8)?;
                break;
            }
            points.push(PictureCoordinate {
                x: current_byte,
                y: data.read_code(8)? as u8,
            });
        }

        Ok(AbsoluteLinePictureCommand { points })
    }
}

impl DecodeBitstream<'_> for RelativeLinePictureCommand {
    type Options = ();

    fn decode_bitstream<'a, Data: ReadBitstream>(
        data: &'a mut Data,
        _options: Self::Options,
    ) -> Result<Self, DecodingError>
    where
        Self: Sized,
    {
        let mut points: Vec<RelativeLinePoint> = vec![];
        let start_position = PictureCoordinate::decode_bitstream(data, ())?;

        loop {
            let current_byte = data.read_code(8)? as u8;
            if current_byte >= 0xf0 {
                data.seek_bits(-8)?;
                break;
            }

            points.push(RelativeLinePoint::from_bits(current_byte));
        }

        Ok(RelativeLinePictureCommand {
            start_position,
            relative_points: points,
        })
    }
}

impl DecodeBitstream<'_> for FillPictureCommand {
    type Options = ();

    fn decode_bitstream<'a, Data: ReadBitstream>(
        data: &'a mut Data,
        _options: Self::Options,
    ) -> Result<Self, DecodingError>
    where
        Self: Sized,
    {
        let mut points: Vec<PictureCoordinate> = vec![];

        loop {
            let current_byte = data.read_code(8)? as u8;
            if current_byte >= 0xf0 {
                data.seek_bits(-8)?;
                break;
            }
            points.push(PictureCoordinate {
                x: current_byte,
                y: data.read_code(8)? as u8,
            });
        }

        Ok(FillPictureCommand {
            start_positions: points,
        })
    }
}

impl DecodeBitstream<'_> for PlotWithPenPictureCommand {
    type Options = bool; // splatter enabled

    fn decode_bitstream<'a, Data: ReadBitstream>(
        data: &'a mut Data,
        splatter_enabled: Self::Options,
    ) -> Result<Self, DecodingError>
    where
        Self: Sized,
    {
        let mut points: Vec<PicturePenPlotPoint> = vec![];

        loop {
            let current_byte = data.read_code(8)? as u8;
            if current_byte >= 0xf0 {
                data.seek_bits(-8)?;
                break;
            }

            if splatter_enabled {
                let texture = current_byte;
                let position = PictureCoordinate::decode_bitstream(data, ())?;
                points.push(PicturePenPlotPoint {
                    position,
                    texture: Some(texture),
                });
            } else {
                let x = current_byte;
                let y = data.read_code(8)? as u8;
                points.push(PicturePenPlotPoint {
                    position: PictureCoordinate { x, y },
                    texture: None,
                });
            }
        }

        Ok(PlotWithPenPictureCommand { points })
    }
}

pub struct PictureDecodeState {
    compress_color_numbers: bool,
    splatter_enabled: bool,
}

impl<'state> DecodeBitstream<'state> for PictureCommand {
    type Options = &'state mut PictureDecodeState;

    fn decode_bitstream<'a, Data: ReadBitstream>(
        data: &'a mut Data,
        state: Self::Options,
    ) -> Result<Self, DecodingError> {
        let opcode_byte = data.read_code(8)? as u8;
        let opcode = PictureCommandOpcodes::try_from(opcode_byte)
            .map_err(|_| PictureDecodingError::UnknownOpcode(opcode_byte))?;

        let command = match opcode {
            PictureCommandOpcodes::SetPictureColor => {
                PictureCommand::SetPictureColor(SetPictureColorPictureCommand {
                    color_number: data.read_code(if state.compress_color_numbers {
                        4
                    } else {
                        8
                    })? as u8,
                })
            }
            PictureCommandOpcodes::DisablePictureDraw => {
                PictureCommand::DisablePictureDraw(DisablePictureDrawPictureCommand)
            }
            PictureCommandOpcodes::SetPriorityColor => {
                PictureCommand::SetPriorityColor(SetPriorityColorPictureCommand {
                    color_number: data.read_code(if state.compress_color_numbers {
                        4
                    } else {
                        8
                    })? as u8,
                })
            }
            PictureCommandOpcodes::DisablePriorityDraw => {
                PictureCommand::DisablePriorityDraw(DisablePriorityDrawPictureCommand)
            }
            PictureCommandOpcodes::DrawYCorner => {
                let start_position = PictureCoordinate::decode_bitstream(data, ())?;
                let steps =
                    Vec::<PictureCornerStep>::decode_bitstream(data, PictureCornerStepAxis::Y)?;

                PictureCommand::DrawYCorner(DrawYCornerPictureCommand {
                    start_position,
                    steps,
                })
            }
            PictureCommandOpcodes::DrawXCorner => {
                let start_position = PictureCoordinate::decode_bitstream(data, ())?;
                let steps =
                    Vec::<PictureCornerStep>::decode_bitstream(data, PictureCornerStepAxis::X)?;

                PictureCommand::DrawXCorner(DrawXCornerPictureCommand {
                    start_position,
                    steps,
                })
            }
            PictureCommandOpcodes::AbsoluteLine => PictureCommand::AbsoluteLine(
                AbsoluteLinePictureCommand::decode_bitstream(data, ())?,
            ),
            PictureCommandOpcodes::RelativeLine => PictureCommand::RelativeLine(
                RelativeLinePictureCommand::decode_bitstream(data, ())?,
            ),
            PictureCommandOpcodes::Fill => {
                PictureCommand::Fill(FillPictureCommand::decode_bitstream(data, ())?)
            }
            PictureCommandOpcodes::ChangePen => {
                let settings = PicturePenSettings::from_bits(data.read_code(8)? as u8);
                PictureCommand::ChangePen(ChangePenPictureCommand { settings })
            }
            PictureCommandOpcodes::PlotWithPen => PictureCommand::PlotWithPen(
                PlotWithPenPictureCommand::decode_bitstream(data, state.splatter_enabled)?,
            ),
            PictureCommandOpcodes::End => PictureCommand::End(EndPictureCommand),
        };

        Ok(command)
    }
}

impl DecodeBitstream<'_> for Picture {
    type Options = bool; // compress_color_numbers

    fn decode_bitstream<'a, Data: ReadBitstream>(
        data: &'a mut Data,
        compress_color_numbers: Self::Options,
    ) -> Result<Self, DecodingError>
    where
        Self: Sized,
    {
        let mut state = PictureDecodeState {
            compress_color_numbers,
            splatter_enabled: false,
        };
        let mut commands: Vec<PictureCommand> = vec![];

        loop {
            let command = PictureCommand::decode_bitstream(data, &mut state)?;

            match command {
                PictureCommand::End(_) => {
                    break;
                }
                PictureCommand::ChangePen(command) => {
                    state.splatter_enabled = command.settings.splatter();
                    commands.push(PictureCommand::ChangePen(command));
                }
                _ => {
                    commands.push(command);
                }
            }
        }

        Ok(Picture { commands })
    }
}
