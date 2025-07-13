use std::collections::{HashMap, HashSet};

use petgraph::{
    Directed, Direction,
    csr::{DefaultIx, IndexType},
    graph::NodeIndex,
    prelude::StableDiGraph,
    visit::{Dfs, EdgeRef},
};

use crate::logic::analysis::node_reference::{NodeReference, ReferenceGraph};

#[derive(Debug, Clone)]
pub enum InvertedGraphNode<Ix: IndexType = DefaultIx> {
    NodeReference(NodeReference<Ix>),
    VirtualRoot,
}

impl<Ix: IndexType> TryFrom<InvertedGraphNode<Ix>> for NodeReference<Ix> {
    type Error = InvertedGraphNode<Ix>;

    fn try_from(value: InvertedGraphNode<Ix>) -> Result<Self, Self::Error> {
        match value {
            InvertedGraphNode::NodeReference(node_reference) => Ok(node_reference),
            InvertedGraphNode::VirtualRoot => Err(value),
        }
    }
}

impl<'a, Ix: IndexType> TryFrom<&'a InvertedGraphNode<Ix>> for NodeReference<Ix> {
    type Error = InvertedGraphNode<Ix>;

    fn try_from(value: &'a InvertedGraphNode<Ix>) -> Result<Self, Self::Error> {
        match value {
            InvertedGraphNode::NodeReference(node_reference) => Ok(node_reference.clone()),
            InvertedGraphNode::VirtualRoot => Err(value.clone()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct InvertedGraph<Ix: IndexType = DefaultIx> {
    pub virtual_root_id: NodeIndex,
    pub reverse_graph: StableDiGraph<InvertedGraphNode<Ix>, ()>,
    reverse_nodes_by_node_id: HashMap<NodeIndex<Ix>, NodeIndex>,
}

impl<Ix: IndexType> InvertedGraph<Ix> {
    pub fn from_graph<'a, N, E>(
        graph: &'a StableDiGraph<N, E, Ix>,
        root_id: NodeIndex<Ix>,
    ) -> Self {
        let mut reverse_nodes_by_node_id = HashMap::new();
        let mut reverse_graph = StableDiGraph::new();
        let mut root_ids = HashSet::new();

        let mut dfs = Dfs::new(&graph, root_id);
        while let Some(block_id) = dfs.next(&graph) {
            let reverse_node_id = reverse_nodes_by_node_id
                .entry(block_id)
                .or_insert_with(|| {
                    reverse_graph.add_node(InvertedGraphNode::NodeReference(block_id.into()))
                })
                .clone();

            for edge in graph.edges_directed(block_id, Direction::Outgoing) {
                let target_block_id = edge.target();
                let target_reverse_node_id = reverse_nodes_by_node_id
                    .entry(target_block_id)
                    .or_insert_with(|| {
                        reverse_graph
                            .add_node(InvertedGraphNode::NodeReference(target_block_id.into()))
                    })
                    .clone();
                if !reverse_graph.contains_edge(target_reverse_node_id, reverse_node_id) {
                    reverse_graph.add_edge(target_reverse_node_id, reverse_node_id, ());
                }
            }

            if graph
                .edges_directed(block_id, petgraph::Direction::Outgoing)
                .count()
                == 0
            {
                root_ids.insert(reverse_node_id);
            }
        }

        let virtual_root_id = reverse_graph.add_node(InvertedGraphNode::VirtualRoot);
        for root_id in root_ids {
            reverse_graph.add_edge(virtual_root_id, root_id, ());
        }

        InvertedGraph {
            reverse_graph,
            virtual_root_id,
            reverse_nodes_by_node_id,
        }
    }
}

impl<Ix: IndexType> ReferenceGraph<InvertedGraphNode<Ix>, (), Directed, Ix> for InvertedGraph<Ix> {
    fn reference_graph(&self) -> &StableDiGraph<InvertedGraphNode<Ix>, ()> {
        &self.reverse_graph
    }

    fn reference_node_id_for_source_node_id(
        &self,
        source_node_id: NodeIndex<Ix>,
    ) -> Option<NodeIndex> {
        self.reverse_nodes_by_node_id.get(&source_node_id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use crate::logic::analysis::basic_block_graph::{
        BasicBlock, BasicBlockEdgeType, BasicBlockGraph, SinglePathBasicBlock,
    };

    use super::*;

    #[test]
    fn test_reverse_control_flow_graph() {
        let mut graph = StableDiGraph::new();
        let block_a = graph.add_node(BasicBlock::SinglePath(SinglePathBasicBlock {
            commands: vec![],
            label: Some("A".to_string()),
        }));
        let block_b = graph.add_node(BasicBlock::SinglePath(SinglePathBasicBlock {
            commands: vec![],
            label: Some("B".to_string()),
        }));
        let block_c = graph.add_node(BasicBlock::SinglePath(SinglePathBasicBlock {
            commands: vec![],
            label: Some("C".to_string()),
        }));
        let block_d = graph.add_node(BasicBlock::SinglePath(SinglePathBasicBlock {
            commands: vec![],
            label: Some("D".to_string()),
        }));

        graph.add_edge(block_a, block_b, BasicBlockEdgeType::Next);
        graph.add_edge(block_b, block_c, BasicBlockEdgeType::Next);
        graph.add_edge(block_c, block_d, BasicBlockEdgeType::Next);

        let basic_block_graph = BasicBlockGraph {
            graph,
            root_block_id: block_a,
        };

        let reverse_cfg =
            InvertedGraph::from_graph(&basic_block_graph.graph, basic_block_graph.root_block_id);

        assert_eq!(reverse_cfg.reverse_graph.node_count(), 5);
        assert_eq!(reverse_cfg.reverse_graph.edge_count(), 4);
        assert!(
            reverse_cfg
                .reverse_graph
                .node_weight(reverse_cfg.virtual_root_id)
                .is_some()
        );
    }
}
