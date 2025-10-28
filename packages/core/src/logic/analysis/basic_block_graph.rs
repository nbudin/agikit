use std::collections::HashMap;

use petgraph::{
    Direction,
    graph::{EdgeIndex, NodeIndex},
    prelude::StableDiGraph,
    visit::EdgeRef,
};

#[cfg(feature = "dot")]
use crate::logic::asm::codegen::AsmCodeGenerationContext;
use crate::logic::{
    LogicConditionClause,
    analysis::{
        ast::{LogicAST, LogicASTNode, LogicCommandNode},
        optimization::{
            DirectedNeighborEdgeUtils, Optimizable, OptimizationPass, OptimizationResult,
            RemoveNodePreservingEdges,
        },
    },
    logic_script::codegen::{errors::LogicScriptCodeGenerationError, node_label_map::LabeledNode},
};

pub trait BasicBlockVisitor {
    fn visit_basic_block(&mut self, graph: &mut BasicBlockGraph, block_id: NodeIndex) -> bool;
}

impl<F: FnMut(&mut BasicBlockGraph, NodeIndex) -> bool> BasicBlockVisitor for F {
    fn visit_basic_block(&mut self, graph: &mut BasicBlockGraph, block_id: NodeIndex) -> bool {
        self(graph, block_id)
    }
}

#[derive(Debug, Clone)]
pub struct SinglePathBasicBlock {
    pub label: Option<String>,
    pub commands: Vec<LogicCommandNode>,
}

#[derive(Debug, Clone)]
pub struct ConditionalBasicBlock {
    pub label: Option<String>,
    pub conditions: Vec<LogicConditionClause>,
}

#[derive(Debug, Clone)]
pub enum BasicBlock {
    SinglePath(SinglePathBasicBlock),
    Conditional(ConditionalBasicBlock),
}

impl LabeledNode for BasicBlock {
    fn label(&self) -> Option<&str> {
        match self {
            BasicBlock::SinglePath(block) => block.label.as_deref(),
            BasicBlock::Conditional(block) => block.label.as_deref(),
        }
    }

