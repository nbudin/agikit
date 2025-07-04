use petgraph::{
    Direction,
    graph::{DiGraph, EdgeIndex, NodeIndex},
    visit::{
        Dfs, EdgeIndexable, EdgeRef, GraphBase, GraphRef, IntoEdgeReferences, IntoEdgesDirected,
        IntoNeighbors, NodeIndexable, Visitable, Walker,
    },
};

pub trait DirectedNeighborEdgeUtils<EdgeType>: GraphBase + petgraph::visit::Data
where
    for<'a> &'a Self: IntoEdgesDirected + NodeIndexable,
    for<'a> <&'a Self as petgraph::visit::Data>::EdgeWeight: PartialEq<EdgeType>,
{
    fn directed_neighbor_edge_of_type<'a>(
        &'a self,
        node_id: <&'a Self as GraphBase>::NodeId,
        direction: Direction,
        edge_type: EdgeType,
    ) -> Option<<&'a Self as IntoEdgeReferences>::EdgeRef> {
        self.edges_directed(node_id, direction)
            .find(|edge| edge.weight() == &edge_type)
    }

    fn directed_neighbor_edge_id_of_type<'a>(
        &'a self,
        node_id: <&'a Self as GraphBase>::NodeId,
        direction: Direction,
        edge_type: EdgeType,
    ) -> Option<<&'a Self as GraphBase>::EdgeId> {
        self.directed_neighbor_edge_of_type(node_id, direction, edge_type)
            .map(|edge| edge.id())
    }

    fn directed_neighbor_node_id_of_type<'a>(
        &'a self,
        node_id: <&'a Self as GraphBase>::NodeId,
        direction: Direction,
        edge_type: EdgeType,
    ) -> Option<<&'a Self as GraphBase>::NodeId> {
        let Some(edge_ref) = self.directed_neighbor_edge_of_type(node_id, direction, edge_type)
        else {
            return None;
        };

        Some(match direction {
            Direction::Outgoing => edge_ref.target(),
            Direction::Incoming => edge_ref.source(),
        })
    }
}

impl<EdgeType, T> DirectedNeighborEdgeUtils<EdgeType> for T
where
    T: GraphBase + petgraph::visit::Data,
    for<'a> &'a T: IntoEdgesDirected + NodeIndexable,
    for<'a> <&'a T as petgraph::visit::Data>::EdgeWeight: PartialEq<EdgeType>,
{
}

pub trait RemoveNodePreservingEdges<NodeIndexType, EdgeIndexType, EdgeWeight> {
    fn incoming_edge_data(
        &self,
        node_id: NodeIndexType,
    ) -> Vec<(EdgeIndexType, NodeIndexType, EdgeWeight)>;
    fn outgoing_edge_data(
        &self,
        node_id: NodeIndexType,
    ) -> Vec<(EdgeIndexType, NodeIndexType, EdgeWeight)>;

    fn remove_node_preserving_edges(
        &mut self,
        node_id: NodeIndexType,
        new_target_id: NodeIndexType,
    );
}

impl<'a, NodeType: 'a, EdgeType: 'a + Clone>
    RemoveNodePreservingEdges<NodeIndex, EdgeIndex, EdgeType> for DiGraph<NodeType, EdgeType>
where
    &'a DiGraph<NodeType, EdgeType>: IntoEdgesDirected + NodeIndexable + EdgeIndexable,
{
    fn incoming_edge_data(&self, node_id: NodeIndex) -> Vec<(EdgeIndex, NodeIndex, EdgeType)> {
        self.edges_directed(node_id, Direction::Incoming)
            .map(|edge| (edge.id(), edge.source(), edge.weight().clone()))
            .collect::<Vec<_>>()
    }

    fn outgoing_edge_data(&self, node_id: NodeIndex) -> Vec<(EdgeIndex, NodeIndex, EdgeType)> {
        self.edges_directed(node_id, Direction::Outgoing)
            .map(|edge| (edge.id(), edge.target(), edge.weight().clone()))
            .collect::<Vec<_>>()
    }

    fn remove_node_preserving_edges(&mut self, node_index: NodeIndex, new_target_id: NodeIndex) {
        let incoming_edges = self.incoming_edge_data(node_index);
        let outgoing_edges = self.outgoing_edge_data(node_index);

        for (edge_id, source_id, edge_weight) in incoming_edges {
            if source_id != new_target_id {
                self.add_edge(source_id, new_target_id, edge_weight);
            }
            self.remove_edge(edge_id);
        }

        for (edge_id, target_id, edge_weight) in outgoing_edges {
            if new_target_id != target_id {
                self.add_edge(new_target_id, target_id, edge_weight);
            }
            self.remove_edge(edge_id);
        }

        self.remove_node(node_index);
    }
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
    fn optimization_visitors(&self) -> Vec<Box<dyn OptimizationVisitor<G>>>;

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
        let mut visitors = Self::optimization_visitors(&self);
        self.run_optimization_passes(&mut visitors);
    }
}
