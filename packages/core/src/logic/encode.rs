use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::Display,
    io::{Cursor, Write},
    rc::Rc,
};

use crate::{
    data_encoding::WriteHeterogeneousData,
    logic::{
        LogicCommand, LogicCondition, LogicConditionClause, LogicGoto, LogicInstruction,
        LogicMessages, LogicOr, LogicProgram, LogicTest,
    },
    resources::encode::{Encode, EncodingError},
    xor_encryption::{AGI_ENCRYPTION_KEY, XorCursor},
};

#[derive(Debug)]
pub enum AssemblyError {
    ByteCodeNotFound,
    ErrorAssemblingInstruction(LogicInstruction, Box<EncodingError>),
    InstructionAddressNotFound,
    InvalidJump(u16),
    InvalidConditionalSkip(u16),
    TargetAddressNotFound,
}

pub struct AddressPlaceholder {
    pub instruction: usize,
    pub jump_target: usize,
}

pub struct AssemblyState {
    pub instructions_by_id: HashMap<usize, LogicInstruction>,
    pub instructions_by_declared_address: HashMap<u16, usize>,
    pub address_placeholders: Vec<AddressPlaceholder>,
}

impl Display for AssemblyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssemblyError::ByteCodeNotFound => write!(f, "Bytecode not found"),
            AssemblyError::ErrorAssemblingInstruction(instruction, error) => {
                write!(
                    f,
                    "Error assembling instruction {:?}: {}",
                    instruction, error
                )
            }
            AssemblyError::InstructionAddressNotFound => {
                write!(f, "Instruction address not found")
            }
            AssemblyError::InvalidJump(address) => write!(f, "Invalid jump address to {}", address),
            AssemblyError::InvalidConditionalSkip(address) => {
                write!(f, "Invalid conditional skip to {}", address)
            }
            AssemblyError::TargetAddressNotFound => write!(f, "Target address not found"),
        }
    }
}

impl Encode<'_> for LogicMessages {
    type Options = bool;

    fn encode<Out: WriteHeterogeneousData>(
        &self,
        mut out: Out,
        encrypt: Self::Options,
    ) -> Result<(), EncodingError> {
        let max_message_id = self.keys().max().unwrap_or(&0);

        let message_buffers: Vec<Vec<u8>> = (0..=*max_message_id)
            .into_iter()
            .map(|id| {
                let message = self.get(&id);
                if let Some(message) = message {
                    format!("{}\0", message).as_bytes().to_vec()
                } else {
                    vec![0]
                }
            })
            .collect();
        let text_section: Vec<u8> = if encrypt {
            let mut encrypted_messages: Vec<u8> = vec![];
            let mut cursor = Cursor::new(&mut encrypted_messages);
            let mut xor_cursor = XorCursor::new(&mut cursor, AGI_ENCRYPTION_KEY.as_bytes(), 0);
            for buffer in &message_buffers {
                xor_cursor.write_all(buffer)?;
            }
            encrypted_messages
        } else {
            message_buffers.iter().flatten().copied().collect()
        };

        let message_header_length = 3 + message_buffers.len() * 2;
        let message_offsets = message_buffers
            .iter()
            .scan(message_header_length - 1, |offset, buffer| {
                let current_offset = *offset;
                *offset += buffer.len();
                if buffer.len() > 0 {
                    Some(current_offset)
                } else {
                    Some(0)
                }
            })
            .collect::<Vec<usize>>();

        out.write_u8(*max_message_id + 1)?;
        // Not sure why there's seemingly an off-by-one error in the format, but empirically there is
        out.write_u16_le(message_header_length as u16 + text_section.len() as u16 - 1)?;
        for offset in message_offsets {
            out.write_u16_le(offset as u16)?;
        }
        out.write_all(&text_section)?;
        Ok(())
    }
}

impl Encode<'_> for LogicTest {
    type Options = ();

    fn encode<Out: WriteHeterogeneousData>(
        &self,
        mut out: Out,
        _options: Self::Options,
    ) -> Result<(), EncodingError> {
        if self.negate {
            out.write_u8(0xfd)?;
        }
        out.write_u8(self.test_command.opcode)?;

        if self.test_command.var_args {
            out.write_u8(self.args.len() as u8)?;
            for arg in &self.args {
                out.write_u16_le(*arg)?;
            }
        } else {
            for arg in &self.args {
                out.write_u8(*arg as u8)?;
            }
        }

        Ok(())
    }
}

