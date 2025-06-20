use crate::{
    data_encoding::WriteHeterogeneousData,
    object_list::ObjectList,
    resources::encode::{Encode, EncodingError},
    xor_encryption::{XorCursor, AGI_ENCRYPTION_KEY},
};

impl Encode<'_> for ObjectList {
    type Options = ();

    fn encode<Out: WriteHeterogeneousData>(
        &self,
        mut out: Out,
        _options: Self::Options,
    ) -> Result<(), EncodingError> {
        let mut cursor = XorCursor::new(&mut out, AGI_ENCRYPTION_KEY.as_bytes(), 0);

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

        Ok(())
    }
}

#[cfg(feature = "js")]
pub mod js {
    use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

    use crate::{buffer::Buffer, object_list::ObjectList, resources::encode::Encode};

    #[wasm_bindgen(js_name = "buildObjectList")]
    pub fn build_object_list(object_list: &ObjectList) -> Result<Buffer, JsValue> {
        let output = object_list
            .encode_to_vec(())
            .map_err(|e| JsValue::from_str(format!("{:?}", e).as_str()))?;
        let buffer = Buffer::from(output);
        Ok(buffer)
    }
}
