use std::io::Cursor;

use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
use web_sys::js_sys::Uint8Array;

use crate::{buffer::Buffer, color_palettes::ColorPalette, resources::decode::Decode};

use super::cel::{render_view_cel, ViewCel, ViewCelData, ViewCelHandle, ViewCelPixelsIterator};

#[derive(Clone, Debug, PartialEq, Eq)]
#[wasm_bindgen]
pub struct ViewLoop {
    #[wasm_bindgen(js_name = "loopNumber")]
    pub loop_number: u8,
    #[wasm_bindgen(getter_with_clone)]
    pub cels: Vec<ViewCel>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[wasm_bindgen]
pub struct AGIView {
    #[wasm_bindgen(getter_with_clone)]
    pub description: Option<String>,
    #[wasm_bindgen(getter_with_clone)]
    pub loops: Vec<ViewLoop>,
}

impl AGIView {
    pub fn get_cel(&self, loop_number: u8, cel_number: u8) -> Option<ViewCelHandle> {
        let cel = self
            .loops
            .get(loop_number as usize)
            .and_then(|loop_| loop_.cels.get(cel_number as usize))?;

        match &cel.data {
            ViewCelData::NonMirrored(non_mirrored_view_cel_data) => Some(ViewCelHandle {
                loop_number,
                cel_number: cel.cel_number,
                width: cel.width,
                height: cel.height,
                transparent_color: cel.transparent_color,
                mirrored_from_loop_number: None,
                data: &non_mirrored_view_cel_data.data,
            }),
            ViewCelData::Mirrored(mirrored_view_cel_data) => {
                let non_mirrored_cel =
                    self.get_cel(mirrored_view_cel_data.loop_number, cel_number)?;

                Some(ViewCelHandle {
                    loop_number,
                    cel_number: cel.cel_number,
                    width: cel.width,
                    height: cel.height,
                    transparent_color: cel.transparent_color,
                    mirrored_from_loop_number: Some(mirrored_view_cel_data.loop_number),
                    data: non_mirrored_cel.data,
                })
            }
        }
    }

    pub fn render_cel(
        &self,
        loop_number: u8,
        cel_number: u8,
        color_palette: &ColorPalette,
    ) -> Option<Vec<u8>> {
        let cel = self.get_cel(loop_number, cel_number)?;

        Some(render_view_cel(
            ViewCelPixelsIterator::new(&cel.data, cel.is_mirrored(), cel.width, cel.height),
            cel.transparent_color,
            color_palette,
        ))
    }
}

#[wasm_bindgen(js_name = "readViewResource")]
pub fn read_view_resource(data: Buffer) -> Result<AGIView, JsValue> {
    let data_array = Uint8Array::new(&data);
    let data_vec = data_array.to_vec();
    AGIView::decode(&mut Cursor::new(data_vec), ())
        .map_err(|e| JsValue::from_str(format!("{:?}", e).as_str()))
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::{
        resources::{encode::Encode, file_provider::FileProvider},
        test_data::contest2_template_dir,
    };

    use super::*;
    use similar_asserts::assert_eq;

    #[test]
    fn smoke_test() {
        let view_data = contest2_template_dir()
            .read_file_bytes("1.agiview")
            .unwrap();
        let view = AGIView::decode_from_bytes(&view_data, ()).unwrap();
        assert_eq!(view.loops.len(), 4);
        assert_eq!(view.loops[0].cels.len(), 8);
        for cel in &view.loops[0].cels {
            assert!(!cel.is_mirrored())
        }
        for cel in &view.loops[1].cels {
            assert!(cel.is_mirrored())
        }

        for loop_ in &view.loops {
            for cel in &loop_.cels {
                match cel.data {
                    ViewCelData::NonMirrored(ref non_mirrored_data) => {
                        assert_eq!(
                            non_mirrored_data.data.len(),
                            cel.width as usize * cel.height as usize
                        );
                    }
                    ViewCelData::Mirrored(ref mirrored_data) => {
                        assert!(mirrored_data.loop_number < view.loops.len() as u8);
                    }
                }
            }
        }

        let encoded = view.encode_to_vec(()).unwrap();
        let redecoded = AGIView::decode(&mut Cursor::new(encoded.clone()), ()).unwrap();
        assert_eq!(view, redecoded);
        assert_eq!(view_data[0..30], encoded.as_slice()[0..30]);
    }
}
