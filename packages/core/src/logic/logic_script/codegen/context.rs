use std::collections::HashMap;

use petgraph::graph::NodeIndex;

use crate::{
    logic::{
        asm::codegen::AsmCodeGenerationContext,
        logic_script::{
            ast::LogicAST,
            basic_block_graph::{BasicBlock, BasicBlockGraph},
            codegen::errors::LogicScriptCodeGenerationError,
            control_flow_analysis::ReverseCFG,
            dominator_tree::DominatorTree,
        },
        LogicProgram,
    },
    word_list::WordList,
};

pub struct LogicScriptCodeGenerationContext<'a> {
    pub asm_context: AsmCodeGenerationContext<'a>,
    pub basic_block_graph: BasicBlockGraph,
    pub dominator_tree: DominatorTree,
    pub reverse_cfg: ReverseCFG,
    pub post_dominator_tree: DominatorTree,
    pub block_labels: HashMap<NodeIndex, String>,
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
        let dominator_tree =
            DominatorTree::from_cfg(&basic_block_graph.graph, basic_block_graph.root_block_id);

        let reverse_cfg = ReverseCFG::from_basic_block_graph(&basic_block_graph);

        let post_dominator_tree =
            DominatorTree::from_cfg(&reverse_cfg.graph, reverse_cfg.virtual_root_id);

        Self {
            asm_context,
            basic_block_graph,
            dominator_tree,
            reverse_cfg,
            post_dominator_tree,
            block_labels: HashMap::new(),
        }
    }

    pub fn get_block(&self, block_id: NodeIndex) -> Option<&BasicBlock> {
        self.basic_block_graph.graph.node_weight(block_id)
    }

    pub fn dominates(&self, a: NodeIndex, b: NodeIndex) -> bool {
        self.dominator_tree.dominates(a, b)
    }

    pub fn post_dominates(&self, a: NodeIndex, b: NodeIndex) -> bool {
        self.post_dominator_tree.dominates(a, b)
    }
}
