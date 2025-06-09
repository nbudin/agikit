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
        resources::{decode::Decode, encode::Encode},
        xor_encryption::{XorCursor, AGI_ENCRYPTION_KEY},
    };
    use pretty_assertions::assert_eq;

    const OBJECT: &[u8] = include_bytes!("../../test_data/OBJECT");
    const OBJECT_JSON: &str = include_str!("../../test_data/object.json");

    #[test]
    fn test_object_json() {
        let object_list = serde_json::from_str::<ObjectList>(OBJECT_JSON)
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
        let object_list = ObjectList::decode(&mut Cursor::new(OBJECT), ()).unwrap();
        let encoded = object_list.encode(()).unwrap();

        let mut object_list_decrypted: Vec<u8> = Vec::with_capacity(OBJECT.len());
        XorCursor::new(&mut Cursor::new(OBJECT), AGI_ENCRYPTION_KEY.as_bytes())
            .read_to_end(&mut object_list_decrypted)
            .unwrap();

        let mut encoded_decrypted: Vec<u8> = Vec::with_capacity(encoded.len());
        XorCursor::new(&mut Cursor::new(&encoded), AGI_ENCRYPTION_KEY.as_bytes())
            .read_to_end(&mut encoded_decrypted)
            .unwrap();

        assert_eq!(object_list_decrypted, encoded_decrypted);
        assert_eq!(OBJECT, encoded);
    }
}
