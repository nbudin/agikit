use bitfield_struct::bitfield;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use strum_macros::AsRefStr;
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

#[derive(Clone, Debug, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
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

#[cfg(feature = "js")]
#[wasm_bindgen]
impl PicturePenSettings {
    #[wasm_bindgen(getter, js_name = "size")]
    pub fn js_size(&self) -> u8 {
        self.size()
    }

    #[wasm_bindgen(getter, js_name = "shape")]
    pub fn js_shape(&self) -> PicturePenShape {
        self.shape()
    }

    #[wasm_bindgen(getter, js_name = "splatter")]
    pub fn js_splatter(&self) -> bool {
        self.splatter()
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PicturePenPlotPoint {
    #[wasm_bindgen(getter_with_clone)]
    pub position: PictureCoordinate,
    pub texture: Option<u8>,
}

#[wasm_bindgen(js_name = "PictureCommandOpcodes")]
#[derive(Clone, Debug, Copy, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum PictureCommandOpcode {
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
    pub fn opcode(&self) -> PictureCommandOpcode {
        PictureCommandOpcode::SetPictureColor
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisablePictureDrawPictureCommand;

#[wasm_bindgen]
impl DisablePictureDrawPictureCommand {
    #[wasm_bindgen(getter)]
    pub fn opcode(&self) -> PictureCommandOpcode {
        PictureCommandOpcode::DisablePictureDraw
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
    pub fn opcode(&self) -> PictureCommandOpcode {
        PictureCommandOpcode::SetPriorityColor
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisablePriorityDrawPictureCommand;

#[wasm_bindgen]
impl DisablePriorityDrawPictureCommand {
    #[wasm_bindgen(getter)]
    pub fn opcode(&self) -> PictureCommandOpcode {
        PictureCommandOpcode::DisablePriorityDraw
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
    pub fn opcode(&self) -> PictureCommandOpcode {
        PictureCommandOpcode::DrawYCorner
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
    pub fn opcode(&self) -> PictureCommandOpcode {
        PictureCommandOpcode::DrawXCorner
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
    pub fn opcode(&self) -> PictureCommandOpcode {
        PictureCommandOpcode::AbsoluteLine
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
    pub fn from_value(value: i8) -> SignedDisplacementValue {
        SignedDisplacementValue::new()
            .with_displacement(value.unsigned_abs())
            .with_negative(value < 0)
    }

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

#[cfg(feature = "js")]
#[wasm_bindgen(js_class = "RelativeLinePoint")]
impl JsRelativeLinePoint {
    #[wasm_bindgen(constructor)]
    pub fn new(x: i8, y: i8) -> JsRelativeLinePoint {
        JsRelativeLinePoint { x, y }
    }
}

#[wasm_bindgen]
impl RelativeLinePictureCommand {
    #[wasm_bindgen(getter)]
    pub fn opcode(&self) -> PictureCommandOpcode {
        PictureCommandOpcode::RelativeLine
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
    pub fn opcode(&self) -> PictureCommandOpcode {
        PictureCommandOpcode::Fill
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
    pub fn opcode(&self) -> PictureCommandOpcode {
        PictureCommandOpcode::ChangePen
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
    pub fn opcode(&self) -> PictureCommandOpcode {
        PictureCommandOpcode::PlotWithPen
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Tsify, AsRefStr)]
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
    pub fn opcode(&self) -> PictureCommandOpcode {
        PictureCommandOpcode::End
    }
}

impl PictureCommand {
    pub fn opcode(&self) -> PictureCommandOpcode {
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
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Picture {
    #[wasm_bindgen(getter_with_clone)]
    pub commands: Vec<PictureCommand>,
}

#[cfg(feature = "js")]
mod js {
    use tsify::serde_wasm_bindgen;
    use wasm_bindgen::{JsValue, prelude::wasm_bindgen};
    use web_sys::js_sys::Object;

    use crate::picture::{
        AbsoluteLinePictureCommand, ChangePenPictureCommand, DisablePictureDrawPictureCommand,
        DisablePriorityDrawPictureCommand, DrawXCornerPictureCommand, DrawYCornerPictureCommand,
        EndPictureCommand, FillPictureCommand, JsRelativeLinePoint, Picture, PictureCommand,
        PictureCoordinate, PictureCornerStep, PictureCornerStepAxis, PicturePenPlotPoint,
        PicturePenSettings, PicturePenShape, PlotWithPenPictureCommand, RelativeLinePictureCommand,
        RelativeLinePoint, SetPictureColorPictureCommand, SetPriorityColorPictureCommand,
        SignedDisplacementValue,
    };

    #[wasm_bindgen(typescript_custom_section)]
    const PICTURE_JSON_TYPES: &'static str = r#"
    export type PictureJSONCommand = Omit<PictureCommand, 'opcode' | 'free'>;
    export type PictureJSON = {
        commands: PictureJSONCommand[];
    };
    export function readPictureJSON(json: PictureJSON): Picture;
    export function buildPictureJSON(picture: Picture): PictureJSON;
    "#;

    #[wasm_bindgen(skip_typescript)]
    pub fn read_picture_json(input: Object) -> Result<Picture, serde_wasm_bindgen::Error> {
        serde_wasm_bindgen::from_value(input.into())
    }

    #[wasm_bindgen(skip_typescript)]
    pub fn build_picture_json(input: Picture) -> Result<JsValue, serde_wasm_bindgen::Error> {
        serde_wasm_bindgen::to_value(&input)
    }

    #[wasm_bindgen]
    impl Picture {
        #[wasm_bindgen(constructor)]
        pub fn new(commands: Vec<PictureCommand>) -> Picture {
            Picture { commands }
        }
    }

    #[wasm_bindgen]
    impl PictureCoordinate {
        #[wasm_bindgen(constructor)]
        pub fn new(x: u8, y: u8) -> PictureCoordinate {
            PictureCoordinate { x, y }
        }
    }

    #[wasm_bindgen]
    impl PictureCornerStep {
        #[wasm_bindgen(constructor)]
        pub fn new(axis: PictureCornerStepAxis, position: u8) -> PictureCornerStep {
            PictureCornerStep { axis, position }
        }
    }

    #[wasm_bindgen]
    impl PicturePenPlotPoint {
        #[wasm_bindgen(constructor)]
        pub fn new(position: PictureCoordinate, texture: Option<u8>) -> PicturePenPlotPoint {
            PicturePenPlotPoint { position, texture }
        }
    }

    #[wasm_bindgen]
    impl PicturePenSettings {
        #[wasm_bindgen(constructor)]
        pub fn js_new(size: u8, shape: PicturePenShape, splatter: bool) -> PicturePenSettings {
            PicturePenSettings::new()
                .with_size(size)
                .with_shape(shape)
                .with_splatter(splatter)
        }
    }

    #[wasm_bindgen]
    impl AbsoluteLinePictureCommand {
        #[wasm_bindgen(constructor)]
        pub fn new(points: Vec<PictureCoordinate>) -> AbsoluteLinePictureCommand {
            AbsoluteLinePictureCommand { points }
        }

        #[wasm_bindgen(js_name = "type", getter, unchecked_return_type = "'AbsoluteLine'")]
        pub fn enum_type(&self) -> String {
            "AbsoluteLine".to_string()
        }

        #[wasm_bindgen(js_name = "toPictureCommand")]
        pub fn to_picture_command(&self) -> PictureCommand {
            PictureCommand::AbsoluteLine(self.clone())
        }
    }

    #[wasm_bindgen]
    impl RelativeLinePictureCommand {
        #[wasm_bindgen(constructor)]
        pub fn new(
            start_position: PictureCoordinate,
            relative_points: Option<Vec<JsRelativeLinePoint>>,
        ) -> RelativeLinePictureCommand {
            let relative_points = relative_points
                .unwrap_or_default()
                .into_iter()
                .map(|p| {
                    RelativeLinePoint::new()
                        .with_x_displacement(SignedDisplacementValue::from_value(p.x))
                        .with_y_displacement(SignedDisplacementValue::from_value(p.y))
                })
                .collect();
            RelativeLinePictureCommand {
                start_position,
                relative_points,
            }
        }

        #[wasm_bindgen(js_name = "type", getter, unchecked_return_type = "'RelativeLine'")]
        pub fn enum_type(&self) -> String {
            "RelativeLine".to_string()
        }

        #[wasm_bindgen(js_name = "toPictureCommand")]
        pub fn to_picture_command(&self) -> PictureCommand {
            PictureCommand::RelativeLine(self.clone())
        }
    }

    #[wasm_bindgen]
    impl DrawXCornerPictureCommand {
        #[wasm_bindgen(constructor)]
        pub fn new(
            start_position: PictureCoordinate,
            steps: Option<Vec<PictureCornerStep>>,
        ) -> DrawXCornerPictureCommand {
            DrawXCornerPictureCommand {
                start_position,
                steps: steps.unwrap_or_default(),
            }
        }

        #[wasm_bindgen(js_name = "type", getter, unchecked_return_type = "'DrawXCorner'")]
        pub fn enum_type(&self) -> String {
            "DrawXCorner".to_string()
        }

        #[wasm_bindgen(js_name = "toPictureCommand")]
        pub fn to_picture_command(&self) -> PictureCommand {
            PictureCommand::DrawXCorner(self.clone())
        }
    }

    #[wasm_bindgen]
    impl DrawYCornerPictureCommand {
        #[wasm_bindgen(constructor)]
        pub fn new(
            start_position: PictureCoordinate,
            steps: Option<Vec<PictureCornerStep>>,
        ) -> DrawYCornerPictureCommand {
            DrawYCornerPictureCommand {
                start_position,
                steps: steps.unwrap_or_default(),
            }
        }

        #[wasm_bindgen(js_name = "type", getter, unchecked_return_type = "'DrawYCorner'")]
        pub fn enum_type(&self) -> String {
            "DrawYCorner".to_string()
        }

        #[wasm_bindgen(js_name = "toPictureCommand")]
        pub fn to_picture_command(&self) -> PictureCommand {
            PictureCommand::DrawYCorner(self.clone())
        }
    }

    #[wasm_bindgen]
    impl FillPictureCommand {
        #[wasm_bindgen(constructor)]
        pub fn new(start_positions: Vec<PictureCoordinate>) -> FillPictureCommand {
            FillPictureCommand { start_positions }
        }

        #[wasm_bindgen(js_name = "type", getter, unchecked_return_type = "'Fill'")]
        pub fn enum_type(&self) -> String {
            "Fill".to_string()
        }

        #[wasm_bindgen(js_name = "toPictureCommand")]
        pub fn to_picture_command(&self) -> PictureCommand {
            PictureCommand::Fill(self.clone())
        }
    }

    #[wasm_bindgen]
    impl PlotWithPenPictureCommand {
        #[wasm_bindgen(constructor)]
        pub fn new(points: Vec<PicturePenPlotPoint>) -> PlotWithPenPictureCommand {
            PlotWithPenPictureCommand { points }
        }

        #[wasm_bindgen(js_name = "type", getter, unchecked_return_type = "'PlotWithPen'")]
        pub fn enum_type(&self) -> String {
            "PlotWithPen".to_string()
        }

        #[wasm_bindgen(js_name = "toPictureCommand")]
        pub fn to_picture_command(&self) -> PictureCommand {
            PictureCommand::PlotWithPen(self.clone())
        }
    }

    #[wasm_bindgen]
    impl DisablePictureDrawPictureCommand {
        #[wasm_bindgen(constructor)]
        pub fn new() -> DisablePictureDrawPictureCommand {
            DisablePictureDrawPictureCommand
        }

        #[wasm_bindgen(
            js_name = "type",
            getter,
            unchecked_return_type = "'DisablePictureDraw'"
        )]
        pub fn enum_type(&self) -> String {
            "DisablePictureDraw".to_string()
        }

        #[wasm_bindgen(js_name = "toPictureCommand")]
        pub fn to_picture_command(&self) -> PictureCommand {
            PictureCommand::DisablePictureDraw(self.clone())
        }
    }

    #[wasm_bindgen]
    impl SetPictureColorPictureCommand {
        #[wasm_bindgen(constructor)]
        pub fn new(color_number: u8) -> SetPictureColorPictureCommand {
            SetPictureColorPictureCommand { color_number }
        }

        #[wasm_bindgen(js_name = "type", getter, unchecked_return_type = "'SetPictureColor'")]
        pub fn enum_type(&self) -> String {
            "SetPictureColor".to_string()
        }

        #[wasm_bindgen(js_name = "toPictureCommand")]
        pub fn to_picture_command(&self) -> PictureCommand {
            PictureCommand::SetPictureColor(self.clone())
        }
    }

    #[wasm_bindgen]
    impl DisablePriorityDrawPictureCommand {
        #[wasm_bindgen(constructor)]
        pub fn new() -> DisablePriorityDrawPictureCommand {
            DisablePriorityDrawPictureCommand
        }

        #[wasm_bindgen(
            js_name = "type",
            getter,
            unchecked_return_type = "'DisablePriorityDraw'"
        )]
        pub fn enum_type(&self) -> String {
            "DisablePriorityDraw".to_string()
        }

        #[wasm_bindgen(js_name = "toPictureCommand")]
        pub fn to_picture_command(&self) -> PictureCommand {
            PictureCommand::DisablePriorityDraw(self.clone())
        }
    }

    #[wasm_bindgen]
    impl SetPriorityColorPictureCommand {
        #[wasm_bindgen(constructor)]
        pub fn new(color_number: u8) -> SetPriorityColorPictureCommand {
            SetPriorityColorPictureCommand { color_number }
        }

        #[wasm_bindgen(js_name = "type", getter, unchecked_return_type = "'SetPriorityColor'")]
        pub fn enum_type(&self) -> String {
            "SetPriorityColor".to_string()
        }

        #[wasm_bindgen(js_name = "toPictureCommand")]
        pub fn to_picture_command(&self) -> PictureCommand {
            PictureCommand::SetPriorityColor(self.clone())
        }
    }

    #[wasm_bindgen]
    impl ChangePenPictureCommand {
        #[wasm_bindgen(constructor)]
        pub fn new(settings: PicturePenSettings) -> ChangePenPictureCommand {
            ChangePenPictureCommand { settings }
        }

        #[wasm_bindgen(js_name = "type", getter, unchecked_return_type = "'ChangePen'")]
        pub fn enum_type(&self) -> String {
            "ChangePen".to_string()
        }

        #[wasm_bindgen(js_name = "toPictureCommand")]
        pub fn to_picture_command(&self) -> PictureCommand {
            PictureCommand::ChangePen(self.clone())
        }
    }

    #[wasm_bindgen]
    impl EndPictureCommand {
        #[wasm_bindgen(constructor)]
        pub fn new() -> EndPictureCommand {
            EndPictureCommand
        }

        #[wasm_bindgen(js_name = "type", getter, unchecked_return_type = "'End'")]
        pub fn enum_type(&self) -> String {
            "End".to_string()
        }

        #[wasm_bindgen(js_name = "toPictureCommand")]
        pub fn to_picture_command(&self) -> PictureCommand {
            PictureCommand::End(self.clone())
        }
    }
}
