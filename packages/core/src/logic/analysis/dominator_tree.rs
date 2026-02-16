use std::collections::{HashMap, HashSet};

use petgraph::{
    Directed, Direction,
    csr::{DefaultIx, IndexType},
    graph::NodeIndex,
    prelude::{EdgeIndex, StableDiGraph, StableGraph},
    visit::{Dfs, EdgeRef, GraphBase, IntoEdgesDirected, IntoNeighbors, NodeCount, VisitMap, Visitable},
};

use crate::logic::analysis::{
    invert_graph::{InvertedGraph, InvertedGraphNode},
    node_reference::{NodeReference, ReferenceGraph},
};

pub enum ImmediatePostDominator<Ix: IndexType = DefaultIx> {
    Node(NodeIndex<Ix>),
    VirtualRoot,
}

pub struct DominationAnalysis<Ix: IndexType = DefaultIx> {
    dominator_tree: DominatorTree<Ix>,
    reverse_cfg: InvertedGraph<Ix>,
    post_dominator_tree: DominatorTree,
}

impl<Ix: IndexType> DominationAnalysis<Ix> {
    pub fn from_graph<'a, G>(graph: &'a G, root_id: NodeIndex<Ix>) -> Self
    where
        &'a G: GraphBase<NodeId = NodeIndex<Ix>, EdgeId = EdgeIndex<Ix>>
            + NodeCount
            + Visitable
            + IntoEdgesDirected,
    {
        let dominator_tree = DominatorTree::from_graph(&graph, root_id);
        let reverse_cfg = InvertedGraph::from_graph(&graph, root_id);
        let post_dominator_tree = DominatorTree::from_graph::<
            StableDiGraph<InvertedGraphNode<Ix>, ()>,
        >(
            reverse_cfg.reference_graph(), reverse_cfg.virtual_root_id
        );

        Self {
            dominator_tree,
            reverse_cfg,
            post_dominator_tree,
        }
    }

    pub fn dominates(&self, a: NodeIndex<Ix>, b: NodeIndex<Ix>) -> bool {
        self.dominator_tree.dominates(a, b)
    }

    pub fn immediately_dominates(&self, a: NodeIndex<Ix>, b: NodeIndex<Ix>) -> bool {
        self.dominator_tree.immediately_dominates(a, b)
    }

    pub fn immediate_dominator(&self, node_index: NodeIndex<Ix>) -> Option<NodeIndex<Ix>> {
        self.dominator_tree.immediate_dominator(node_index)
    }

    pub fn dominance_frontier(
        &self,
        node_index: NodeIndex<Ix>,
    ) -> impl Iterator<Item = NodeIndex<Ix>> {
        self.dominator_tree.dominance_frontier(node_index)
    }

    pub fn post_dominates(&self, a: NodeIndex<Ix>, b: NodeIndex<Ix>) -> bool {
        let Some(reverse_a) = self.reverse_cfg.reference_node_id_for_source_node_id(a) else {
            return false;
        };
        let Some(reverse_b) = self.reverse_cfg.reference_node_id_for_source_node_id(b) else {
            return false;
        };
        self.post_dominator_tree.dominates(reverse_a, reverse_b)
    }

    pub fn immediately_post_dominates(&self, a: NodeIndex<Ix>, b: NodeIndex<Ix>) -> bool {
        let Some(reverse_a) = self.reverse_cfg.reference_node_id_for_source_node_id(a) else {
            return false;
        };
        let Some(reverse_b) = self.reverse_cfg.reference_node_id_for_source_node_id(b) else {
            return false;
        };
        self.post_dominator_tree
            .immediately_dominates(reverse_a, reverse_b)
    }

    pub fn immediate_post_dominator(
        &self,
        node_index: NodeIndex<Ix>,
    ) -> Option<ImmediatePostDominator<Ix>> {
        let Some(reverse_node_index) = self
            .reverse_cfg
            .reference_node_id_for_source_node_id(node_index)
        else {
            return None;
        };
        self.post_dominator_tree
            .immediate_dominator(reverse_node_index)
            .map(|reverse_index| {
                let reverse_node = self
                    .reverse_cfg
                    .reverse_graph
                    .node_weight(reverse_index)
                    .unwrap();
                match reverse_node {
                    InvertedGraphNode::NodeReference(node_reference) => {
                        ImmediatePostDominator::Node(node_reference.id())
                    }
                    InvertedGraphNode::VirtualRoot => ImmediatePostDominator::VirtualRoot,
                }
            })
    }

    pub fn post_dominance_frontier(
        &self,
        node_index: NodeIndex<Ix>,
    ) -> Box<dyn Iterator<Item = NodeIndex<Ix>> + '_> {
        let Some(reverse_node_index) = self
            .reverse_cfg
            .reference_node_id_for_source_node_id(node_index)
        else {
            return Box::new(std::iter::empty::<NodeIndex<Ix>>());
        };

        Box::new(
            self.post_dominator_tree
                .dominance_frontier(reverse_node_index)
                .filter_map(|reverse_index| {
                    self.reverse_cfg
                        .source_node_id_for_reference_node_id(reverse_index)
                }),
        )
    }
}

