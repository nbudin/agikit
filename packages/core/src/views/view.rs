use bitfield_struct::bitfield;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
use web_sys::js_sys::Uint8Array;

use crate::{
    color_palettes::ColorPalette,
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

pub struct ViewRLEDataIterator<'a> {
    input: &'a mut dyn Iterator<Item = u8>,
    current_color: u8,
    counter: usize,
}

impl<'a> ViewRLEDataIterator<'a> {
    pub fn new(input: &'a mut dyn Iterator<Item = u8>) -> Self {
        ViewRLEDataIterator {
            input,
            current_color: 0,
            counter: 0,
        }
    }
}

impl<'a> Iterator for ViewRLEDataIterator<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.counter == 0 {
            let byte = self.input.next()?;
            if byte == 0 {
                return None;
            }

            let rle_byte = RLEColorByte::from_bits(byte);
            self.current_color = rle_byte.color();
            self.counter = rle_byte.count() as usize;
        }

        if self.counter == 0 {
            return None;
        }

        self.counter -= 1;

        Some(self.current_color)
    }
}

pub struct ViewCelPixelsIterator<'a> {
    data: &'a [u8],
    is_mirrored: bool,
    width: u8,
    height: u8,
    index: usize,
}

impl<'a> ViewCelPixelsIterator<'a> {
    pub fn new(data: &'a [u8], is_mirrored: bool, width: u8, height: u8) -> Self {
        ViewCelPixelsIterator {
            data,
            is_mirrored,
            width,
            height,
            index: 0,
        }
    }
}

impl<'a> Iterator for ViewCelPixelsIterator<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.width as usize * self.height as usize {
            return None;
        }

        let offset = if self.is_mirrored {
            let row = self.index / self.width as usize;
            let col = self.width as usize - 1 - (self.index % self.width as usize);
            row * self.width as usize + col
        } else {
            self.index
        };

        self.index += 1;

        Some(self.data[offset])
    }
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
                    let mut bytes_iterator = cel_reader.iter_bytes();
                    pixels.extend(ViewRLEDataIterator::new(&mut bytes_iterator).take(pixel_count));

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

    pub fn get_cel(&self, loop_number: u8, cel_number: u8) -> Option<&ViewCel> {
        self.loops
            .get(loop_number as usize)
            .and_then(|loop_| loop_.cels.get(cel_number as usize))
    }

    pub fn get_cel_data<'a>(&'a self, cel: &'a ViewCel) -> (&'a [u8], bool) {
        let data = match &cel.data {
            ViewCelData::NonMirrored(data) => &data.data,
            ViewCelData::Mirrored(mirrored_data) => self
                .loops
                .get(mirrored_data.loop_number as usize)
                .and_then(|loop_| loop_.cels.get(cel.cel_number as usize))
                .and_then(|cel| {
                    if let ViewCelData::NonMirrored(data) = &cel.data {
                        Some(&data.data)
                    } else {
                        None
                    }
                })
                .unwrap(),
        };
        (data.as_slice(), cel.is_mirrored())
    }

    pub fn render_cel(
        &self,
        loop_number: u8,
        cel_number: u8,
        color_palette: &ColorPalette,
    ) -> Option<Vec<u8>> {
        let cel = self.get_cel(loop_number, cel_number)?;
        let (data, is_mirrored) = self.get_cel_data(cel);

        Some(render_view_cel(
            ViewCelPixelsIterator::new(data, is_mirrored, cel.width, cel.height),
            cel.transparent_color,
            color_palette,
        ))
    }
}

pub fn render_view_cel(
    data: impl Iterator<Item = u8>,
    transparent_color: u8,
    color_palette: &ColorPalette,
) -> Vec<u8> {
    data.flat_map(|color| {
        if color == transparent_color {
            [0, 0, 0, 0] // Transparent pixel
        } else {
            let rgb_color = color_palette.colors[color as usize];
            rgb_color
        }
    })
    .collect()
}

#[wasm_bindgen(js_name = "renderViewCel")]
pub fn render_view_cel_from_arrays(
    source_buffer: Uint8Array,
    transparent_color: u8,
    color_palette: &ColorPalette,
) -> Result<Uint8Array, JsValue> {
    let data_vec = source_buffer.to_vec();
    let rendered_data = render_view_cel(data_vec.iter().copied(), transparent_color, color_palette);
    let array_buffer = Uint8Array::new_with_length(rendered_data.len() as u32);
    array_buffer.copy_from(&rendered_data);
    Ok(array_buffer)
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
