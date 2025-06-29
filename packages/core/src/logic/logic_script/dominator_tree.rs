use std::collections::HashMap;

use petgraph::{
    csr::{DefaultIx, IndexType},
    graph::{DiGraph, NodeIndex},
    visit::{Dfs, EdgeRef},
    Direction,
};

#[derive(Debug, Clone)]
pub struct DominatorTreeNode<Ix: IndexType = DefaultIx> {
    pub cfg_index: NodeIndex<Ix>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DominatorTreeEdgeType {
    ImmediateDominator,
}

pub struct DominatorTree<Ix: IndexType = DefaultIx> {
    pub graph: DiGraph<DominatorTreeNode<Ix>, DominatorTreeEdgeType>,
    pub root_id: NodeIndex,
    dominator_tree_index_by_cfg_index: HashMap<NodeIndex<Ix>, NodeIndex>,
}

impl<Ix: IndexType> DominatorTree<Ix> {
    pub fn from_cfg<NodeType, EdgeType>(
        cfg: &DiGraph<NodeType, EdgeType, Ix>,
        start: NodeIndex<Ix>,
    ) -> Self {
        SemiNCASpanningTree::from_cfg(cfg, start).build_dominator_tree()
    }

    pub fn immediate_dominator(&self, cfg_index: NodeIndex<Ix>) -> Option<NodeIndex<Ix>> {
        let node_index = self
            .dominator_tree_index_by_cfg_index
            .get(&cfg_index)
            .unwrap();
        self.graph
            .edges_directed(*node_index, Direction::Incoming)
            .next()
            .map(|edge| self.graph[edge.source()].cfg_index)
    }

    pub fn dominates(&self, a: NodeIndex<Ix>, b: NodeIndex<Ix>) -> bool {
        if a == b {
            return true;
        }

        let mut current_b = b;
        while let Some(idom) = self.immediate_dominator(current_b) {
            if idom == a {
                return true;
            }
            current_b = idom;
        }

        false
    }

    pub fn immediately_dominates(&self, a: NodeIndex<Ix>, b: NodeIndex<Ix>) -> bool {
        self.immediate_dominator(b) == Some(a)
    }
}

#[derive(Debug, Clone)]
struct SpanningTreeNodeInfo<Ix: IndexType = DefaultIx> {
    dfs_num: usize,
    idom: Option<NodeIndex<Ix>>,
    sdom: Option<NodeIndex<Ix>>,
    parent: Option<NodeIndex<Ix>>,
    ancestor: Option<NodeIndex<Ix>>,
    best: Option<NodeIndex<Ix>>,
}

struct SemiNCASpanningTree<Ix: IndexType = DefaultIx> {
    root_id: NodeIndex<Ix>,
    spanning_tree_node_info: HashMap<NodeIndex<Ix>, SpanningTreeNodeInfo<Ix>>,
    cfg_indexes_in_dfs_order: Vec<NodeIndex<Ix>>,
}

impl<Ix: IndexType> SemiNCASpanningTree<Ix> {
    pub fn from_cfg<NodeType, EdgeType>(
        cfg: &DiGraph<NodeType, EdgeType, Ix>,
        start: NodeIndex<Ix>,
    ) -> Self {
        let mut nodes_in_dfs_order = Vec::with_capacity(cfg.node_count());
        let mut spanning_tree_node_info = HashMap::with_capacity(cfg.node_count());
        let mut dfs = Dfs::new(cfg, start);

        while let Some(node_index) = dfs.next(cfg) {
            let parent = cfg
                .edges_directed(node_index, Direction::Incoming)
                .next()
                .map(|edge| edge.source());

            spanning_tree_node_info.insert(
                node_index,
                SpanningTreeNodeInfo {
                    dfs_num: nodes_in_dfs_order.len(),
                    idom: parent,
                    sdom: None,
                    parent,
                    ancestor: None,
                    best: None,
                },
            );
            nodes_in_dfs_order.push(node_index);
        }

        let mut semi_nca_tree = SemiNCASpanningTree {
            root_id: start,
            spanning_tree_node_info,
            cfg_indexes_in_dfs_order: nodes_in_dfs_order,
        };
        semi_nca_tree.compute_semidominators(cfg);
        semi_nca_tree.compute_immediate_dominators();
        semi_nca_tree
    }

    pub fn build_dominator_tree(&self) -> DominatorTree<Ix> {
        let mut graph = DiGraph::new();
        let dominator_tree_nodes_by_cfg_index = self
            .cfg_indexes_in_dfs_order
            .iter()
            .map(|&cfg_index| (cfg_index, graph.add_node(DominatorTreeNode { cfg_index })))
            .collect::<HashMap<_, _>>();

        for cfg_index in self.cfg_indexes_in_dfs_order.iter() {
            let node_info = &self.spanning_tree_node_info[&cfg_index];
            let dt_node = dominator_tree_nodes_by_cfg_index[cfg_index];
            let idom = node_info.idom;

            if let Some(idom) = idom {
                graph.add_edge(
                    dominator_tree_nodes_by_cfg_index[&idom],
                    dt_node,
                    DominatorTreeEdgeType::ImmediateDominator,
                );
            }
        }

        DominatorTree {
            graph,
            root_id: dominator_tree_nodes_by_cfg_index[&self.root_id],
            dominator_tree_index_by_cfg_index: dominator_tree_nodes_by_cfg_index,
        }
    }

