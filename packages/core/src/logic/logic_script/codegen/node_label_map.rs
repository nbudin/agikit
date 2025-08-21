use std::{
    collections::{HashMap, hash_map::Entry},
    hash::Hash,
};

use petgraph::{
    csr::DefaultIx,
    data::DataMap,
    prelude::NodeIndex,
    visit::{Data, Dfs, GraphBase, GraphRef, IntoNeighbors, Visitable},
};

pub trait LabeledNode {
    fn label(&self) -> Option<&str>;
    fn set_label(&mut self, label: Option<&str>);
}

#[derive(Debug, Clone)]
pub enum GetOrInsertLabelResult {
    GotExisting(String),
    Inserted(String),
}

impl GetOrInsertLabelResult {
    pub fn label(&self) -> &str {
        match self {
            GetOrInsertLabelResult::GotExisting(label) => label.as_str(),
            GetOrInsertLabelResult::Inserted(label) => label.as_str(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NodeLabelMap<Ix: Eq + Copy + Hash = DefaultIx> {
    labels_by_node_id: HashMap<NodeIndex<Ix>, String>,
    node_id_by_label: HashMap<String, NodeIndex<Ix>>,
    generated_label_counter: u16,
}

impl<Ix: Eq + Copy + Hash> NodeLabelMap<Ix> {
    pub fn new<'a, G, W: LabeledNode>(graph: &'a G, root_id: NodeIndex<Ix>) -> Self
    where
        &'a G: GraphBase<NodeId = NodeIndex<Ix>>
            + GraphRef
            + Visitable
            + IntoNeighbors
            + Data<NodeWeight = W>
            + DataMap,
    {
        let mut labels_by_node_id = HashMap::new();
        let mut node_id_by_label = HashMap::new();

        let mut dfs = Dfs::new(graph, root_id);
        while let Some(node_id) = dfs.next(&graph) {
            let Some(node) = graph.node_weight(node_id) else {
                continue;
            };

            let Some(label) = node.label() else {
                continue;
            };

            labels_by_node_id.insert(node_id, label.to_string());
            node_id_by_label.insert(label.to_string(), node_id);
        }

        Self {
            labels_by_node_id,
            node_id_by_label,
            generated_label_counter: 0,
        }
    }

    pub fn get_node_id_for_label(&self, label: &str) -> Option<NodeIndex<Ix>> {
        self.node_id_by_label.get(label).copied()
    }

    pub fn get_label_for_node_id(&self, node_id: NodeIndex<Ix>) -> Option<&str> {
        self.labels_by_node_id.get(&node_id).map(|s| s.as_str())
    }

    pub fn get_or_insert_label_for_node_id(
        &mut self,
        node_id: NodeIndex<Ix>,
    ) -> GetOrInsertLabelResult {
        let entry = self.labels_by_node_id.entry(node_id);

        match entry {
            Entry::Occupied(entry) => GetOrInsertLabelResult::GotExisting(entry.get().to_string()),
            Entry::Vacant(entry) => {
                let label = loop {
                    let label = format!("GeneratedLabel{}", self.generated_label_counter);
                    self.generated_label_counter += 1;
                    if !self.node_id_by_label.contains_key(&label) {
                        break label;
                    }
                };

                self.node_id_by_label.insert(label.clone(), node_id);
                entry.insert(label.clone());
                GetOrInsertLabelResult::Inserted(label)
            }
        }
    }

    pub fn remove_label_for_node_id(&mut self, node_id: NodeIndex<Ix>) {
        let entry = self.labels_by_node_id.entry(node_id);
        match entry {
            Entry::Occupied(entry) => {
                let (_, label) = entry.remove_entry();
                self.node_id_by_label.remove_entry(&label);
            }
            Entry::Vacant(_) => {}
        }
    }
}
