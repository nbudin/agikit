use std::collections::HashMap;

use tsify::Tsify;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::logic::commands::{AGICommand, TestCommand};

pub mod commands;

#[derive(Debug, Clone, PartialEq, Eq, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct LogicCommand {
    pub address: u16,
    pub agi_command: AGICommand,
    pub args: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct LogicTest {
    pub test_command: TestCommand,
    pub args: Vec<u8>,
    pub negate: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct LogicOr {
    pub or_tests: Vec<LogicTest>,
}

#[derive(Debug, Clone, PartialEq, Eq, Tsify)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum LogicConditionClause {
    Test(LogicTest),
    Or(LogicOr),
}

#[derive(Debug, Clone, PartialEq, Eq, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct LogicCondition {
    pub address: u16,
    pub clauses: Vec<LogicConditionClause>,
    pub skip_address: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct LogicGoto {
    pub address: u16,
    pub jump_address: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Tsify)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum LogicInstruction {
    Command(LogicCommand),
    Condition(LogicCondition),
    Goto(LogicGoto),
}

#[derive(Debug, Clone, PartialEq, Eq, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct LogicProgram {
    pub instructions: Vec<LogicInstruction>,
    pub messages: HashMap<u8, String>,
}
