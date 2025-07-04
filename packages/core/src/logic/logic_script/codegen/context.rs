use std::collections::HashMap;

use petgraph::graph::NodeIndex;

use crate::{
    logic::{
        analysis::{
            ast::LogicAST,
            basic_block_graph::{BasicBlock, BasicBlockGraph},
            dominator_tree::DominationAnalysis,
            optimization::Optimizable,
        },
        asm::codegen::AsmCodeGenerationContext,
        logic_script::codegen::errors::LogicScriptCodeGenerationError,
        LogicProgram,
    },
    word_list::WordList,
};

pub struct LogicScriptCodeGenerationContext<'a> {
    pub asm_context: AsmCodeGenerationContext<'a>,
    pub basic_block_graph: BasicBlockGraph,
    pub block_labels: HashMap<NodeIndex, String>,
    pub domination_analysis: DominationAnalysis,
}

impl<'a> LogicScriptCodeGenerationContext<'a> {
    pub fn try_from_program(
        program: &'a LogicProgram,
        word_list: &'a WordList,
    ) -> Result<Self, LogicScriptCodeGenerationError> {
        let ast = LogicAST::from_instructions(&program.instructions)?;
        let mut basic_block_graph = BasicBlockGraph::from_ast(&ast);
        basic_block_graph.optimize();
        let asm_context = AsmCodeGenerationContext {
            logic: program,
            word_list: word_list,
        };
        Ok(Self::from_basic_block_graph(basic_block_graph, asm_context))
    }

    pub fn from_basic_block_graph(
        basic_block_graph: BasicBlockGraph,
        asm_context: AsmCodeGenerationContext<'a>,
    ) -> Self {
        let domination_analysis = DominationAnalysis::from_graph(
            &basic_block_graph.graph,
            basic_block_graph.root_block_id,
        );

        Self {
            asm_context,
            basic_block_graph,
            domination_analysis,
            block_labels: HashMap::new(),
        }
    }

    pub fn get_block(&self, block_id: NodeIndex) -> Option<&BasicBlock> {
        self.basic_block_graph.graph.node_weight(block_id)
    }
}