    fn set_label(&mut self, label: Option<&str>) {
        let label = label.map(|l| l.to_string());
        match self {
            BasicBlock::SinglePath(block) => block.label = label,
            BasicBlock::Conditional(block) => block.label = label,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BasicBlockEdgeType {
    Next,
    IfThen,
    IfElse,
}

pub enum BasicBlockControlFlow<'a> {
    SinglePath {
        block: &'a SinglePathBasicBlock,
        next_id: Option<NodeIndex>,
    },
    Conditional {
        block: &'a ConditionalBasicBlock,
        conditions: Vec<LogicConditionClause>,
        then_id: NodeIndex,
        else_id: Option<NodeIndex>,
    },
}

#[derive(Debug, Clone)]
pub struct BasicBlockGraph {
    pub graph: StableDiGraph<BasicBlock, BasicBlockEdgeType>,
    pub root_block_id: NodeIndex,
}

impl BasicBlockGraph {
    pub fn from_ast(ast: &LogicAST) -> Self {
        let mut graph = StableDiGraph::new();
        let mut block_ids_by_node_id = HashMap::new();
        let root_block_id =
            build_basic_blocks(ast, ast.root_node_id, &mut graph, &mut block_ids_by_node_id);

        BasicBlockGraph {
            graph,
            root_block_id,
        }
    }

    pub fn directed_neighbor_edge_id_of_type(
        &self,
        node_id: NodeIndex,
        direction: Direction,
        edge_type: BasicBlockEdgeType,
    ) -> Option<EdgeIndex> {
        self.graph
            .edges_directed(node_id, direction)
            .find_map(|edge| {
                if edge.weight() == &edge_type {
                    Some(edge.id())
                } else {
                    None
                }
            })
    }

    pub fn control_flow_for_block(
        &self,
        block_id: NodeIndex,
    ) -> Result<BasicBlockControlFlow<'_>, LogicScriptCodeGenerationError> {
        let block = self.graph.node_weight(block_id);

        match block {
            Some(BasicBlock::SinglePath(block)) => {
                let edges = self
                    .graph
                    .edges_directed(block_id, Direction::Outgoing)
                    .filter(|edge| *edge.weight() == BasicBlockEdgeType::Next)
                    .collect::<Vec<_>>();

                if edges.len() > 1 {
                    Err(
                        LogicScriptCodeGenerationError::MalformedBasicBlockEdgeTypes(
                            block_id,
                            Some(BasicBlock::SinglePath(block.clone())),
                            edges
                                .into_iter()
                                .map(|edge| edge.weight().clone())
                                .collect(),
                        ),
                    )
                } else {
                    Ok(BasicBlockControlFlow::SinglePath {
                        block,
                        next_id: edges.first().map(|edge| edge.target()),
                    })
                }
            }
            Some(BasicBlock::Conditional(block)) => {
                let if_then_edges = self
                    .graph
                    .edges_directed(block_id, Direction::Outgoing)
                    .filter(|edge| *edge.weight() == BasicBlockEdgeType::IfThen)
                    .collect::<Vec<_>>();

                let if_else_edges = self
                    .graph
                    .edges_directed(block_id, Direction::Outgoing)
                    .filter(|edge| *edge.weight() == BasicBlockEdgeType::IfElse)
                    .collect::<Vec<_>>();

                if if_then_edges.len() != 1 || if_else_edges.len() > 1 {
                    Err(
                        LogicScriptCodeGenerationError::MalformedBasicBlockEdgeTypes(
                            block_id,
                            Some(BasicBlock::Conditional(block.clone())),
                            if_then_edges
                                .into_iter()
                                .chain(if_else_edges.into_iter())
                                .map(|edge| edge.weight().clone())
                                .collect(),
                        ),
                    )
                } else {
                    Ok(BasicBlockControlFlow::Conditional {
                        block,
                        conditions: block.conditions.clone(),
                        then_id: if_then_edges.first().unwrap().target(),
                        else_id: if_else_edges.first().map(|edge| edge.target()),
                    })
                }
            }
            None => Err(LogicScriptCodeGenerationError::BlockNotFound(block_id)),
        }
    }
}

impl Optimizable<StableDiGraph<BasicBlock, BasicBlockEdgeType>> for BasicBlockGraph {
    fn get_graph(&self) -> &StableDiGraph<BasicBlock, BasicBlockEdgeType> {
        &self.graph
    }

    fn get_graph_mut(&mut self) -> &mut StableDiGraph<BasicBlock, BasicBlockEdgeType> {
        &mut self.graph
    }

    fn root_id(&self) -> NodeIndex {
        self.root_block_id
    }

