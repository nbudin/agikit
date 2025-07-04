use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::logic::{
    asm::expressions::AsmLogicArgument,
    commands::{AGICommand, TestCommand},
};

pub mod analysis;
pub mod asm;
pub mod commands;
pub mod decode;
pub mod encode;
pub mod logic_script;

#[derive(Debug, Clone, PartialEq, Eq, Tsify, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct LogicCommand {
    pub address: u16,
    pub agi_command: AGICommand,
    pub args: Vec<u8>,
}

impl LogicCommand {
    pub fn args(&self) -> Vec<AsmLogicArgument> {
        self.args
            .iter()
            .zip(self.agi_command.arg_types.iter())
            .map(|(&arg, arg_type)| AsmLogicArgument {
                value: arg as u16,
                arg_type: *arg_type,
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Tsify, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct LogicTest {
    pub test_command: TestCommand,
    pub args: Vec<u16>,
    pub negate: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Tsify, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct LogicOr {
    pub or_tests: Vec<LogicTest>,
}

#[derive(Debug, Clone, PartialEq, Eq, Tsify, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum LogicConditionClause {
    Test(LogicTest),
    Or(LogicOr),
}

#[derive(Debug, Clone, PartialEq, Eq, Tsify, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct LogicCondition {
    pub address: u16,
    pub clauses: Vec<LogicConditionClause>,
    pub skip_address: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Tsify, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct LogicGoto {
    pub address: u16,
    pub jump_address: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Tsify, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum LogicInstruction {
    Command(LogicCommand),
    Condition(LogicCondition),
    Goto(LogicGoto),
}

impl LogicInstruction {
    pub fn address(&self) -> u16 {
        match self {
            LogicInstruction::Command(cmd) => cmd.address,
            LogicInstruction::Condition(cond) => cond.address,
            LogicInstruction::Goto(goto) => goto.address,
        }
    }
}

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export type LogicMessages = {
  [key: number]: string;
};
"#;

pub type LogicMessages = HashMap<u8, String>;

#[derive(Debug, Clone, PartialEq, Eq, Tsify, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct LogicProgram {
    pub instructions: Vec<LogicInstruction>,
    pub messages: LogicMessages,
}

#[cfg(test)]
mod tests {
    use crate::{
        agi_version::AGIVersion,
        resources::{decode::Decode, encode::Encode, ResourceType},
        test_data::uriquest_resources,
    };

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn smoke_test() {
        let collection = uriquest_resources();
        let logic_data = collection
            .read_resource_data(ResourceType::LOGIC, 0)
            .expect("Failed to read logic resource 0");
        let logic_program = LogicProgram::decode_from_bytes(&logic_data, &AGIVersion::new(2, 917))
            .expect("Failed to decode logic program");

        let reencoded = logic_program
            .encode_to_vec(true)
            .expect("Failed to encode logic program");
        assert_eq!(
            reencoded
                .iter()
                .enumerate()
                .map(|(i, byte)| format!("{:04x}: {:02x}", i, byte))
                .collect::<Vec<_>>(),
            logic_data
                .iter()
                .enumerate()
                .map(|(i, byte)| format!("{:04x}: {:02x}", i, byte))
                .collect::<Vec<_>>(),
            "Re-encoded logic data does not match original"
        );
    }
}
