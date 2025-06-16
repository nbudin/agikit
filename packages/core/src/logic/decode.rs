use std::{
    collections::HashMap,
    fmt::Display,
    io::{Cursor, Seek, SeekFrom},
};

use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

use crate::{
    agi_version::{AGIMajorVersion, AGIVersion},
    buffer::Buffer,
    data_encoding::ReadHeterogeneousData,
    logic::{
        commands::{AGICommand, TestCommand},
        LogicCommand, LogicCondition, LogicConditionClause, LogicGoto, LogicInstruction,
        LogicMessages, LogicOr, LogicProgram, LogicTest,
    },
    resources::decode::{Decode, DecodingError},
    xor_encryption::{XorCursor, AGI_ENCRYPTION_KEY},
};

#[derive(Debug)]
pub enum DisassemblyError {
    InvalidOpcode(u8),
    InvalidTestOpcode(u8),
}

impl Display for DisassemblyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DisassemblyError::InvalidOpcode(opcode) => {
                write!(f, "Invalid opcode: 0x{:02X}", opcode)
            }
            DisassemblyError::InvalidTestOpcode(opcode) => {
                write!(f, "Invalid test opcode: 0x{:02X}", opcode)
            }
        }
    }
}

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

impl<'opt, Data: ReadHeterogeneousData> Decode<'opt, Data> for LogicCondition {
    type Options = u16;

    fn decode<'a>(data: &'a mut Data, address: Self::Options) -> Result<Self, DecodingError>
    where
        Self: Sized,
    {
        let mut clauses = vec![];
        let mut negate_next = false;
        let mut disjunction: Option<LogicOr> = None;

        loop {
            let test_opcode = data.read_u8()?;
            match test_opcode {
                0xff => {
                    break;
                }
                0xfd => {
                    negate_next = true;
                }
                0xfc => {
                    if let Some(logic_or) = disjunction {
                        clauses.push(LogicConditionClause::Or(logic_or));
                        disjunction = None;
                    } else {
                        disjunction = Some(LogicOr {
                            or_tests: Vec::new(),
                        });
                    }
                }
                _ => {
                    let test_command = TestCommand::get(test_opcode)
                        .ok_or_else(|| DisassemblyError::InvalidTestOpcode(test_opcode))?;

                    let arg_count = if test_command.var_args {
                        data.read_u8()?
                    } else {
                        test_command.arg_types.len() as u8
                    };

                    let args = (0..arg_count)
                        .map(|_| data.read_u8())
                        .collect::<Result<Vec<_>, _>>()?;

                    let logic_test = LogicTest {
                        test_command: test_command.clone(),
                        args,
                        negate: negate_next,
                    };
                    negate_next = false;

                    if let Some(logic_or) = &mut disjunction {
                        logic_or.or_tests.push(logic_test);
                    } else {
                        clauses.push(LogicConditionClause::Test(logic_test));
                    }
                }
            }
        }

        let skip_offset = data.read_i16_le()?;

        let condition = LogicCondition {
            address,
            clauses,
            skip_address: (address as i32 + skip_offset as i32) as u16,
        };

        Ok(condition)
    }
}

impl<'opt, Data: ReadHeterogeneousData> Decode<'opt, Data> for LogicInstruction {
    type Options = &'opt AGIVersion;

    fn decode<'a>(data: &'a mut Data, agi_version: Self::Options) -> Result<Self, DecodingError>
    where
        Self: Sized,
    {
        let address = data.stream_position()? as u16 - 2; // Adjust for the text offset header
        let opcode = data.read_u8()?;
        match opcode {
            0xff => {
                let condition = LogicCondition::decode(data, address)?;
                Ok(LogicInstruction::Condition(condition))
            }
            0xfe => {
                let jump_offset = data.read_i16_le()?;
                Ok(LogicInstruction::Goto(LogicGoto {
                    address,
                    jump_address: (address as i32 + jump_offset as i32) as u16,
                }))
            }
            _ => {
                let agi_command = AGICommand::get(opcode, agi_version)
                    .ok_or_else(|| DisassemblyError::InvalidOpcode(opcode))?;

                let args = (0..(agi_command.arg_types.len()))
                    .map(|_| data.read_u8())
                    .collect::<Result<Vec<_>, _>>()?;

                let command = LogicCommand {
                    address,
                    agi_command: agi_command.clone(),
                    args,
                };

                Ok(LogicInstruction::Command(command))
            }
        }
    }
}

impl<'opt, Data: ReadHeterogeneousData> Decode<'opt, Data> for Vec<LogicInstruction> {
    type Options = (&'opt AGIVersion, u16);

    fn decode<'a>(
        data: &'a mut Data,
        (agi_version, text_offset): Self::Options,
    ) -> Result<Self, DecodingError>
    where
        Self: Sized,
    {
        let mut instructions = Vec::new();
        while data.stream_position()? < text_offset as u64 + 2 {
            let instruction = LogicInstruction::decode(data, agi_version)?;
            instructions.push(instruction);
        }
        Ok(instructions)
    }
}

impl<'opt, Data: ReadHeterogeneousData> Decode<'opt, Data> for LogicProgram {
    type Options = &'opt AGIVersion;

    fn decode<'a>(data: &'a mut Data, agi_version: Self::Options) -> Result<Self, DecodingError>
    where
        Self: Sized,
    {
        let text_offset = data.read_u16_le()? + 2; // the offset doesn't include its own length
        let instructions = Vec::<LogicInstruction>::decode(data, (agi_version, text_offset))?;
        let messages = LogicMessages::decode(data, (agi_version, text_offset))?;

        let program = LogicProgram {
            messages,
            instructions, // Placeholder for now, as the actual decoding logic is not implemented
        };

        Ok(program)
    }
}

#[wasm_bindgen(js_name = "readLogicResource")]
pub fn read_logic_resource_js(
    resource_data: Buffer,
    agi_version: AGIVersion,
) -> Result<LogicProgram, JsValue> {
    let data_vec = Vec::from(resource_data);
    let mut cursor = Cursor::new(data_vec);

    LogicProgram::decode(&mut cursor, &agi_version)
        .map_err(|e| JsValue::from_str(format!("{:?}", e).as_str()))
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
    fn test_decode() {
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