    fn optimization_passes(
        &self,
    ) -> Vec<Box<dyn OptimizationPass<StableDiGraph<BasicBlock, BasicBlockEdgeType>>>> {
        vec![
            Box::new(remove_empty_block),
            Box::new(concatenate_linear_blocks),
        ]
    }
}

#[cfg(feature = "dot")]
impl BasicBlock {
    pub fn node_attrs(&self, asm_context: &AsmCodeGenerationContext, node_id: NodeIndex) -> String {
        use crate::logic::asm::{codegen::GenerateLogicAsm, expressions::LogicBooleanExpression};

        let (shape, node_desc) = match self {
            BasicBlock::SinglePath(block) => (
                "box",
                block
                    .commands
                    .iter()
                    .map(|c| c.command.generate_asm(asm_context, &HashMap::new()))
                    .collect::<Result<Vec<_>, _>>()
                    .expect("Error generating asm")
                    .join("\n"),
            ),
            BasicBlock::Conditional(block) => (
                "diamond",
                format!(
                    "if ({})",
                    LogicBooleanExpression::from_clauses(&block.conditions, asm_context)
                        .expect("Error generating conditions")
                        .generate_asm(asm_context, &HashMap::new())
                        .expect("Error generating asm"),
                ),
            ),
        };

        let label = format!(
            "{}\n{}",
            self.label()
                .map(|l| l.to_owned())
                .unwrap_or_else(|| format!("Node {}", node_id.index())),
            node_desc
        );
        format!(
            "shape = {}, label = {}",
            shape,
            serde_json::to_string(&label).unwrap()
        )
    }
}

#[cfg(feature = "dot")]
impl BasicBlockGraph {
    pub fn to_dot(&self, asm_context: &AsmCodeGenerationContext) -> String {
        use petgraph::dot::{Config, Dot};

        format!(
            "{:?}",
            Dot::with_attr_getters(
                &self.graph,
                &[Config::NodeNoLabel, Config::EdgeNoLabel],
                &|_graph_ref, edge_ref| match *edge_ref.weight() {
                    BasicBlockEdgeType::Next => "",
                    BasicBlockEdgeType::IfThen => "label = \"then\"",
                    BasicBlockEdgeType::IfElse => "label = \"else\"",
                }
                .to_string(),
                &|_graph_ref, (node_id, node_weight)| {
                    node_weight.node_attrs(asm_context, node_id)
                },
            )
        )
    }
}

fn build_basic_blocks(
    ast: &LogicAST,
    node_id: NodeIndex,
    graph: &mut StableDiGraph<BasicBlock, BasicBlockEdgeType>,
    block_ids_by_node_id: &mut HashMap<NodeIndex, NodeIndex>,
) -> NodeIndex {
    let find_or_build_blocks_for_node =
        |node_id: NodeIndex,
         graph: &mut StableDiGraph<BasicBlock, BasicBlockEdgeType>,
         block_ids_by_node_id: &mut HashMap<NodeIndex, NodeIndex>|
         -> NodeIndex {
            block_ids_by_node_id
                .get(&node_id)
                .copied()
                .unwrap_or_else(|| build_basic_blocks(ast, node_id, graph, block_ids_by_node_id))
        };

    let node = &ast
        .graph
        .node_weight(node_id.into())
        .expect("Node not found in AST graph");
    match node {
        LogicASTNode::Command(node) => {
            let block = BasicBlock::SinglePath(SinglePathBasicBlock {
                label: node.label.as_ref().map(|l| l.label.clone()),
                commands: vec![node.clone()],
            });
            let block_id = graph.add_node(block);
            block_ids_by_node_id.insert(node_id, block_id);

            if let Some(next_node_id) = ast.next_node_id(node_id) {
                let subsequent_block_id =
                    find_or_build_blocks_for_node(next_node_id, graph, block_ids_by_node_id);
                graph.add_edge(block_id, subsequent_block_id, BasicBlockEdgeType::Next);
            }

            block_id
        }
        LogicASTNode::If(node) => {
            let block = BasicBlock::Conditional(ConditionalBasicBlock {
                label: node.label.as_ref().map(|l| l.label.clone()),
                conditions: node.clauses.clone(),
            });
            let block_id = graph.add_node(block);
            block_ids_by_node_id.insert(node_id, block_id);

            if let Some(then_node_id) = ast.then_node_id(node_id) {
                let then_block_id =
                    find_or_build_blocks_for_node(then_node_id, graph, block_ids_by_node_id);
                graph.add_edge(block_id, then_block_id, BasicBlockEdgeType::IfThen);
            }

            if let Some(else_node_id) = ast.else_node_id(node_id) {
                let else_block_id =
                    find_or_build_blocks_for_node(else_node_id, graph, block_ids_by_node_id);
                graph.add_edge(block_id, else_block_id, BasicBlockEdgeType::IfElse);
            }

            block_id
        }
        LogicASTNode::Goto(node) => {
            let block = BasicBlock::SinglePath(SinglePathBasicBlock {
                commands: vec![],
                label: node.label.as_ref().map(|l| l.label.clone()),
            });
            let block_id = graph.add_node(block);
            block_ids_by_node_id.insert(node_id, block_id);

            if let Some(target_node_id) = ast.goto_target_node_id(node_id) {
                let target_block_id =
                    find_or_build_blocks_for_node(target_node_id, graph, block_ids_by_node_id);
                graph.add_edge(block_id, target_block_id, BasicBlockEdgeType::Next);
            }

            block_id
        }
    }
}

pub fn remove_empty_block(
    graph: &mut StableDiGraph<BasicBlock, BasicBlockEdgeType>,
    block_id: NodeIndex,
) -> OptimizationResult {
    let block = graph.node_weight(block_id);
    if let Some(BasicBlock::SinglePath(block)) = block {
        if block.commands.is_empty() {
            if let Some(next_block_id) = graph.directed_neighbor_node_id_of_type(
                block_id,
                Direction::Outgoing,
                BasicBlockEdgeType::Next,
            ) {
                let incoming_edge_data = graph.incoming_edge_data(block_id);

                let block_label = graph
                    .node_weight(block_id)
                    .and_then(|block| block.label().map(|l| l.to_owned()));

                let new_target_block = graph.node_weight_mut(next_block_id).unwrap();
                if new_target_block.label().is_none() && block_label.is_some() {
                    new_target_block.set_label(block_label.as_deref());
                }

                for (_, source_id, weight) in incoming_edge_data {
                    graph.update_edge(source_id, next_block_id, weight);
                }
                graph.remove_node(block_id);

                return OptimizationResult::Changed;
            }
        }
    }

    OptimizationResult::Unchanged
}

pub fn concatenate_linear_blocks(
    graph: &mut StableDiGraph<BasicBlock, BasicBlockEdgeType>,
    block_id: NodeIndex,
) -> OptimizationResult {
    let block = graph.node_weight(block_id);
    if let Some(BasicBlock::SinglePath(block)) = block {
        if block.label.is_some() {
            // this block might be referenced by a jump
            return OptimizationResult::Unchanged;
        }

        if let Some(prev_block_id) = graph.directed_neighbor_node_id_of_type(
            block_id,
            Direction::Incoming,
            BasicBlockEdgeType::Next,
        ) {
            let next_edge_id = graph.directed_neighbor_edge_id_of_type(
                block_id,
                Direction::Outgoing,
                BasicBlockEdgeType::Next,
            );
            let commands = block.commands.clone();
            let prev_block = graph.node_weight_mut(prev_block_id).unwrap();

            if let BasicBlock::SinglePath(prev_block) = prev_block {
                prev_block.commands.extend(commands);
                if let Some(next_edge_id) = next_edge_id {
                    let (_, next_block_id) = graph.edge_endpoints(next_edge_id).unwrap();
                    graph.add_edge(prev_block_id, next_block_id, BasicBlockEdgeType::Next);
                    graph.remove_edge(next_edge_id);
                }
            }

            graph.remove_node(block_id);
            return OptimizationResult::Changed;
        }
    }

    OptimizationResult::Unchanged
}

#[cfg(test)]
mod tests {
    use petgraph::{Direction, prelude::StableDiGraph};

