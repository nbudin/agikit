use std::collections::HashMap;

use petgraph::{
    graph::{DiGraph, EdgeIndex, NodeIndex},
    visit::{Dfs, EdgeRef},
    Direction,
};

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
pub struct BasicBlock {
    pub label: Option<String>,
    pub commands: Vec<LogicCommandNode>,
    pub conditions: Option<Vec<LogicConditionClause>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BasicBlockEdgeType {
    Next,
    IfThen,
    IfElse,
}

pub enum BasicBlockControlFlow {
    SinglePath {
        next_id: Option<NodeIndex>,
    },
    Conditional {
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
        let mut dfs = Dfs::new(&self.graph, self.root_block_id);
        let mut changed = false;
        while let Some(block_id) = dfs.next(&self.graph) {
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
        let edges = self
            .graph
            .edges_directed(block_id, Direction::Outgoing)
            .collect::<Vec<_>>();
        let edge_types = edges.iter().map(|e| e.weight().clone()).collect::<Vec<_>>();

        if edge_types.is_empty() {
            Ok(BasicBlockControlFlow::SinglePath { next_id: None })
        } else if edge_types == vec![BasicBlockEdgeType::Next] {
            Ok(BasicBlockControlFlow::SinglePath {
                next_id: Some(edges.first().unwrap().target()),
            })
        } else if edge_types == vec![BasicBlockEdgeType::IfThen]
            || edge_types == vec![BasicBlockEdgeType::IfThen, BasicBlockEdgeType::IfElse]
            || edge_types == vec![BasicBlockEdgeType::IfElse, BasicBlockEdgeType::IfThen]
        {
            Ok(BasicBlockControlFlow::Conditional {
                conditions: self
                    .graph
                    .node_weight(block_id)
                    .and_then(|block| block.conditions.clone())
                    .unwrap_or_default(),
                then_id: edges
                    .iter()
                    .find(|edge| *edge.weight() == BasicBlockEdgeType::IfThen)
                    .map(|edge| edge.target())
                    .unwrap(),
                else_id: edges
                    .iter()
                    .find(|edge| *edge.weight() == BasicBlockEdgeType::IfElse)
                    .map(|edge| edge.target()),
            })
        } else {
            Err(
                LogicScriptCodeGenerationError::MalformedBasicBlockEdgeTypes(
                    block_id,
                    self.graph.node_weight(block_id).cloned(),
                    edge_types,
                ),
            )
        }
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
            let block = BasicBlock {
                label: node.label.as_ref().map(|l| l.label.clone()),
                commands: vec![node.clone()],
                conditions: None,
            };
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
            let block = BasicBlock {
                commands: vec![],
                label: node.label.as_ref().map(|l| l.label.clone()),
                conditions: Some(node.clauses.clone()),
            };
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
            let block = BasicBlock {
                commands: vec![],
                label: node.label.as_ref().map(|l| l.label.clone()),
                conditions: None,
            };
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

pub fn remove_empty_block(graph: &mut BasicBlockGraph, block_id: NodeIndex) -> bool {
    if let Some(edge_id) = graph.directed_neighbor_edge_id_of_type(
        block_id,
        Direction::Outgoing,
        BasicBlockEdgeType::Next,
    ) {
        let (prev_block_id, next_block_id) = graph.graph.edge_endpoints(edge_id).unwrap();
        let block = graph.graph.node_weight(block_id).unwrap();

        if block.commands.is_empty() {
            graph
                .graph
                .add_edge(prev_block_id, next_block_id, BasicBlockEdgeType::Next);
            graph.graph.remove_node(block_id);
            graph.graph.remove_edge(edge_id);
            return true;
        }
    }

    false
}

pub fn concatenate_linear_blocks(graph: &mut BasicBlockGraph, block_id: NodeIndex) -> bool {
    if let Some(next_edge_id) = graph.directed_neighbor_edge_id_of_type(
        block_id,
        Direction::Outgoing,
        BasicBlockEdgeType::Next,
    ) {
        let prev_edge_id = graph.directed_neighbor_edge_id_of_type(
            block_id,
            Direction::Incoming,
            BasicBlockEdgeType::Next,
        );

        if let Some(prev_edge_id) = prev_edge_id {
            let (prev_block_id, next_block_id) = graph.graph.edge_endpoints(prev_edge_id).unwrap();
            let commands = graph.graph.node_weight(block_id).unwrap().commands.clone();

            let prev_block = graph.graph.node_weight_mut(prev_block_id).unwrap();
            prev_block.commands.extend(commands);

            graph.graph.remove_edge(prev_edge_id);
            graph.graph.remove_edge(next_edge_id);
            graph
                .graph
                .add_edge(prev_block_id, next_block_id, BasicBlockEdgeType::Next);
            graph.graph.remove_node(block_id);
            return true;
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
