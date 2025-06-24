use std::collections::{HashMap, HashSet};

use petgraph::{
    graph::{DiGraph, NodeIndex},
    visit::{Dfs, EdgeRef},
};

use crate::logic::logic_script::basic_block_graph::BasicBlockGraph;

pub enum ReverseCFGNode {
    BasicBlock { id: NodeIndex },
    VirtualRoot,
}

pub struct ReverseCFG {
    pub graph: DiGraph<ReverseCFGNode, ()>,
    pub virtual_root_id: NodeIndex,
}

impl ReverseCFG {
    pub fn from_basic_block_graph(basic_block_graph: &BasicBlockGraph) -> Self {
        let mut reverse_nodes_by_block_id = HashMap::new();
        let mut reverse_graph = DiGraph::new();
        let mut root_ids = HashSet::new();

        let mut dfs = Dfs::new(&basic_block_graph.graph, basic_block_graph.root_block_id);
        while let Some(block_id) = dfs.next(&basic_block_graph.graph) {
            let reverse_node_id =
                reverse_graph.add_node(ReverseCFGNode::BasicBlock { id: block_id });
            reverse_nodes_by_block_id.insert(block_id, reverse_node_id);

            for edge in basic_block_graph
                .graph
                .edges_directed(block_id, petgraph::Direction::Incoming)
            {
                let source_block_id = edge.source();
                let source_reverse_node_id = reverse_nodes_by_block_id
                    .entry(source_block_id)
                    .or_insert_with(|| {
                        reverse_graph.add_node(ReverseCFGNode::BasicBlock {
                            id: source_block_id,
                        })
                    });
                if !reverse_graph.contains_edge(reverse_node_id, *source_reverse_node_id) {
                    reverse_graph.add_edge(reverse_node_id, *source_reverse_node_id, ());
                }
            }

            for edge in basic_block_graph
                .graph
                .edges_directed(block_id, petgraph::Direction::Outgoing)
            {
                let target_block_id = edge.target();
                let target_reverse_node_id = reverse_nodes_by_block_id
                    .entry(target_block_id)
                    .or_insert_with(|| {
                        reverse_graph.add_node(ReverseCFGNode::BasicBlock {
                            id: target_block_id,
                        })
                    });
                if !reverse_graph.contains_edge(reverse_node_id, *target_reverse_node_id) {
                    reverse_graph.add_edge(reverse_node_id, *target_reverse_node_id, ());
                }
            }

            if basic_block_graph
                .graph
                .edges_directed(block_id, petgraph::Direction::Outgoing)
                .count()
                == 0
            {
                root_ids.insert(reverse_node_id);
            }
        }

        let virtual_root_id = reverse_graph.add_node(ReverseCFGNode::VirtualRoot);
        for root_id in root_ids {
            reverse_graph.add_edge(virtual_root_id, root_id, ());
        }

        ReverseCFG {
            graph: reverse_graph,
            virtual_root_id,
        }
    }
}
