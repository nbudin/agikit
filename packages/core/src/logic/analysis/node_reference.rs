use petgraph::{
    EdgeType,
    csr::{DefaultIx, IndexType},
    graph::NodeIndex,
    prelude::StableGraph,
    visit::NodeRef,
};

#[derive(Clone, Copy, Debug)]
pub struct NodeReference<Ix: IndexType = DefaultIx>(NodeIndex<Ix>);

impl<Ix: IndexType> NodeReference<Ix> {
    pub fn id(&self) -> NodeIndex<Ix> {
        self.0.id()
    }

    pub fn index(&self) -> usize {
        self.0.index()
    }
}

impl<'a, Ix: IndexType> From<&'a NodeReference<Ix>> for NodeReference<Ix> {
    fn from(value: &'a NodeReference<Ix>) -> Self {
        *value
    }
}

impl<Ix: IndexType> From<NodeIndex<Ix>> for NodeReference<Ix> {
    fn from(value: NodeIndex<Ix>) -> Self {
        Self(value)
    }
}

impl<Ix: IndexType> From<usize> for NodeReference<Ix> {
    fn from(value: usize) -> Self {
        Self(NodeIndex::new(value))
    }
}

impl<Ix: IndexType> From<NodeReference<Ix>> for NodeIndex<Ix> {
    fn from(value: NodeReference<Ix>) -> Self {
        value.id()
    }
}

impl<Ix: IndexType> From<NodeReference<Ix>> for usize {
    fn from(value: NodeReference<Ix>) -> Self {
        value.index()
    }
}

pub trait ReferenceGraph<RN, RE, Ty: EdgeType, Ix: IndexType = DefaultIx>
where
    for<'a> &'a RN: TryInto<NodeReference<Ix>>,
{
    fn reference_graph(&self) -> &StableGraph<RN, RE, Ty>;

    fn source_node_id_for_reference_node_id(
        &self,
        reference_node_id: NodeIndex,
    ) -> Option<NodeIndex<Ix>> {
        self.reference_graph()
            .node_weight(reference_node_id)
            .and_then(|rn| {
                rn.try_into()
                    .ok()
                    .map(|node_reference: NodeReference<Ix>| node_reference.id())
            })
    }

    fn reference_node_id_for_source_node_id(
        &self,
        source_node_id: NodeIndex<Ix>,
    ) -> Option<NodeIndex>;
}
