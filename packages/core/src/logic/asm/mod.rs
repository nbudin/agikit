use crate::logic::LogicInstruction;

pub mod codegen;

#[derive(Debug)]
pub struct LogicLabel<'a> {
    pub address: u16,
    pub label: String,
    pub references: Vec<&'a LogicInstruction>,
}
