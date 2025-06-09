use std::io::{Seek, SeekFrom};

use crate::{
    agi_version::{AGIMajorVersion, AGIVersion},
    data_encoding::ReadHeterogeneousData,
    logic::{LogicMessages, LogicProgram},
    resource::{Decode, DecodingError},
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
        let message_header_length = 3 + message_count as usize * 2;
        data.seek(SeekFrom::Start(
            text_offset as u64 + message_header_length as u64,
        ))?;
        let mut xor_cursor = XorCursor::new(data, AGI_ENCRYPTION_KEY.as_bytes());
        let mut message_section_reader = match agi_version.major {
            AGIMajorVersion::AGI2 => Box::new(&mut xor_cursor as &mut dyn ReadHeterogeneousData),
            AGIMajorVersion::AGI3 => Box::new(data as &mut dyn ReadHeterogeneousData),
        };

        for message_index in 0..message_count {
            message_section_reader.seek(SeekFrom::Start(
                text_offset as u64 + 3 + message_index as u64 * 2,
            ))?;
            let message_offset = message_section_reader.read_u16_le()?;
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
        let messages = LogicMessages::decode(data, (agi_version, text_offset))?;

        let program = LogicProgram {
            messages,
            instructions: Vec::new(), // Placeholder for now, as the actual decoding logic is not implemented
        };

        Ok(program)
    }
}