#[cfg(feature = "dot")]
impl<Ix: IndexType> DominationAnalysis<Ix> {
    pub fn dominators_to_dot<N, E, F: Fn(NodeIndex<Ix>, &N) -> String>(
        &self,
        graph: &StableDiGraph<N, E, Ix>,
        node_attrs: &F,
    ) -> String {
        use petgraph::dot::{Config, Dot};

        format!(
            "{:?}",
            Dot::with_attr_getters(
                &self.dominator_tree.graph,
                &[Config::EdgeNoLabel, Config::NodeNoLabel],
                &|_graph, _edge| { "".to_string() },
                &|_graph, (_node_id, node_reference)| {
                    if let Some(node_weight) = graph.node_weight(node_reference.id()) {
                        node_attrs(node_reference.id(), &node_weight)
                    } else {
                        format!(
                            "label = {}",
                            serde_json::to_string(&format!("{:?}", node_reference)).unwrap()
                        )
                    }
                },
            )
        )
    }

    pub fn reverse_cfg_to_dot<N, E, F: Fn(NodeIndex<Ix>, &N) -> String>(
        &self,
        graph: &StableDiGraph<N, E, Ix>,
        node_attrs: &F,
    ) -> String {
        use petgraph::dot::{Config, Dot};

        format!(
            "{:?}",
            Dot::with_attr_getters(
                &self.reverse_cfg.reverse_graph,
                &[Config::EdgeNoLabel, Config::NodeNoLabel],
                &|_graph, _edge| { "".to_string() },
                &|_graph, (reverse_node_id, reverse_node)| {
                    if let Some(source_node_id) = self
                        .reverse_cfg
                        .source_node_id_for_reference_node_id(reverse_node_id)
                    {
                        if let Some(node_weight) = graph.node_weight(source_node_id) {
                            node_attrs(source_node_id, &node_weight)
                        } else {
                            format!(
                                "label = {}",
                                serde_json::to_string(&format!("{:?}", source_node_id)).unwrap()
                            )
                        }
                    } else {
                        format!(
                            "label = {}",
                            serde_json::to_string(&format!("{:?}", reverse_node)).unwrap()
                        )
                    }
                },
            )
        )
    }

