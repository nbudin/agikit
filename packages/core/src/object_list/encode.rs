use std::io::Cursor;

use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
use web_sys::js_sys::Uint8Array;

use crate::{
    buffer::Buffer,
    data_encoding::WriteHeterogeneousData,
    object_list::ObjectList,
    resources::encode::{Encode, EncodingError},
    xor_encryption::{XorCursor, AGI_ENCRYPTION_KEY},
};

impl Encode for ObjectList {
    type Options = ();

    fn encode(&self, _options: Self::Options) -> Result<Vec<u8>, EncodingError> {
        let mut data = Vec::new();
        let mut raw_cursor = Cursor::new(&mut data);
        let mut cursor = XorCursor::new(&mut raw_cursor, AGI_ENCRYPTION_KEY.as_bytes(), 0);

        let (name_offsets, _) =
            self.objects
                .iter()
                .fold((vec![], 0), |(mut offsets, offset), entry| {
                    offsets.push(offset);
                    (offsets, offset + entry.name.len() + 1)
                });

        let header_len = name_offsets.len() * 3;
        cursor.write_u16_le(header_len as u16)?;
        cursor.write_u8(self.max_animated_objects)?;
        for (offset, entry) in name_offsets.iter().zip(self.objects.iter()) {
            let name_offset = *offset + header_len;
            cursor.write_u16_le(name_offset as u16)?;
            cursor.write_u8(entry.starting_room_number)?;
        }

        for entry in &self.objects {
            cursor.write_null_terminated_string(&entry.name)?;
        }

        Ok(data)
    }
}

#[wasm_bindgen(js_name = "buildObjectList")]
pub fn build_object_list(object_list: &ObjectList) -> Result<Buffer, JsValue> {
    let encoded = object_list
        .encode(())
        .map_err(|e| JsValue::from_str(format!("{:?}", e).as_str()))?;
    let array = Uint8Array::from(encoded.as_slice());
    let buffer = Buffer::from(array.buffer());
    Ok(buffer)
}