    use crate::{
        agi_version::AGIVersion,
        logic::{
            LogicCommand, LogicProgram,
            analysis::{
                ast::{LogicAST, LogicASTNodeMetadata, LogicCommandNode},
                basic_block_graph::{
                    BasicBlock, BasicBlockEdgeType, BasicBlockGraph, SinglePathBasicBlock,
                    concatenate_linear_blocks, remove_empty_block,
                },
                optimization::{DirectedNeighborEdgeUtils, Optimizable},
            },
            asm::LogicLabel,
            commands::AGICommand,
        },
        resources::{ResourceType, decode::Decode},
        test_data::uriquest_resources,
    };

    fn build_increment_command_node(address: u16) -> LogicCommandNode {
        LogicCommandNode {
            command: LogicCommand {
                agi_command: AGICommand::by_name("increment", &AGIVersion::new(2, 917))
                    .unwrap()
                    .clone(),
                address,
                args: vec![address as u8],
            },
            label: Some(LogicLabel {
                label: format!("Address{}", address),
                address,
            }),
            metadata: LogicASTNodeMetadata {
                instruction_address: Some(address),
            },
        }
    }

    #[test]
    fn test_remove_empty_block() {
        let mut graph = StableDiGraph::<BasicBlock, BasicBlockEdgeType>::new();
        let block1_id = graph.add_node(BasicBlock::SinglePath(SinglePathBasicBlock {
            commands: vec![build_increment_command_node(1)],
            label: Some("Address1".to_string()),
        }));
        let block2_id = graph.add_node(BasicBlock::SinglePath(SinglePathBasicBlock {
            commands: vec![],
            label: None,
        }));
        let block3_id = graph.add_node(BasicBlock::SinglePath(SinglePathBasicBlock {
            commands: vec![build_increment_command_node(3)],
            label: Some("Address3".to_string()),
        }));
        graph.add_edge(block1_id, block2_id, BasicBlockEdgeType::Next);
        graph.add_edge(block2_id, block3_id, BasicBlockEdgeType::Next);

        assert!(
            remove_empty_block(&mut graph, block2_id).is_changed(),
            "Block was not removed"
        );

        assert_eq!(2, graph.node_count());
        let Some(BasicBlock::SinglePath(block1)) = graph.node_weight(block1_id) else {
            panic!(
                "Unexpected weight for {:?}: {:?}",
                block1_id,
                graph.node_weight(block1_id)
            );
        };
        assert_eq!(1, block1.commands.len());
        let Some(next_block_id) = graph.directed_neighbor_node_id_of_type(
            block1_id,
            Direction::Outgoing,
            BasicBlockEdgeType::Next,
        ) else {
            panic!("{:?} had no Next edge", block1_id);
        };
        let Some(BasicBlock::SinglePath(next_block)) = graph.node_weight(next_block_id) else {
            panic!(
                "Unexpected weight for {:?}: {:?}",
                next_block_id,
                graph.node_weight(next_block_id)
            );
        };
        assert_eq!(1, next_block.commands.len());
    }

