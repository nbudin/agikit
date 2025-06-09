use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::logic::commands::{AGICommand, TestCommand};

pub mod commands;
pub mod decode;

#[derive(Debug, Clone, PartialEq, Eq, Tsify, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct LogicCommand {
    pub address: u16,
    pub agi_command: AGICommand,
    pub args: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Tsify, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct LogicTest {
    pub test_command: TestCommand,
    pub args: Vec<u8>,
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
