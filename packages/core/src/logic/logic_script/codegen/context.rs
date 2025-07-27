use std::sync::{Arc, Mutex};

use petgraph::graph::NodeIndex;

use crate::{
    logic::{
        LogicProgram,
        analysis::{
            ast::LogicAST,
            basic_block_graph::{BasicBlock, BasicBlockGraph},
            dominator_tree::DominationAnalysis,
            optimization::Optimizable,
        },
        asm::codegen::AsmCodeGenerationContext,
        logic_script::codegen::{
            errors::LogicScriptCodeGenerationError, node_label_map::NodeLabelMap,
        },
    },
    word_list::WordList,
};

pub struct LogicScriptCodeGenerationContext<'a> {
    pub asm_context: AsmCodeGenerationContext<'a>,
    pub basic_block_graph: BasicBlockGraph,
    pub block_labels: Arc<Mutex<NodeLabelMap>>,
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

        let label_map = NodeLabelMap::new(&basic_block_graph.graph, basic_block_graph.root_id());

        Self {
            asm_context,
            basic_block_graph,
            domination_analysis,
            block_labels: Arc::new(Mutex::new(label_map)),
        }
    }

    pub fn get_block(&self, block_id: NodeIndex) -> Option<&BasicBlock> {
        self.basic_block_graph.graph.node_weight(block_id)
    }

    pub fn label_for_block_id(&self, block_id: NodeIndex) -> String {
        self.block_labels
            .lock()
            .unwrap()
            .get_or_insert_label_for_node_id(block_id)
            .label()
            .to_string()
    }
}
