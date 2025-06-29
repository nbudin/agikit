use std::collections::HashMap;

use petgraph::{
    graph::{DiGraph, EdgeIndex, NodeIndex},
    visit::{Dfs, EdgeRef, Walker},
    Direction,
};

#[cfg(feature = "dot")]
use crate::logic::asm::codegen::AsmCodeGenerationContext;
use crate::logic::{
    logic_script::{
        ast::{LogicAST, LogicASTNode, LogicCommandNode},
        codegen::errors::LogicScriptCodeGenerationError,
    },
    LogicConditionClause,
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

impl BasicBlock {
    pub fn label(&self) -> Option<&str> {
        match self {
            BasicBlock::SinglePath(block) => block.label.as_deref(),
            BasicBlock::Conditional(block) => block.label.as_deref(),
        }
    }

    pub fn set_label(&mut self, label: Option<String>) {
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
    pub graph: DiGraph<BasicBlock, BasicBlockEdgeType>,
    pub root_block_id: NodeIndex,
}

impl BasicBlockGraph {
    pub fn from_ast(ast: &LogicAST) -> Self {
        let mut graph = DiGraph::new();
        let mut block_ids_by_node_id = HashMap::new();
        let root_block_id =
            build_basic_blocks(ast, ast.root_node_id, &mut graph, &mut block_ids_by_node_id);

        BasicBlockGraph {
            graph,
            root_block_id,
        }
    }

    pub fn run_optimization_pass(&mut self, visitor: &mut dyn BasicBlockVisitor) -> bool {
        let queue = Dfs::new(&self.graph, self.root_block_id)
            .iter(&self.graph)
            .map(|block_id| block_id)
            .collect::<Vec<_>>();
        let mut changed = false;
        for block_id in queue {
            if visitor.visit_basic_block(self, block_id) {
                changed = true;
            }
        }
        changed
    }

    pub fn run_optimization_passes(&mut self, visitors: &mut [Box<dyn BasicBlockVisitor>]) -> () {
        let mut keep_going = true;
        while keep_going {
            let mut changed = false;
            for visitor in &mut *visitors {
                if self.run_optimization_pass(visitor.as_mut()) {
                    changed = true;
                }
            }
            keep_going = changed;
        }
    }

    pub fn optimize(&mut self) {
        let mut visitors: Vec<Box<dyn BasicBlockVisitor>> = vec![
            Box::new(remove_empty_block),
            Box::new(concatenate_linear_blocks),
        ];
        self.run_optimization_passes(&mut visitors);
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
    ) -> Result<BasicBlockControlFlow, LogicScriptCodeGenerationError> {
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

#[cfg(feature = "dot")]
impl BasicBlockGraph {
    pub fn to_dot(&self, asm_context: &AsmCodeGenerationContext) -> String {
        use petgraph::dot::{Config, Dot};

        use crate::logic::asm::expressions::LogicBooleanExpression;

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
                    use crate::logic::asm::codegen::GenerateLogicAsm;

                    let (shape, node_desc) = match node_weight {
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
                                LogicBooleanExpression::from_clauses(
                                    &block.conditions,
                                    asm_context
                                )
                                .expect("Error generating conditions")
                                .generate_asm(asm_context, &HashMap::new())
                                .expect("Error generating asm"),
                            ),
                        ),
                    };

                    let label = format!(
                        "{}\n{}",
                        node_weight
                            .label()
                            .map(|l| l.to_owned())
                            .unwrap_or_else(|| format!("Node {}", node_id.index())),
                        node_desc
                    );
                    format!(
                        "shape = {}, label = {}",
                        shape,
                        serde_json::to_string(&label).unwrap()
                    )
                },
            )
        )
    }
}