    #[test]
    fn test_concatenate_linear_blocks() {
        let mut graph = StableDiGraph::<BasicBlock, BasicBlockEdgeType>::new();
        let block1_id = graph.add_node(BasicBlock::SinglePath(SinglePathBasicBlock {
            commands: vec![build_increment_command_node(1)],
            label: Some("Address1".to_string()),
        }));
        let block2_id = graph.add_node(BasicBlock::SinglePath(SinglePathBasicBlock {
            commands: vec![build_increment_command_node(3)],
            label: None,
        }));
        graph.add_edge(block1_id, block2_id, BasicBlockEdgeType::Next);

        assert!(
            concatenate_linear_blocks(&mut graph, block2_id).is_changed(),
            "Block was not removed"
        );

        assert_eq!(1, graph.node_count());
        assert_eq!(0, graph.edge_count());
        let Some(BasicBlock::SinglePath(block1)) = graph.node_weight(block1_id) else {
            panic!(
                "Unexpected weight for {:?}: {:?}",
                block1_id,
                graph.node_weight(block1_id)
            );
        };
        assert_eq!(2, block1.commands.len());
    }

    #[test]
    fn smoke_test_optimization() {
        let collection = uriquest_resources();
        let logic_data = collection
            .read_resource_data(ResourceType::LOGIC, 0)
            .expect("Failed to read logic resource 0");
        let logic_program =
            LogicProgram::decode_from_bytes(&logic_data.data, &AGIVersion::new(2, 917))
                .expect("Failed to decode logic program");

        let ast = LogicAST::from_instructions(&logic_program.instructions)
            .expect("Failed to build AST from instructions");

        let mut basic_block_graph = BasicBlockGraph::from_ast(&ast);
        let initial_node_count = basic_block_graph.graph.node_count();
        assert!(
            initial_node_count > 0,
            "Basic block graph should not be empty"
        );
        basic_block_graph.optimize();
        assert!(
            basic_block_graph.graph.node_count() < initial_node_count,
            "Basic block graph should be optimized to fewer nodes"
        );
    }
}
