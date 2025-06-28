use std::collections::{HashMap, HashSet};

use petgraph::{
    graph::{DiGraph, NodeIndex},
    visit::{Dfs, EdgeRef},
};

use crate::logic::logic_script::basic_block_graph::BasicBlockGraph;

#[derive(Debug, Clone)]
pub enum ReverseCFGNode {
    BasicBlock { id: NodeIndex },
    VirtualRoot,
}

#[derive(Debug, Clone)]
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
            let reverse_node_id = reverse_nodes_by_block_id
                .entry(block_id)
                .or_insert_with(|| {
                    reverse_graph.add_node(ReverseCFGNode::BasicBlock { id: block_id })
                })
                .clone();

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
                    })
                    .clone();
                if !reverse_graph.contains_edge(target_reverse_node_id, reverse_node_id) {
                    reverse_graph.add_edge(target_reverse_node_id, reverse_node_id, ());
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

#[cfg(test)]
mod tests {
    use crate::logic::logic_script::basic_block_graph::{
        BasicBlock, BasicBlockEdgeType, SinglePathBasicBlock,
    };

    use super::*;

    #[test]
    fn test_reverse_control_flow_graph() {
        let mut graph = DiGraph::new();
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

        let reverse_cfg = ReverseCFG::from_basic_block_graph(&basic_block_graph);

        assert_eq!(reverse_cfg.graph.node_count(), 5);
        assert_eq!(reverse_cfg.graph.edge_count(), 4);
        assert!(reverse_cfg
            .graph
            .node_weight(reverse_cfg.virtual_root_id)
            .is_some());
    }
}
