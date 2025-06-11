use std::io::{Cursor, Seek, SeekFrom};

use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
use web_sys::js_sys::Uint8Array;

use crate::{
    data_encoding::ReadHeterogeneousData,
    object_list::{ObjectList, ObjectListEntry},
    resources::decode::{Decode, DecodingError},
    wasm_utils::Buffer,
    xor_encryption::{XorCursor, AGI_ENCRYPTION_KEY},
};

impl<Data: ReadHeterogeneousData> Decode<'_, Data> for ObjectList
where
    for<'a> XorCursor<'a, Data>: ReadHeterogeneousData,
{
    type Options = ();

    fn decode<'a>(data: &'a mut Data, _options: Self::Options) -> Result<Self, DecodingError>
    where
        Self: Sized,
    {
        let mut data = XorCursor::new(data, AGI_ENCRYPTION_KEY.as_bytes());
        let object_names_offset = data.read_u16_le()? + 3;
        let max_animated_objects = data.read_u8()?;

        let mut name_offsets_and_starting_room_numbers: Vec<(u16, u8)> = Vec::new();
        while data.stream_position()? < object_names_offset as u64 {
            let name_offset = data.read_u16_le()? + 3;
            let starting_room_number = data.read_u8()?;
            name_offsets_and_starting_room_numbers.push((name_offset, starting_room_number));
        }

        let mut objects: Vec<ObjectListEntry> = Vec::new();
        for (name_offset, starting_room_number) in name_offsets_and_starting_room_numbers {
            data.seek(SeekFrom::Start(name_offset as u64))?;
            let name = data.read_null_terminated_string()?;

            let object = ObjectListEntry {
                name,
                starting_room_number,
            };
            objects.push(object);
        }

        Ok(ObjectList {
            objects,
            max_animated_objects,
        })
    }
}

#[wasm_bindgen(js_name = "readObjectList")]
pub fn read_object_list(data: Buffer) -> Result<ObjectList, JsValue> {
    let data_array = Uint8Array::new(&data);
    let data_vec = data_array.to_vec();
    ObjectList::decode(&mut Cursor::new(data_vec), ())
        .map_err(|e| JsValue::from_str(format!("{:?}", e).as_str()))
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::{object_list::ObjectList, resources::decode::Decode, TEST_DATA_DIR};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_decode_object() {
        let object_data = TEST_DATA_DIR
            .get_file("uriquest/OBJECT")
            .unwrap()
            .contents();
        let object_json_data = TEST_DATA_DIR
            .get_file("uriquest/object.json")
            .unwrap()
            .contents_utf8()
            .unwrap();

        let object_list = ObjectList::decode(&mut Cursor::new(object_data), ()).unwrap();
        let json_object_list = serde_json::from_str::<ObjectList>(object_json_data)
            .expect("Failed to deserialize OBJECT JSON");
        assert_eq!(object_list, json_object_list);
    }
}
