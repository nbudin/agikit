use std::fmt::Display;

use peg::{error::ParseError, str::LineCol};
use petgraph::graph::NodeIndex;

use crate::logic::{
    asm::expressions::ParsedLogicArgument,
    logic_script::{
        codegen::errors::LogicScriptCodeGenerationError,
        compile::{ast_generator::ASTGenerationError, diagnostics::LogicScriptDiagnostic},
        identifiers::DefineError,
        locations::WithLocation,
    },
};

#[derive(Debug)]
pub enum CompilationError {
    ASTGenerationError(ASTGenerationError),
    BlockHasNotBeenCompiled(NodeIndex),
    CannotFindAddressForEmptyBlock(NodeIndex),
    CannotFindNextBlockAfterIf(NodeIndex),
    ConflictingInstructionForAddress(u16),
    LogicScriptCodeGenerationError(LogicScriptCodeGenerationError),
    DefineError(DefineError),
    FailedDiagnostics(Vec<LogicScriptDiagnostic<WithLocation<ParsedLogicArgument>>>),
    ParseError(ParseError<LineCol>),
    IoError(std::io::Error),
}

impl Display for CompilationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompilationError::ASTGenerationError(err) => err.fmt(f),
            CompilationError::BlockHasNotBeenCompiled(node_index) => f.write_fmt(format_args!(
                "Block at {:?} has not been compiled",
                node_index
            )),
            CompilationError::CannotFindAddressForEmptyBlock(node_index) => f.write_fmt(
                format_args!("Cannot find address for empty block at {:?}", node_index),
            ),
            CompilationError::CannotFindNextBlockAfterIf(node_index) => f.write_fmt(format_args!(
                "Cannot find next block after if statement at {:?}",
                node_index
            )),
            CompilationError::ConflictingInstructionForAddress(addr) => {
                f.write_fmt(format_args!("Conflicting instruction for address {}", addr))
            }
            CompilationError::LogicScriptCodeGenerationError(err) => err.fmt(f),
            CompilationError::DefineError(err) => err.fmt(f),
            CompilationError::FailedDiagnostics(diagnostics) => diagnostics
                .iter()
                .map(|diag| f.write_fmt(format_args!("{}", diag)))
                .collect(),
            CompilationError::ParseError(err) => err.fmt(f),
            CompilationError::IoError(err) => f.write_fmt(format_args!("IO Error: {}", err)),
        }
    }
}

impl From<DefineError> for CompilationError {
    fn from(value: DefineError) -> Self {
        Self::DefineError(value)
    }
}

impl From<ParseError<LineCol>> for CompilationError {
    fn from(value: ParseError<LineCol>) -> Self {
        Self::ParseError(value)
    }
}

impl From<std::io::Error> for CompilationError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl From<LogicScriptCodeGenerationError> for CompilationError {
    fn from(value: LogicScriptCodeGenerationError) -> Self {
        Self::LogicScriptCodeGenerationError(value)
    }
}

impl From<ASTGenerationError> for CompilationError {
    fn from(value: ASTGenerationError) -> Self {
        Self::ASTGenerationError(value)
    }
}
