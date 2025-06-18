use crate::logic::LogicInstruction;

pub mod codegen;
pub mod expressions;
pub mod literals;
pub mod operators;

#[derive(Debug)]
pub struct LogicLabel<'a> {
    pub address: u16,
    pub label: String,
    pub references: Vec<&'a LogicInstruction>,
}
