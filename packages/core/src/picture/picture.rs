use bitfield_struct::bitfield;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PictureCoordinate {
    pub x: u8,
    pub y: u8,
}

#[wasm_bindgen]
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum PictureCornerStepAxis {
    X,
    Y,
}

#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PictureCornerStep {
    pub axis: PictureCornerStepAxis,
    pub position: u8,
}

#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PictureRelativeLineDisplacement {
    #[wasm_bindgen(js_name = "xDisplacement")]
    pub x_displacement: u8,
    #[wasm_bindgen(js_name = "yDisplacement")]
    pub y_displacement: u8,
}

#[wasm_bindgen]
#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub enum PicturePenShape {
    Rectangle = 1,
    Circle = 0,
}

impl PicturePenShape {
    const fn into_bits(self) -> u8 {
        self as _
    }
    const fn from_bits(value: u8) -> Self {
        match value {
            0 => Self::Circle,
            _ => Self::Rectangle,
        }
    }
}

#[bitfield(u8)]
#[derive(Serialize, Deserialize)]
#[wasm_bindgen]
pub struct PicturePenSettings {
    #[bits(3)]
    pub size: u8,
    pub _unused: bool,
    #[bits(1)]
    pub shape: PicturePenShape,
    pub splatter: bool,
    #[bits(2)]
    pub _unused2: u8,
}

#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PicturePenPlotPoint {
    #[wasm_bindgen(getter_with_clone)]
    pub position: PictureCoordinate,
    pub texture: Option<u8>,
}

#[wasm_bindgen]
#[derive(Clone, Debug, Copy, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum PictureCommandOpcodes {
    SetPictureColor = 0xf0,
    DisablePictureDraw = 0xf1,
    SetPriorityColor = 0xf2,
    DisablePriorityDraw = 0xf3,
    DrawYCorner = 0xf4,
    DrawXCorner = 0xf5,
    AbsoluteLine = 0xf6,
    RelativeLine = 0xf7,
    Fill = 0xf8,
    ChangePen = 0xf9,
    PlotWithPen = 0xfa,
    End = 0xff,
}

#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetPictureColorPictureCommand {
    #[wasm_bindgen(js_name = "colorNumber")]
    pub color_number: u8,
}

