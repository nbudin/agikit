use bitfield_struct::bitfield;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
use web_sys::js_sys::Uint8Array;

use crate::{
    data_encoding::{DecodingError, HeterogeneousDataReader},
    wasm_utils::Buffer,
};

#[derive(Clone, Debug)]
pub struct NonMirroredViewCelData {
    pub data: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct MirroredViewCelData {
    pub loop_number: u8,
}

#[derive(Clone, Debug)]
pub enum ViewCelData {
    NonMirrored(NonMirroredViewCelData),
    Mirrored(MirroredViewCelData),
}

#[derive(Clone, Debug)]
#[wasm_bindgen]
pub struct ViewCel {
    #[wasm_bindgen(js_name = "celNumber")]
    pub cel_number: u8,
    pub width: u8,
    pub height: u8,
    #[wasm_bindgen(js_name = "transparentColor")]
    pub transparent_color: u8,
    data: ViewCelData,
}

#[wasm_bindgen]
impl ViewCel {
    #[wasm_bindgen(getter = "mirrored")]
    pub fn is_mirrored(&self) -> bool {
        matches!(self.data, ViewCelData::Mirrored(_))
    }

    #[wasm_bindgen(getter = "mirroredFromLoopNumber")]
    pub fn mirrored_from_loop_number(&self) -> Option<u8> {
        if let ViewCelData::Mirrored(data) = &self.data {
            Some(data.loop_number)
        } else {
            None
        }
    }

    #[wasm_bindgen(getter = "buffer")]
    pub fn js_buffer(&self) -> Option<Buffer> {
        if let ViewCelData::NonMirrored(data) = &self.data {
            let js_array = Uint8Array::new_with_length(data.data.len() as u32);
            Some(Buffer::from_array_buffer(&js_array.buffer()))
        } else {
            None
        }
    }
}

#[bitfield(u8)]
struct TransparencyMirroringByte {
    #[bits(4)]
    transparent_color: u8,
    #[bits(3)]
    mirrored_from_loop_number: u8,
    is_mirrored: bool,
}

#[bitfield(u8)]
struct RLEColorByte {
    #[bits(4)]
    count: u8,
    #[bits(4)]
    color: u8,
}

#[derive(Clone, Debug)]
#[wasm_bindgen]
pub struct ViewLoop {
    pub loop_number: u8,
    #[wasm_bindgen(getter_with_clone)]
    pub cels: Vec<ViewCel>,
}

#[derive(Clone, Debug)]
#[wasm_bindgen]
pub struct AGIView {
    #[wasm_bindgen(getter_with_clone)]
    pub description: Option<String>,
    #[wasm_bindgen(getter_with_clone)]
    pub loops: Vec<ViewLoop>,
}

impl AGIView {
    pub fn from_data<'a>(data: Box<dyn Iterator<Item = u8> + 'a>) -> Result<Self, DecodingError> {
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
                let mut cel_reader = HeterogeneousDataReader::from_offset(
                    &rest,
                    cel_offset as usize - header_length,
                );

                let width = cel_reader.next_u8()?;
                let height = cel_reader.next_u8()?;
                let transparency_mirroring_byte =
                    TransparencyMirroringByte::from_bits(cel_reader.next_u8()?);

                let data = if transparency_mirroring_byte.is_mirrored()
                    && transparency_mirroring_byte.mirrored_from_loop_number() as usize
                        != loop_number
                {
                    let loop_number = transparency_mirroring_byte.mirrored_from_loop_number();
                    ViewCelData::Mirrored(MirroredViewCelData { loop_number })
                } else {
                    let pixel_count = width as usize * height as usize;
                    let mut pixels = Vec::with_capacity(pixel_count);

                    loop {
                        if pixels.len() >= pixel_count {
                            break;
                        }
                        let byte = cel_reader.next_u8()?;
                        if byte > 0 {
                            let byte = RLEColorByte::from_bits(byte);
                            for _ in 0..byte.count() {
                                pixels.push(byte.color());
                            }
                        } else {
                            break;
                        }
                    }

                    if pixels.len() < pixel_count {
                        let remaining = pixel_count - pixels.len();
                        pixels.extend(
                            std::iter::repeat(transparency_mirroring_byte.transparent_color())
                                .take(remaining),
                        );
                    }

                    ViewCelData::NonMirrored(NonMirroredViewCelData { data: pixels })
                };

                cels.push(ViewCel {
                    cel_number: cel_number as u8,
                    width,
                    height,
                    transparent_color: transparency_mirroring_byte.transparent_color(),
                    data,
                });
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

#[wasm_bindgen(js_name = "readViewResource")]
pub fn read_view_resource(data: Buffer) -> Result<AGIView, JsValue> {
    let data_array = Uint8Array::new(&data);
    let data_vec = data_array.to_vec();
    AGIView::from_data(Box::new(data_vec.iter().copied()))
        .map_err(|e| JsValue::from_str(format!("{:?}", e).as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;

    static VIEW_DATA: &[u8] = include_bytes!("../../test_data/1.agiview");

    #[test]
    fn smoke_test_read_view() {
        let view = AGIView::from_data(Box::new(VIEW_DATA.iter().copied())).unwrap();
        assert_eq!(view.loops.len(), 4);
        assert_eq!(view.loops[0].cels.len(), 8);
        for cel in &view.loops[0].cels {
            assert!(!cel.is_mirrored())
        }
        for cel in &view.loops[1].cels {
            assert!(cel.is_mirrored())
        }
    }
}
