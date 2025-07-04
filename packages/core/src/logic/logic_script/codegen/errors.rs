use std::fmt::Display;

use petgraph::graph::NodeIndex;

use crate::logic::{
    analysis::{
        ast::DecompilationError,
        basic_block_graph::{BasicBlock, BasicBlockEdgeType},
    },
    asm::{codegen::AsmCodeGenerationError, expressions::ParsedLogicArgument},
    logic_script::statements::LogicScriptStatement,
};

#[derive(Debug)]
pub enum LogicScriptCodeGenerationError {
    AsmCodeGenerationError(AsmCodeGenerationError),
    SerdeJsonError(serde_json::Error),
    DecompilationError(DecompilationError),
    BlockNotFound(NodeIndex),
    StatementGraphNodeNotFound(NodeIndex),
    JumpToUnlabeledStatement(NodeIndex, Option<BasicBlock>),
    ConditionalToUnlabeledBlock(NodeIndex, Option<BasicBlock>),
    MalformedBasicBlockEdgeTypes(NodeIndex, Option<BasicBlock>, Vec<BasicBlockEdgeType>),
    UnexpectedArgument(ParsedLogicArgument),
    GotoWithNoTarget(LogicScriptStatement<ParsedLogicArgument>),
}

fn describe_block(block_id: &NodeIndex, block: &Option<BasicBlock>) -> String {
    if let Some(block) = block {
        format!("block ID {}: {:?}", block_id.index(), block)
    } else {
        format!("block ID {}", block_id.index())
    }
}

impl Display for LogicScriptCodeGenerationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogicScriptCodeGenerationError::AsmCodeGenerationError(e) => e.fmt(f),
            LogicScriptCodeGenerationError::SerdeJsonError(e) => e.fmt(f),
            LogicScriptCodeGenerationError::DecompilationError(e) => e.fmt(f),
            LogicScriptCodeGenerationError::BlockNotFound(block_id) => {
                write!(f, "Block not found with ID: {}", block_id.index())
            }
            LogicScriptCodeGenerationError::StatementGraphNodeNotFound(node_id) => {
                write!(
                    f,
                    "Statement graph node not found with ID: {}",
                    node_id.index()
                )
            }
            LogicScriptCodeGenerationError::UnexpectedArgument(arg) => {
                write!(f, "Unexpected argument: {:?}", arg)
            }
            LogicScriptCodeGenerationError::JumpToUnlabeledStatement(block_id, block) => {
                write!(
                    f,
                    "Jump to unlabeled statement with {}",
                    describe_block(block_id, block)
                )
            }
            LogicScriptCodeGenerationError::ConditionalToUnlabeledBlock(block_id, block) => {
                write!(
                    f,
                    "Conditional branch to unlabeled {}",
                    describe_block(block_id, block)
                )
            }
            LogicScriptCodeGenerationError::MalformedBasicBlockEdgeTypes(
                block_id,
                block,
                edge_types,
            ) => {
                write!(
                    f,
                    "Malformed basic block edge types for {} with edge types: {:?}",
                    describe_block(block_id, block),
                    edge_types
                )
            }
            LogicScriptCodeGenerationError::GotoWithNoTarget(statement) => {
                write!(
                    f,
                    "Goto node with label {:?} has no target",
                    statement.get_goto_target_label()
                )
            }
        }
    }
}

impl From<AsmCodeGenerationError> for LogicScriptCodeGenerationError {
    fn from(error: AsmCodeGenerationError) -> Self {
        LogicScriptCodeGenerationError::AsmCodeGenerationError(error)
    }
}

impl From<serde_json::Error> for LogicScriptCodeGenerationError {
    fn from(error: serde_json::Error) -> Self {
        LogicScriptCodeGenerationError::SerdeJsonError(error)
    }
}

impl From<DecompilationError> for LogicScriptCodeGenerationError {
    fn from(error: DecompilationError) -> Self {
        LogicScriptCodeGenerationError::DecompilationError(error)
    }
}
