use std::io::{Seek, SeekFrom};

use crate::{
    data_encoding::ReadHeterogeneousData,
    object_list::{ObjectList, ObjectListEntry},
    resource::{Decode, DecodingError},
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

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::{object_list::ObjectList, resource::Decode};
    use pretty_assertions::assert_eq;

    const OBJECT: &[u8] = include_bytes!("../../test_data/OBJECT");
    const OBJECT_JSON: &str = include_str!("../../test_data/object.json");

    #[test]
    fn test_decode_object() {
        let object_list = ObjectList::decode(&mut Cursor::new(OBJECT), ()).unwrap();
        let json_object_list = serde_json::from_str::<ObjectList>(OBJECT_JSON)
            .expect("Failed to deserialize OBJECT JSON");
        assert_eq!(object_list, json_object_list);
    }
}