    pub fn post_dominators_to_dot<N, E, F: Fn(NodeIndex<Ix>, &N) -> String>(
        &self,
        graph: &StableDiGraph<N, E, Ix>,
        node_attrs: &F,
    ) -> String {
        use petgraph::dot::{Config, Dot};

        format!(
            "{:?}",
            Dot::with_attr_getters(
                &self.post_dominator_tree.graph,
                &[Config::EdgeNoLabel, Config::NodeNoLabel],
                &|_graph, _edge| { "".to_string() },
                &|_graph, (node_id, node_weight)| {
                    if let Some(source_node_id) = self
                        .reverse_cfg
                        .source_node_id_for_reference_node_id(node_id)
                    {
                        if let Some(node_weight) = graph.node_weight(source_node_id) {
                            node_attrs(source_node_id, &node_weight)
                        } else {
                            format!(
                                "label = {}",
                                serde_json::to_string(&format!("{:?}", source_node_id)).unwrap()
                            )
                        }
                    } else {
                        format!(
                            "label = {}",
                            serde_json::to_string(&format!("{:?}", node_weight)).unwrap()
                        )
                    }
                },
            )
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DominatorTreeEdgeType {
    ImmediateDominator,
}

pub struct DominatorTree<Ix: IndexType = DefaultIx> {
    pub root_id: NodeIndex,
    graph: StableDiGraph<NodeReference<Ix>, DominatorTreeEdgeType>,
    dominator_tree_index_by_source_graph_index: HashMap<NodeIndex<Ix>, NodeIndex>,
}

impl<Ix: IndexType> ReferenceGraph<NodeReference<Ix>, DominatorTreeEdgeType, Directed, Ix>
    for DominatorTree<Ix>
{
    fn reference_graph(&self) -> &StableGraph<NodeReference<Ix>, DominatorTreeEdgeType, Directed> {
        &self.graph
    }

    fn reference_node_id_for_source_node_id(
        &self,
        source_node_id: NodeIndex<Ix>,
    ) -> Option<NodeIndex> {
        self.dominator_tree_index_by_source_graph_index
            .get(&source_node_id)
            .cloned()
    }
}

impl<Ix: IndexType> DominatorTree<Ix> {
    pub fn from_graph<'a, G>(graph: &'a G, start: NodeIndex<Ix>) -> Self
    where
        &'a G: GraphBase<NodeId = NodeIndex<Ix>, EdgeId = EdgeIndex<Ix>>
            + NodeCount
            + Visitable
            + IntoEdgesDirected,
    {
        SemiNCASpanningTree::from_graph(graph, start).build_dominator_tree()
    }

    pub fn dominance_frontier(
        &self,
        node_index: NodeIndex<Ix>,
    ) -> impl Iterator<Item = NodeIndex<Ix>> {
        let node_index = self
            .reference_node_id_for_source_node_id(node_index)
            .unwrap();
        self.graph
            .edges_directed(node_index, Direction::Outgoing)
            .into_iter()
            .map(|edge| {
                self.source_node_id_for_reference_node_id(edge.target())
                    .unwrap()
            })
    }

    pub fn immediate_dominator(&self, node_index: NodeIndex<Ix>) -> Option<NodeIndex<Ix>> {
        let node_index = self
            .reference_node_id_for_source_node_id(node_index)
            .unwrap();
        let mut incoming_edges = self.graph.edges_directed(node_index, Direction::Incoming);
        let idom = incoming_edges
            .next()
            .and_then(|edge| self.source_node_id_for_reference_node_id(edge.source()));
        if incoming_edges.next().is_some() {
            panic!("A node should not have multiple immediate dominators!");
        }
        idom
    }

    pub fn dominates(&self, a: NodeIndex<Ix>, b: NodeIndex<Ix>) -> bool {
        let Some(a_ref) = self.reference_node_id_for_source_node_id(a) else {
            return false;
        };
        let Some(b_ref) = self.reference_node_id_for_source_node_id(b) else {
            return false;
        };

        if a_ref == b_ref {
            return true;
        }

        let mut dfs = Dfs::new(&self.graph, a_ref);
        while let Some(node_index) = dfs.next(&self.graph) {
            if node_index == b_ref {
                return true;
            }
        }

        false
    }

    pub fn immediately_dominates(&self, a: NodeIndex<Ix>, b: NodeIndex<Ix>) -> bool {
        self.immediate_dominator(a) == Some(b)
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
    source_graph_indexes_in_dfs_order: Vec<NodeIndex<Ix>>,
}

impl<Ix: IndexType> SemiNCASpanningTree<Ix> {
    pub fn from_graph<'a, G>(graph: &'a G, start: NodeIndex<Ix>) -> Self
    where
        &'a G: GraphBase<NodeId = NodeIndex<Ix>, EdgeId = EdgeIndex<Ix>>
            + IntoEdgesDirected
            + Visitable
            + IntoNeighbors
            + NodeCount,
    {
        let mut nodes_in_dfs_order = Vec::with_capacity(graph.node_count());
        let mut spanning_tree_node_info = HashMap::with_capacity(graph.node_count());

        // Manual DFS that tracks the actual DFS tree parent for each discovered node.
        // The previous implementation incorrectly used the first incoming edge's source
        // as the parent, which could yield a wrong parent when a node has multiple
        // predecessors (e.g., back-edges from loops).
        let mut visited = graph.visit_map();
        visited.visit(start);
        // Stack items are (node_to_visit, dfs_tree_parent)
        let mut stack: Vec<(NodeIndex<Ix>, Option<NodeIndex<Ix>>)> = vec![(start, None)];

        while let Some((node_index, parent)) = stack.pop() {
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

            // Push neighbors in reverse order so we visit them in forward order
            let neighbors: Vec<_> = graph.neighbors(node_index).collect();
            for &neighbor in neighbors.iter().rev() {
                if visited.visit(neighbor) {
                    stack.push((neighbor, Some(node_index)));
                }
            }
        }

        let mut semi_nca_tree = SemiNCASpanningTree {
            root_id: start,
            spanning_tree_node_info,
            source_graph_indexes_in_dfs_order: nodes_in_dfs_order,
        };
        semi_nca_tree.compute_semidominators(graph);
        semi_nca_tree.compute_immediate_dominators();
        semi_nca_tree
    }

    pub fn build_dominator_tree<'a>(&self) -> DominatorTree<Ix> {
        let mut graph = StableDiGraph::new();
        let dominator_tree_nodes_by_source_graph_index = self
            .source_graph_indexes_in_dfs_order
            .iter()
            .map(|&source_graph_index| {
                (
                    source_graph_index,
                    graph.add_node(source_graph_index.into()),
                )
            })
            .collect::<HashMap<_, _>>();

        for source_graph_index in self.source_graph_indexes_in_dfs_order.iter() {
            let node_info = &self.spanning_tree_node_info[&source_graph_index];
            let dt_node = dominator_tree_nodes_by_source_graph_index[source_graph_index];
            let idom = node_info.idom;

            if let Some(idom) = idom {
                graph.add_edge(
                    dominator_tree_nodes_by_source_graph_index[&idom],
                    dt_node,
                    DominatorTreeEdgeType::ImmediateDominator,
                );
            }
        }

        DominatorTree {
            graph,
            root_id: dominator_tree_nodes_by_source_graph_index[&self.root_id],
            dominator_tree_index_by_source_graph_index: dominator_tree_nodes_by_source_graph_index,
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

    fn compute_semidominators<'a, G>(&mut self, graph: &'a G)
    where
        &'a G: GraphBase<NodeId = NodeIndex<Ix>, EdgeId = EdgeIndex<Ix>> + IntoEdgesDirected,
    {
        // iterate nodes in reverse DFS order, omitting the root
        let reverse_dfs_order_without_root = self.source_graph_indexes_in_dfs_order[1..]
            .iter()
            .rev()
            .cloned()
            .collect::<Vec<_>>();

        for &node in reverse_dfs_order_without_root.iter() {
            // we know there will be a parent because we're omitting the root
            let parent = self.spanning_tree_node_info[&node].parent.unwrap();
            let mut semi = parent;

            for inward_edge in graph.edges_directed(node, Direction::Incoming) {
                let predecessor_index = inward_edge.source();
                let node_dfs_num = self.spanning_tree_node_info[&node].dfs_num;
                if self
                    .spanning_tree_node_info
                    .get(&predecessor_index)
                    .is_none()
                {
                    panic!("No node for {:?}", predecessor_index);
                }
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
        let dfs_order_without_root = self.source_graph_indexes_in_dfs_order[1..]
            .iter()
            .cloned()
            .collect::<Vec<_>>();

        for &node in dfs_order_without_root.iter() {
            // we know there's an idom for this node because we initialize it to the parent for all
            // nodes (except the root, for which there's no parent)
            let mut idom = self.spanning_tree_node_info[&node].idom.unwrap();
            let sdom = self.spanning_tree_node_info[&node].sdom.unwrap();
            let sdom_dfs_num = self.spanning_tree_node_info[&sdom].dfs_num;
            let mut visited = HashSet::new();

            while self.spanning_tree_node_info[&idom].dfs_num > sdom_dfs_num
                && !visited.contains(&idom)
            {
                visited.insert(idom);
                idom = self.spanning_tree_node_info[&idom].idom.unwrap();
            }

            if self.spanning_tree_node_info[&idom].dfs_num <= sdom_dfs_num {
                self.spanning_tree_node_info.get_mut(&node).unwrap().idom = Some(idom);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DominatorTree;
    use petgraph::prelude::StableDiGraph;

    #[test]
    fn test_simple_dominator_tree() {
        let mut graph = StableDiGraph::<(), ()>::new();
        let n0 = graph.add_node(());
        let n1 = graph.add_node(());
        let n2 = graph.add_node(());
        let n3 = graph.add_node(());
        let n4 = graph.add_node(());

        graph.extend_with_edges(&[(n0, n1), (n0, n2), (n1, n3), (n2, n3), (n2, n4)]);

        let dominator_tree = DominatorTree::from_graph(&graph, n0);
        assert!(dominator_tree.dominates(n0, n1));
        assert!(dominator_tree.dominates(n0, n2));
        assert!(dominator_tree.dominates(n0, n3));
        assert!(dominator_tree.dominates(n2, n4));
        assert!(!dominator_tree.dominates(n1, n2));
    }
}
