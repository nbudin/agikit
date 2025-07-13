use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use petgraph::{graph::NodeIndex, visit::Dfs};

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
        logic_script::codegen::errors::LogicScriptCodeGenerationError,
    },
    word_list::WordList,
};

pub struct LogicScriptLabelMap {
    labels_by_block_id: HashMap<NodeIndex, String>,
    block_id_by_label: HashMap<String, NodeIndex>,
    generated_label_counter: u16,
}

impl LogicScriptLabelMap {
    pub fn new(graph: &BasicBlockGraph) -> Self {
        let mut labels_by_block_id = HashMap::new();
        let mut block_id_by_label = HashMap::new();

        let mut dfs = Dfs::new(&graph.graph, graph.root_block_id);
        while let Some(block_id) = dfs.next(&graph.graph) {
            let Some(block) = graph.graph.node_weight(block_id) else {
                continue;
            };

            let Some(label) = block.label() else {
                continue;
            };

            labels_by_block_id.insert(block_id, label.to_string());
            block_id_by_label.insert(label.to_string(), block_id);
        }

        Self {
            labels_by_block_id,
            block_id_by_label,
            generated_label_counter: 0,
        }
    }

    pub fn label_for_block_id(&mut self, block_id: NodeIndex) -> String {
        self.labels_by_block_id
            .entry(block_id)
            .or_insert_with(|| {
                let label = loop {
                    let label = format!("GeneratedLabel{}", self.generated_label_counter);
                    self.generated_label_counter += 1;
                    if !self.block_id_by_label.contains_key(&label) {
                        break label;
                    }
                };

                self.block_id_by_label.insert(label.clone(), block_id);

                label
            })
            .clone()
    }
}

pub struct LogicScriptCodeGenerationContext<'a> {
    pub asm_context: AsmCodeGenerationContext<'a>,
    pub basic_block_graph: BasicBlockGraph,
    pub block_labels: Arc<Mutex<LogicScriptLabelMap>>,
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

        let label_map = LogicScriptLabelMap::new(&basic_block_graph);

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
            .label_for_block_id(block_id)
    }
}
