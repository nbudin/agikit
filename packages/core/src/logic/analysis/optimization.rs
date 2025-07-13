use petgraph::{
    Direction,
    graph::{EdgeIndex, NodeIndex},
    prelude::StableDiGraph,
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

    fn remove_node_preserving_edges<F: Fn(&Self, EdgeIndexType) -> bool>(
        &mut self,
        node_id: NodeIndexType,
        new_source_id: NodeIndexType,
        new_target_id: NodeIndexType,
        preserve_edge: F,
    );
}

impl<'a, NodeType: 'a, EdgeType: 'a + Clone>
    RemoveNodePreservingEdges<NodeIndex, EdgeIndex, EdgeType> for StableDiGraph<NodeType, EdgeType>
where
    &'a StableDiGraph<NodeType, EdgeType>: IntoEdgesDirected + NodeIndexable + EdgeIndexable,
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

    fn remove_node_preserving_edges<F: Fn(&Self, EdgeIndex) -> bool>(
        &mut self,
        node_index: NodeIndex,
        new_source_id: NodeIndex,
        new_target_id: NodeIndex,
        preserve_edge: F,
    ) {
        let incoming_edges = self.incoming_edge_data(node_index);
        let outgoing_edges = self.outgoing_edge_data(node_index);

        for (edge_id, source_id, edge_weight) in incoming_edges {
            if preserve_edge(self, edge_id) {
                self.add_edge(source_id, new_target_id, edge_weight);
            }
            self.remove_edge(edge_id);
        }

        for (edge_id, target_id, edge_weight) in outgoing_edges {
            if preserve_edge(self, edge_id) {
                self.add_edge(new_source_id, target_id, edge_weight);
            }
            self.remove_edge(edge_id);
        }

        self.remove_node(node_index);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationResult {
    Unchanged = 0,
    Changed = 1,
}

impl OptimizationResult {
    pub fn is_changed(&self) -> bool {
        *self == OptimizationResult::Changed
    }

    // Rust doesn't currently allow overloading || via a trait
    pub fn or(&self, other: &OptimizationResult) -> OptimizationResult {
        if self.is_changed() || other.is_changed() {
            OptimizationResult::Changed
        } else {
            OptimizationResult::Unchanged
        }
    }
}

pub trait OptimizationVisitor<G: GraphBase> {
    fn visit(&mut self, graph: &mut G, node_id: G::NodeId) -> OptimizationResult;
}

impl<G: GraphBase, F: FnMut(&mut G, G::NodeId) -> OptimizationResult> OptimizationVisitor<G> for F {
    fn visit(&mut self, graph: &mut G, node_id: G::NodeId) -> OptimizationResult {
        self(graph, node_id)
    }
}

pub trait OptimizationPass<G: GraphBase> {
    fn run<'a>(&'a mut self, graph: &'a mut G, root_id: G::NodeId) -> OptimizationResult;
}

impl<G: GraphBase, V: OptimizationVisitor<G>> OptimizationPass<G> for V
where
    for<'a> &'a G: GraphRef + NodeIndexable + Visitable + IntoNeighbors,
    for<'a> <&'a G as GraphBase>::NodeId: From<G::NodeId> + Into<G::NodeId>,
{
    fn run<'a>(&'a mut self, graph: &'a mut G, root_id: G::NodeId) -> OptimizationResult {
        let queue = {
            Dfs::new(graph as &G, root_id.into())
                .iter(graph as &G)
                .map(|node_id| node_id.into())
                .collect::<Vec<G::NodeId>>()
        };

        let mut changed = OptimizationResult::Unchanged;
        for node_id in queue {
            changed = changed.or(&self.visit(graph, node_id));
        }
        changed
    }
}

pub trait Optimizable<G: GraphBase>
where
    Self: Sized,
    for<'a> &'a G: GraphRef + NodeIndexable + Visitable + IntoNeighbors,
    for<'a> <&'a G as GraphBase>::NodeId: Into<G::NodeId>,
{
    fn get_graph(&self) -> &G;
    fn get_graph_mut(&mut self) -> &mut G;
    fn root_id(&self) -> <&G as GraphBase>::NodeId;
    fn optimization_passes(&self) -> Vec<Box<dyn OptimizationPass<G>>>;

    fn run_optimization_passes_once(
        &mut self,
        passes: &mut [Box<dyn OptimizationPass<G>>],
    ) -> OptimizationResult {
        let mut result = OptimizationResult::Unchanged;
        let root_id = self.root_id().into();
        for pass in &mut *passes {
            result = result.or(&pass.run(self.get_graph_mut(), root_id));
        }
        result
    }

    fn optimize(&mut self) -> OptimizationResult {
        let mut keep_going = true;
        let mut overall_result = OptimizationResult::Unchanged;

        while keep_going {
            let mut passes = Self::optimization_passes(&self);
            let iteration_result = self.run_optimization_passes_once(&mut passes);
            overall_result = overall_result.or(&iteration_result);
            keep_going = iteration_result.is_changed();
        }

        overall_result
    }
}
