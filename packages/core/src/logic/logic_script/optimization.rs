use petgraph::{
    csr::{DefaultIx, IndexType},
    graph::NodeIndex,
    visit::{
        Dfs, EdgeRef, GraphBase, GraphRef, IntoEdgesDirected, IntoNeighbors, NodeIndexable,
        Visitable, Walker,
    },
    Direction,
};

pub trait DirectedNeighborEdgeUtils<NodeType, EdgeType, Ix: IndexType = DefaultIx>:
    GraphBase + petgraph::visit::Data
where
    for<'a> &'a Self: IntoEdgesDirected + NodeIndexable,
    for<'a> <&'a Self as petgraph::visit::Data>::EdgeWeight: PartialEq<EdgeType>,
{
    fn directed_neighbor_edge_id_of_type(
        &self,
        node_id: NodeIndex<Ix>,
        direction: Direction,
        edge_type: EdgeType,
    ) -> Option<<&Self as GraphBase>::EdgeId> {
        self.edges_directed(self.from_index(node_id.index()), direction)
            .find_map(|edge| {
                if edge.weight() == &edge_type {
                    Some(edge.id())
                } else {
                    None
                }
            })
    }
}

impl<NodeType, EdgeType, Ix: IndexType, T> DirectedNeighborEdgeUtils<NodeType, EdgeType, Ix> for T
where
    T: GraphBase + petgraph::visit::Data,
    for<'a> &'a T: IntoEdgesDirected + NodeIndexable,
    for<'a> <&'a T as petgraph::visit::Data>::EdgeWeight: PartialEq<EdgeType>,
{
}

pub trait OptimizationVisitor<G: GraphBase> {
    fn visit(&mut self, graph: &mut G, node_id: G::NodeId) -> bool;
}

impl<G: GraphBase, F: FnMut(&mut G, G::NodeId) -> bool> OptimizationVisitor<G> for F {
    fn visit(&mut self, graph: &mut G, node_id: G::NodeId) -> bool {
        self(graph, node_id)
    }
}

pub trait Optimizable<G: GraphBase>
where
    Self: Sized,
    for<'a> &'a G: GraphRef + NodeIndexable + Visitable + IntoNeighbors,
    for<'a> <&'a G as GraphBase>::NodeId: ToOwned<Owned = <G as GraphBase>::NodeId>,
{
    fn get_graph(&self) -> &G;
    fn get_graph_mut(&mut self) -> &mut G;
    fn root_id(&self) -> <&G as GraphBase>::NodeId;
    fn optimization_visitors() -> Vec<Box<dyn OptimizationVisitor<G>>>;

    fn run_optimization_pass(&mut self, visitor: &mut dyn OptimizationVisitor<G>) -> bool {
        let queue = {
            let graph = self.get_graph();
            Dfs::new(graph, self.root_id())
                .iter(&graph)
                .map(|block_id| block_id.to_owned())
                .collect::<Vec<_>>()
        };

        let mut changed = false;
        let graph = self.get_graph_mut();
        for block_id in queue {
            if visitor.visit(graph, block_id) {
                changed = true;
            }
        }
        changed
    }

    fn run_optimization_passes(&mut self, visitors: &mut [Box<dyn OptimizationVisitor<G>>]) -> () {
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

    fn optimize(&mut self) {
        let mut visitors = Self::optimization_visitors();
        self.run_optimization_passes(&mut visitors);
    }
}
