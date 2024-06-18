use petgraph::visit::{EdgeIndexable, NodeIndexable};

#[derive(Debug)]
pub struct Cut<G> where G: EdgeIndexable + NodeIndexable {
    pub source_set: Vec<G::NodeId>,
    pub destination_set: Vec<G::NodeId>,
    pub cut_set: Vec<G::EdgeId>,
    pub size: usize,
}

impl<G> Cut<G> where G: EdgeIndexable + NodeIndexable {
    pub fn new(source_set: Vec<G::NodeId>,
               destination_set: Vec<G::NodeId>,
               cut_set: Vec<G::EdgeId>) -> Self {
        let size = cut_set.len();
        Self {
            source_set,
            destination_set,
            cut_set,
            size,
        }
    }
}

impl<G> PartialEq for Cut<G> where G: EdgeIndexable + NodeIndexable {
    fn eq(&self, other: &Self) -> bool {
        self.source_set == other.source_set && self.destination_set == other.destination_set && self.cut_set == other.cut_set
    }
}