impl Encode<'_> for LogicOr {
    type Options = ();

    fn encode<Out: WriteHeterogeneousData>(
        &self,
        mut out: Out,
        options: Self::Options,
    ) -> Result<(), EncodingError> {
        out.write_u8(0xfc)?;
        for test in &self.or_tests {
            test.encode(&mut out, options)?;
        }
        out.write_u8(0xfc)?;
        Ok(())
    }
}

impl Encode<'_> for LogicConditionClause {
    type Options = ();

    fn encode<Out: WriteHeterogeneousData>(
        &self,
        out: Out,
        _options: Self::Options,
    ) -> Result<(), EncodingError> {
        match self {
            LogicConditionClause::Test(test) => test.encode(out, ()),
            LogicConditionClause::Or(or) => or.encode(out, ()),
        }
    }
}

impl Encode<'_> for LogicCommand {
    type Options = ();

    fn encode<Out: WriteHeterogeneousData>(
        &self,
        mut out: Out,
        _options: Self::Options,
    ) -> Result<(), EncodingError> {
        out.write_u8(self.agi_command.opcode)?;
        for arg in &self.args {
            out.write_u8(*arg)?;
        }
        Ok(())
    }
}

impl Encode<'_> for LogicCondition {
    type Options = (Rc<RefCell<AssemblyState>>, usize);

    fn encode<Out: WriteHeterogeneousData>(
        &self,
        mut out: Out,
        (state, instruction_id): Self::Options,
    ) -> Result<(), EncodingError> {
        let jump_target = {
            let read_state = state.borrow();
            read_state
                .instructions_by_declared_address
                .get(&self.skip_address)
                .copied()
        };
        let Some(jump_target) = jump_target else {
            return Err(AssemblyError::InvalidConditionalSkip(self.skip_address).into());
        };

        state
            .borrow_mut()
            .address_placeholders
            .push(AddressPlaceholder {
                instruction: instruction_id,
                jump_target: jump_target.clone(),
            });

        out.write_u8(0xff)?;
        for clause in self.clauses.iter() {
            clause.encode(&mut out, ())?;
        }
        out.write_all(&[0xff, 0x00, 0x00])?;
        Ok(())
    }
}

impl Encode<'_> for LogicGoto {
    type Options = (Rc<RefCell<AssemblyState>>, usize);

    fn encode<Out: WriteHeterogeneousData>(
        &self,
        mut out: Out,
        (state, instruction_id): Self::Options,
    ) -> Result<(), EncodingError> {
        let jump_target = {
            let read_state = state.borrow();
            read_state
                .instructions_by_declared_address
                .get(&self.jump_address)
                .copied()
        };
        let Some(jump_target) = jump_target else {
            return Err(AssemblyError::InvalidJump(self.jump_address).into());
        };

        state
            .borrow_mut()
            .address_placeholders
            .push(AddressPlaceholder {
                instruction: instruction_id,
                jump_target: jump_target.clone(),
            });

        out.write_all(&[0xfe, 0x00, 0x00])?;
        Ok(())
    }
}

impl Encode<'_> for LogicInstruction {
    type Options = (Rc<RefCell<AssemblyState>>, usize);

    fn encode<Out: WriteHeterogeneousData>(
        &self,
        out: Out,
        (state, instruction_id): Self::Options,
    ) -> Result<(), EncodingError> {
        match self {
            LogicInstruction::Command(logic_command) => logic_command.encode(out, ()),
            LogicInstruction::Condition(logic_condition) => {
                logic_condition.encode(out, (state, instruction_id))
            }
            LogicInstruction::Goto(logic_goto) => logic_goto.encode(out, (state, instruction_id)),
        }
    }
}

