use peg::{error::ParseError, str::LineCol};
use petgraph::graph::NodeIndex;

use crate::logic::logic_script::{
    codegen::errors::LogicScriptCodeGenerationError,
    compile::{ast_generator::ASTGenerationError, diagnostics::LogicScriptDiagnostic},
    identifiers::DefineError,
};

pub enum CompilationError {
    ASTGenerationError(ASTGenerationError),
    BlockHasNotBeenCompiled(NodeIndex),
    CannotFindAddressForEmptyBlock(NodeIndex),
    CannotFindNextBlockAfterIf(NodeIndex),
    ConflictingInstructionForAddress(u16),
    LogicScriptCodeGenerationError(LogicScriptCodeGenerationError),
    DefineError(DefineError),
    FailedDiagnostics(Vec<LogicScriptDiagnostic>),
    ParseError(ParseError<LineCol>),
    IoError(std::io::Error),
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