fn build_basic_blocks(
    ast: &LogicAST,
    node_id: NodeIndex,
    graph: &mut DiGraph<BasicBlock, BasicBlockEdgeType>,
    block_ids_by_node_id: &mut HashMap<NodeIndex, NodeIndex>,
) -> NodeIndex {
    let find_or_build_blocks_for_node =
        |node_id: NodeIndex,
         graph: &mut DiGraph<BasicBlock, BasicBlockEdgeType>,
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

fn remove_block(graph: &mut BasicBlockGraph, block_id: NodeIndex, new_target_block_id: NodeIndex) {
    let block_label = graph
        .graph
        .node_weight(block_id)
        .and_then(|block| block.label().map(|l| l.to_owned()));
    let incoming_edges = graph
        .graph
        .edges_directed(block_id, Direction::Incoming)
        .map(|edge| (edge.id(), edge.source(), edge.weight().clone()))
        .collect::<Vec<_>>();
    let outgoing_edges = graph
        .graph
        .edges_directed(block_id, Direction::Outgoing)
        .map(|edge| (edge.id(), edge.target(), edge.weight().clone()))
        .collect::<Vec<_>>();

    for (edge_id, source_id, edge_weight) in incoming_edges {
        graph.graph.remove_edge(edge_id);
        if source_id != new_target_block_id {
            graph
                .graph
                .add_edge(source_id, new_target_block_id, edge_weight);
        }
    }

    for (edge_id, target_id, edge_weight) in outgoing_edges {
        graph.graph.remove_edge(edge_id);
        if new_target_block_id != target_id {
            graph
                .graph
                .add_edge(new_target_block_id, target_id, edge_weight);
        }
    }

    graph.graph.remove_node(block_id);

    let new_target_block = graph.graph.node_weight_mut(new_target_block_id).unwrap();
    if new_target_block.label().is_none() && block_label.is_some() {
        new_target_block.set_label(block_label);
    }
}

pub fn remove_empty_block(graph: &mut BasicBlockGraph, block_id: NodeIndex) -> bool {
    let block = graph.graph.node_weight(block_id);
    if let Some(BasicBlock::SinglePath(block)) = block {
        if let Some(edge_id) = graph.directed_neighbor_edge_id_of_type(
            block_id,
            Direction::Outgoing,
            BasicBlockEdgeType::Next,
        ) {
            let (_, next_block_id) = graph.graph.edge_endpoints(edge_id).unwrap();
            if block.commands.is_empty() {
                remove_block(graph, block_id, next_block_id);
                return true;
            }
        }
    }

    false
}

pub fn concatenate_linear_blocks(graph: &mut BasicBlockGraph, block_id: NodeIndex) -> bool {
    let block = graph.graph.node_weight(block_id);
    if let Some(BasicBlock::SinglePath(block)) = block {
        if let Some(prev_edge_id) = graph.directed_neighbor_edge_id_of_type(
            block_id,
            Direction::Incoming,
            BasicBlockEdgeType::Next,
        ) {
            if let Some(next_edge_id) = graph.directed_neighbor_edge_id_of_type(
                block_id,
                Direction::Outgoing,
                BasicBlockEdgeType::Next,
            ) {
                let (prev_block_id, _) = graph.graph.edge_endpoints(prev_edge_id).unwrap();
                let (_, next_block_id) = graph.graph.edge_endpoints(next_edge_id).unwrap();
                let commands = block.commands.clone();

                let prev_block = graph.graph.node_weight_mut(prev_block_id).unwrap();
                if let BasicBlock::SinglePath(prev_block) = prev_block {
                    prev_block.commands.extend(commands);

                    remove_block(graph, block_id, next_block_id);
                    return true;
                }
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use crate::{
        agi_version::AGIVersion,
        logic::{
            logic_script::{ast::LogicAST, basic_block_graph::BasicBlockGraph},
            LogicProgram,
        },
        resources::{decode::Decode, ResourceType},
        test_data::uriquest_resources,
    };

    #[test]
    fn test_optimization() {
        let collection = uriquest_resources();
        let logic_data = collection
            .read_resource_data(ResourceType::LOGIC, 0)
            .expect("Failed to read logic resource 0");
        let logic_program = LogicProgram::decode_from_bytes(&logic_data, &AGIVersion::new(2, 917))
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