impl Encode<'_> for Vec<LogicInstruction> {
    type Options = ();

    fn encode<Out: WriteHeterogeneousData>(
        &self,
        mut out: Out,
        _options: Self::Options,
    ) -> Result<(), EncodingError> {
        let instructions_by_id = self
            .iter()
            .enumerate()
            .map(|(id, inst)| (id, inst.clone()))
            .collect();

        let state = Rc::new(RefCell::new(AssemblyState {
            instructions_by_id,
            instructions_by_declared_address: self
                .iter()
                .enumerate()
                .map(|(id, instruction)| (instruction.address(), id))
                .collect(),
            address_placeholders: Vec::new(),
        }));

        let mut byte_code_by_instruction: HashMap<usize, Vec<u8>> = HashMap::new();
        let mut instruction_addresses: HashMap<usize, u16> = HashMap::new();
        let mut address: u16 = 0;

        for (id, instruction) in self.iter().enumerate() {
            let encoded_instruction =
                instruction
                    .encode_to_vec((state.clone(), id))
                    .map_err(|err| {
                        AssemblyError::ErrorAssemblingInstruction(
                            instruction.clone(),
                            Box::new(err),
                        )
                    })?;
            let instruction_len = encoded_instruction.len() as u16;
            byte_code_by_instruction.insert(id, encoded_instruction);
            instruction_addresses.insert(id, address);
            address += instruction_len;
        }

        for placeholder in state.borrow().address_placeholders.iter() {
            let byte_code_len = byte_code_by_instruction
                .get(&placeholder.instruction)
                .ok_or_else(|| AssemblyError::ByteCodeNotFound)?
                .len();
            let instruction_address = instruction_addresses
                .get(&placeholder.instruction)
                .ok_or_else(|| AssemblyError::InstructionAddressNotFound)?;
            let target_address = instruction_addresses
                .get(&placeholder.jump_target)
                .ok_or_else(|| AssemblyError::TargetAddressNotFound)?;

            let offset = ((*target_address as i32)
                - ((*instruction_address as i32) + byte_code_len as i32))
                as i16;
            let mut offset_buffer = Vec::from([0u8; 2]);
            let mut offset_cursor = Cursor::new(&mut offset_buffer);
            offset_cursor.write_i16_le(offset)?;

            byte_code_by_instruction
                .get_mut(&placeholder.instruction)
                .ok_or_else(|| AssemblyError::ByteCodeNotFound)?
                .splice(byte_code_len - 2.., offset_buffer.iter().copied());
        }

        let instruction_byte_codes = self
            .iter()
            .enumerate()
            .map(|(id, _)| {
                byte_code_by_instruction
                    .get(&id)
                    .cloned()
                    .ok_or_else(|| AssemblyError::ByteCodeNotFound)
            })
            .collect::<Result<Vec<Vec<u8>>, _>>()?;

        for byte_code in instruction_byte_codes.iter() {
            out.write_all(byte_code)?;
        }
        Ok(())
    }
}

impl Encode<'_> for LogicProgram {
    type Options = bool;

    fn encode<Out: WriteHeterogeneousData>(
        &self,
        mut out: Out,
        encrypt_messages: Self::Options,
    ) -> Result<(), EncodingError> {
        let instructions = self.instructions.encode_to_vec(())?;
        out.write_u16_le(instructions.len() as u16)?;
        out.write_all(&instructions)?;
        self.messages.encode(&mut out, encrypt_messages)?;

        Ok(())
    }
}

#[cfg(feature = "js")]
pub mod js {
    use serde::{Deserialize, Serialize};
    use tsify::Tsify;
    use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

    use crate::{
        buffer::Buffer,
        logic::{LogicInstruction, LogicMessages, LogicProgram},
        resources::encode::Encode,
    };

    #[derive(Serialize, Deserialize, Tsify)]
    #[serde(transparent)]
    #[tsify(into_wasm_abi, from_wasm_abi)]
    pub struct MessageArray(pub Vec<Option<String>>);

    impl From<MessageArray> for LogicMessages {
        fn from(messages: MessageArray) -> Self {
            messages
                .0
                .into_iter()
                .enumerate()
                .filter_map(|(id, message)| message.map(|msg| (id as u8, msg)))
                .collect()
        }
    }

    #[wasm_bindgen(js_name = "encodeMessages")]
    pub fn js_encode_messages(messages: MessageArray, encrypt: bool) -> Result<Buffer, JsValue> {
        let messages: LogicMessages = messages.into();
        messages
            .encode_to_vec(encrypt)
            .map(|data| Buffer::from(data))
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "assembleLogic")]
    pub fn js_assemble_logic(
        instructions: Vec<LogicInstruction>,
        messages: MessageArray,
        encrypt_messages: bool,
    ) -> Result<Buffer, JsValue> {
        let logic_program = LogicProgram {
            instructions,
            messages: messages.into(),
        };
        logic_program
            .encode_to_vec(encrypt_messages)
            .map(|data| Buffer::from(data))
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}
