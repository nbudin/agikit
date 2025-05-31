use bitfield_struct::bitfield;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
use web_sys::js_sys::Uint8Array;

use crate::{color_palettes::ColorPalette, wasm_utils::Buffer};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NonMirroredViewCelData {
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MirroredViewCelData {
    pub loop_number: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ViewCelData {
    NonMirrored(NonMirroredViewCelData),
    Mirrored(MirroredViewCelData),
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[wasm_bindgen]
pub struct ViewCel {
    #[wasm_bindgen(js_name = "celNumber")]
    pub cel_number: u8,
    pub width: u8,
    pub height: u8,
    #[wasm_bindgen(js_name = "transparentColor")]
    pub transparent_color: u8,
    #[wasm_bindgen(skip)]
    pub data: ViewCelData,
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

#[derive(Clone, Debug)]
pub struct ViewCelHandle<'a> {
    pub loop_number: u8,
    pub cel_number: u8,
    pub width: u8,
    pub height: u8,
    pub transparent_color: u8,
    pub data: &'a Vec<u8>,
    pub mirrored_from_loop_number: Option<u8>,
}

impl<'a> ViewCelHandle<'a> {
    pub fn is_mirrored(&self) -> bool {
        self.mirrored_from_loop_number.is_some()
    }

    pub fn render(&self, color_palette: &ColorPalette) -> Vec<u8> {
        render_view_cel(
            ViewCelPixelsIterator::new(
                self.data,
                self.mirrored_from_loop_number.is_some(),
                self.width,
                self.height,
            ),
            self.transparent_color,
            color_palette,
        )
    }
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
pub struct TransparencyMirroringByte {
    #[bits(4)]
    pub transparent_color: u8,
    #[bits(3)]
    pub mirrored_from_loop_number: u8,
    pub is_mirrored: bool,
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
