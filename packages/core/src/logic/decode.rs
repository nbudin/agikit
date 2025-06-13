use std::{
    collections::HashMap,
    io::{Seek, SeekFrom},
};

use crate::{
    agi_version::{AGIMajorVersion, AGIVersion},
    data_encoding::ReadHeterogeneousData,
    logic::{LogicMessages, LogicProgram},
    resources::decode::{Decode, DecodingError},
    xor_encryption::{XorCursor, AGI_ENCRYPTION_KEY},
};

impl<'opt, Data: ReadHeterogeneousData> Decode<'opt, Data> for LogicMessages {
    type Options = (&'opt AGIVersion, u16);

    fn decode<'a>(
        data: &'a mut Data,
        (agi_version, text_offset): Self::Options,
    ) -> Result<Self, DecodingError>
    where
        Self: Sized,
    {
        let mut messages = LogicMessages::new();
        data.seek(SeekFrom::Start(text_offset as u64))?;

        let message_count = data.read_u8()?;
        let _end_of_messages = data.read_u16_le()?;

        let message_offsets: HashMap<u8, u16> = (0..message_count)
            .map(|i| {
                let message_offset = data.read_u16_le()?;
                Ok((i, message_offset))
            })
            .collect::<Result<_, DecodingError>>()?;

        let xor_offset = data.stream_position()? as usize;
        let mut xor_cursor = XorCursor::new(data, AGI_ENCRYPTION_KEY.as_bytes(), xor_offset);
        let mut message_section_reader = match agi_version.major {
            AGIMajorVersion::AGI2 => Box::new(&mut xor_cursor as &mut dyn ReadHeterogeneousData),
            AGIMajorVersion::AGI3 => Box::new(data as &mut dyn ReadHeterogeneousData),
        };

        for message_index in 0..message_count {
            let message_offset = message_offsets.get(&message_index).copied().unwrap_or(0);
            if message_offset == 0 {
                continue; // Skip empty messages
            }

            message_section_reader.seek(SeekFrom::Start(
                text_offset as u64 + message_offset as u64 + 1,
            ))?;
            let message = message_section_reader.read_null_terminated_string()?;
            messages.insert(message_index, message);
        }

        Ok(messages)
    }
}

impl<'opt, Data: ReadHeterogeneousData> Decode<'opt, Data> for LogicProgram {
    type Options = &'opt AGIVersion;

    fn decode<'a>(data: &'a mut Data, agi_version: Self::Options) -> Result<Self, DecodingError>
    where
        Self: Sized,
    {
        let text_offset = data.read_u16_le()?;
        let messages = LogicMessages::decode(data, (agi_version, text_offset + 2))?;

        let program = LogicProgram {
            messages,
            instructions: Vec::new(), // Placeholder for now, as the actual decoding logic is not implemented
        };

        Ok(program)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;
    use crate::{
        agi_version::AGIVersion,
        resources::{
            dirs::{ResourceDirDecodeOptions, ResourceDirs},
            resource_collection::{ResourceCollection, ResourceCollectionVersionData},
            ResourceType,
        },
        TEST_DATA_DIR,
    };

    #[test]
    fn test_decode_messages() {
        let file_provider = TEST_DATA_DIR.get_dir("uriquest").unwrap();
        let dirs = ResourceDirs::read(ResourceDirDecodeOptions::AGI2 { file_provider }).unwrap();

        let collection = ResourceCollection::new(
            ResourceCollectionVersionData::AGI2,
            file_provider.clone(),
            dirs,
        );
        let logic_data = collection
            .read_resource_data(ResourceType::LOGIC, 0)
            .expect("Failed to read logic resource 0");
        let mut cursor = Cursor::new(logic_data);
        let logic_program = LogicProgram::decode(&mut cursor, &AGIVersion::new(2, 917))
            .expect("Failed to decode logic program");

        assert_eq!(logic_program.messages.len(), 45);
        assert_eq!(logic_program.messages.get(&0).unwrap(), "AGI");
    }
}
