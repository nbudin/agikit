use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

pub mod decode;
pub mod encode;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen]
pub struct ObjectListEntry {
    #[wasm_bindgen(getter_with_clone)]
    pub name: String,
    #[wasm_bindgen(js_name = "startingRoomNumber")]
    pub starting_room_number: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen]
pub struct ObjectList {
    #[wasm_bindgen(getter_with_clone)]
    pub objects: Vec<ObjectListEntry>,
    #[wasm_bindgen(js_name = "maxAnimatedObjects")]
    pub max_animated_objects: u8,
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Read};

    use crate::{
        object_list::ObjectList,
        resources::{decode::Decode, encode::Encode, file_provider::FileProvider},
        test_data::uriquest_dir,
        xor_encryption::{XorCursor, AGI_ENCRYPTION_KEY},
    };
    use pretty_assertions::assert_eq;

    #[test]
    fn test_object_json() {
        let object_json_data = uriquest_dir()
            .read_file_utf8("object.json")
            .expect("Failed to read object.json as UTF-8");
        let object_list = serde_json::from_str::<ObjectList>(&object_json_data)
            .expect("Failed to deserialize OBJECT JSON");

        assert_eq!(object_list.max_animated_objects, 16);
        assert_eq!(object_list.objects.len(), 36);
        assert_eq!(object_list.objects[0].name, "?");
        assert_eq!(object_list.objects[0].starting_room_number, 0);
        assert_eq!(object_list.objects[2].name, "acoustic guitar");
        assert_eq!(object_list.objects[2].starting_room_number, 10);
    }

    #[test]
    fn smoke_test() {
        let object_data = uriquest_dir()
            .read_file_bytes("OBJECT")
            .expect("Failed to get OBJECT file");
        let object_list = ObjectList::decode_from_bytes(&object_data, ()).unwrap();
        let encoded = object_list.encode_to_vec(()).unwrap();

        let mut object_list_decrypted: Vec<u8> = Vec::with_capacity(object_data.len());
        XorCursor::new(
            &mut Cursor::new(object_data.clone()),
            AGI_ENCRYPTION_KEY.as_bytes(),
            0,
        )
        .read_to_end(&mut object_list_decrypted)
        .unwrap();

        let mut encoded_decrypted: Vec<u8> = Vec::with_capacity(encoded.len());
        XorCursor::new(&mut Cursor::new(&encoded), AGI_ENCRYPTION_KEY.as_bytes(), 0)
            .read_to_end(&mut encoded_decrypted)
            .unwrap();

        assert_eq!(object_list_decrypted, encoded_decrypted);
        assert_eq!(object_data, encoded);
    }
}