#[wasm_bindgen]
impl SetPictureColorPictureCommand {
    #[wasm_bindgen(getter)]
    pub fn opcode(&self) -> PictureCommandOpcodes {
        PictureCommandOpcodes::SetPictureColor
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisablePictureDrawPictureCommand;

#[wasm_bindgen]
impl DisablePictureDrawPictureCommand {
    #[wasm_bindgen(getter)]
    pub fn opcode(&self) -> PictureCommandOpcodes {
        PictureCommandOpcodes::DisablePictureDraw
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetPriorityColorPictureCommand {
    #[wasm_bindgen(js_name = "colorNumber")]
    pub color_number: u8,
}

#[wasm_bindgen]
impl SetPriorityColorPictureCommand {
    #[wasm_bindgen(getter)]
    pub fn opcode(&self) -> PictureCommandOpcodes {
        PictureCommandOpcodes::SetPriorityColor
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisablePriorityDrawPictureCommand;

#[wasm_bindgen]
impl DisablePriorityDrawPictureCommand {
    #[wasm_bindgen(getter)]
    pub fn opcode(&self) -> PictureCommandOpcodes {
        PictureCommandOpcodes::DisablePriorityDraw
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrawYCornerPictureCommand {
    #[wasm_bindgen(getter_with_clone, js_name = "startPosition")]
    pub start_position: PictureCoordinate,
    #[wasm_bindgen(getter_with_clone)]
    pub steps: Vec<PictureCornerStep>,
}

#[wasm_bindgen]
impl DrawYCornerPictureCommand {
    #[wasm_bindgen(getter)]
    pub fn opcode(&self) -> PictureCommandOpcodes {
        PictureCommandOpcodes::DrawYCorner
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrawXCornerPictureCommand {
    #[wasm_bindgen(getter_with_clone, js_name = "startPosition")]
    pub start_position: PictureCoordinate,
    #[wasm_bindgen(getter_with_clone)]
    pub steps: Vec<PictureCornerStep>,
}

#[wasm_bindgen]
impl DrawXCornerPictureCommand {
    #[wasm_bindgen(getter)]
    pub fn opcode(&self) -> PictureCommandOpcodes {
        PictureCommandOpcodes::DrawXCorner
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbsoluteLinePictureCommand {
    #[wasm_bindgen(getter_with_clone)]
    pub points: Vec<PictureCoordinate>,
}

#[wasm_bindgen]
impl AbsoluteLinePictureCommand {
    #[wasm_bindgen(getter)]
    pub fn opcode(&self) -> PictureCommandOpcodes {
        PictureCommandOpcodes::AbsoluteLine
    }
}

#[bitfield(u8)]
pub struct SignedDisplacementValue {
    #[bits(3)]
    displacement: u8,
    negative: bool,
    #[bits(4)]
    _padding: u8,
}

impl SignedDisplacementValue {
    pub fn value(&self) -> i8 {
        (self.displacement() as i8) * (if self.negative() { -1 } else { 1 })
    }
}

#[bitfield(u8)]
#[derive(Serialize, Deserialize)]
pub struct RelativeLinePoint {
    #[bits(4)]
    pub y_displacement: SignedDisplacementValue,
    #[bits(4)]
    pub x_displacement: SignedDisplacementValue,
}

#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelativeLinePictureCommand {
    #[wasm_bindgen(getter_with_clone, js_name = "startPosition")]
    pub start_position: PictureCoordinate,
    #[wasm_bindgen(skip)]
    pub relative_points: Vec<RelativeLinePoint>,
}

#[cfg(feature = "js")]
#[wasm_bindgen(js_name = "RelativeLinePoint")]
pub struct JsRelativeLinePoint {
    pub x: i8,
    pub y: i8,
}

#[wasm_bindgen]
impl RelativeLinePictureCommand {
    #[wasm_bindgen(getter)]
    pub fn opcode(&self) -> PictureCommandOpcodes {
        PictureCommandOpcodes::RelativeLine
    }

    #[cfg(feature = "js")]
    #[wasm_bindgen(getter, js_name = "relativePoints")]
    pub fn js_relative_points(&self) -> Vec<JsRelativeLinePoint> {
        self.relative_points
            .iter()
            .map(|point| JsRelativeLinePoint {
                x: point.x_displacement().value(),
                y: point.y_displacement().value(),
            })
            .collect()
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FillPictureCommand {
    #[wasm_bindgen(getter_with_clone, js_name = "startPositions")]
    pub start_positions: Vec<PictureCoordinate>,
}

#[wasm_bindgen]
impl FillPictureCommand {
    #[wasm_bindgen(getter)]
    pub fn opcode(&self) -> PictureCommandOpcodes {
        PictureCommandOpcodes::Fill
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangePenPictureCommand {
    #[wasm_bindgen(getter_with_clone)]
    pub settings: PicturePenSettings,
}

#[wasm_bindgen]
impl ChangePenPictureCommand {
    #[wasm_bindgen(getter)]
    pub fn opcode(&self) -> PictureCommandOpcodes {
        PictureCommandOpcodes::ChangePen
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlotWithPenPictureCommand {
    #[wasm_bindgen(getter_with_clone)]
    pub points: Vec<PicturePenPlotPoint>,
}

#[wasm_bindgen]
impl PlotWithPenPictureCommand {
    #[wasm_bindgen(getter)]
    pub fn opcode(&self) -> PictureCommandOpcodes {
        PictureCommandOpcodes::PlotWithPen
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "type")]
pub enum PictureCommand {
    SetPictureColor(SetPictureColorPictureCommand),
    DisablePictureDraw(DisablePictureDrawPictureCommand),
    SetPriorityColor(SetPriorityColorPictureCommand),
    DisablePriorityDraw(DisablePriorityDrawPictureCommand),
    DrawYCorner(DrawYCornerPictureCommand),
    DrawXCorner(DrawXCornerPictureCommand),
    AbsoluteLine(AbsoluteLinePictureCommand),
    RelativeLine(RelativeLinePictureCommand),
    Fill(FillPictureCommand),
    ChangePen(ChangePenPictureCommand),
    PlotWithPen(PlotWithPenPictureCommand),
    End(EndPictureCommand),
}

#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EndPictureCommand;

#[wasm_bindgen]
impl EndPictureCommand {
    #[wasm_bindgen(getter)]
    pub fn opcode(&self) -> PictureCommandOpcodes {
        PictureCommandOpcodes::End
    }
}

impl PictureCommand {
    pub fn opcode(&self) -> PictureCommandOpcodes {
        match self {
            PictureCommand::SetPictureColor(set_picture_color_picture_command) => {
                set_picture_color_picture_command.opcode()
            }
            PictureCommand::DisablePictureDraw(disable_picture_draw_picture_command) => {
                disable_picture_draw_picture_command.opcode()
            }
            PictureCommand::SetPriorityColor(set_priority_color_picture_command) => {
                set_priority_color_picture_command.opcode()
            }
            PictureCommand::DisablePriorityDraw(disable_priority_draw_picture_command) => {
                disable_priority_draw_picture_command.opcode()
            }
            PictureCommand::DrawYCorner(draw_ycorner_picture_command) => {
                draw_ycorner_picture_command.opcode()
            }
            PictureCommand::DrawXCorner(draw_xcorner_picture_command) => {
                draw_xcorner_picture_command.opcode()
            }
            PictureCommand::AbsoluteLine(absolute_line_picture_command) => {
                absolute_line_picture_command.opcode()
            }
            PictureCommand::RelativeLine(relative_line_picture_command) => {
                relative_line_picture_command.opcode()
            }
            PictureCommand::Fill(fill_picture_command) => fill_picture_command.opcode(),
            PictureCommand::ChangePen(change_pen_picture_command) => {
                change_pen_picture_command.opcode()
            }
            PictureCommand::PlotWithPen(plot_with_pen_picture_command) => {
                plot_with_pen_picture_command.opcode()
            }
            PictureCommand::End(end_picture_command) => end_picture_command.opcode(),
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct Picture {
    #[wasm_bindgen(getter_with_clone)]
    pub commands: Vec<PictureCommand>,
}