    fn link(&mut self, ancestor: NodeIndex<Ix>, node: NodeIndex<Ix>) {
        let node = &mut self.spanning_tree_node_info.get_mut(&node).unwrap();
        node.ancestor = Some(ancestor);
        node.best = Some(ancestor);
    }

    fn ancestor_with_lowest_semi(&mut self, node: NodeIndex<Ix>) -> Option<NodeIndex<Ix>> {
        let mut working_node_info = self.spanning_tree_node_info.get(&node).cloned();
        if let Some(working_node_info) = &mut working_node_info {
            let ancestor = working_node_info.ancestor;

            if let Some(ancestor) = ancestor {
                let candidate_best = self.ancestor_with_lowest_semi(ancestor);
                working_node_info.ancestor = self.spanning_tree_node_info[&ancestor].ancestor;

                let candidate_best_sdom = candidate_best
                    .and_then(|candiate_best| self.spanning_tree_node_info[&candiate_best].sdom);
                let current_best = working_node_info.best;
                let current_best_sdom =
                    current_best.and_then(|best| self.spanning_tree_node_info[&best].sdom);

                if let Some(candidate_best_sdom) = candidate_best_sdom {
                    if let Some(current_best_sdom) = current_best_sdom {
                        if self.spanning_tree_node_info[&candidate_best_sdom].dfs_num
                            < self.spanning_tree_node_info[&current_best_sdom].dfs_num
                        {
                            working_node_info.best = candidate_best;
                        }
                    }
                }
            }

            self.spanning_tree_node_info
                .insert(node, working_node_info.clone());
        }

        working_node_info.and_then(|ni| ni.best)
    }

    fn compute_semidominators<NodeType, EdgeType>(
        &mut self,
        cfg: &DiGraph<NodeType, EdgeType, Ix>,
    ) {
        // iterate nodes in reverse DFS order, omitting the root
        let reverse_dfs_order_without_root = self.cfg_indexes_in_dfs_order[1..]
            .iter()
            .rev()
            .cloned()
            .collect::<Vec<_>>();

        for &node in reverse_dfs_order_without_root.iter() {
            // we know there will be a parent because we're omitting the root
            let parent = self.spanning_tree_node_info[&node].parent.unwrap();
            let mut semi = parent;

            for inward_edge in cfg.edges_directed(node, Direction::Incoming) {
                let predecessor_index = inward_edge.source();
                let node_dfs_num = self.spanning_tree_node_info[&node].dfs_num;
                let predecessor_dfs_num = self.spanning_tree_node_info[&predecessor_index].dfs_num;

                let candidate = if predecessor_dfs_num < node_dfs_num {
                    Some(predecessor_index)
                } else {
                    let ancestor_with_lowest: Option<NodeIndex<Ix>> =
                        self.ancestor_with_lowest_semi(predecessor_index);
                    ancestor_with_lowest.and_then(|ancestor_index| {
                        self.spanning_tree_node_info[&ancestor_index].sdom
                    })
                };

                if let Some(candidate_index) = candidate {
                    if self.spanning_tree_node_info[&candidate_index].dfs_num
                        < self.spanning_tree_node_info[&semi].dfs_num
                    {
                        semi = candidate_index;
                    }
                }
            }

            self.spanning_tree_node_info.get_mut(&node).unwrap().sdom = Some(semi);
            self.link(parent, node);
        }
    }

    fn compute_immediate_dominators(&mut self) {
        // iterate nodes in DFS order, omitting the root
        let dfs_order_without_root = self.cfg_indexes_in_dfs_order[1..]
            .iter()
            .cloned()
            .collect::<Vec<_>>();

        for &node in dfs_order_without_root.iter() {
            // we know there's an idom for this node because we initialize it to the parent for all
            // nodes (except the root, for which there's no parent)
            let mut idom = self.spanning_tree_node_info[&node].idom.unwrap();
            let sdom = self.spanning_tree_node_info[&node].sdom.unwrap();
            let sdom_dfs_num = self.spanning_tree_node_info[&sdom].dfs_num;

            while self.spanning_tree_node_info[&idom].dfs_num > sdom_dfs_num {
                idom = self.spanning_tree_node_info[&idom].idom.unwrap();
            }

            self.spanning_tree_node_info.get_mut(&node).unwrap().idom = Some(idom);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::logic::logic_script::dominator_tree::DominatorTree;
    use petgraph::graph::DiGraph;

    #[test]
    fn test_simple_dominator_tree() {
        let mut graph = DiGraph::<(), ()>::new();
        let n0 = graph.add_node(());
        let n1 = graph.add_node(());
        let n2 = graph.add_node(());
        let n3 = graph.add_node(());
        let n4 = graph.add_node(());

        graph.extend_with_edges(&[(n0, n1), (n0, n2), (n1, n3), (n2, n3), (n2, n4)]);

        let dominator_tree = DominatorTree::from_cfg(&graph, n0);
        assert!(dominator_tree.dominates(n0, n1));
        assert!(dominator_tree.dominates(n0, n2));
        assert!(dominator_tree.dominates(n0, n3));
        assert!(dominator_tree.dominates(n2, n4));
        assert!(!dominator_tree.dominates(n1, n2));
    }
}
