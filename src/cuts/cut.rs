#[derive(Debug, Clone, PartialEq)]
pub struct Cut where {
    pub source_set: Vec<usize>,
    pub destination_set: Vec<usize>,
    pub cut_edge_set: Vec<usize>,
    pub size: usize,
}

impl Cut {
    pub fn new(source_set: Vec<usize>,
               destination_set: Vec<usize>,
               cut_edge_set: Vec<usize>) -> Self {
        let size = cut_edge_set.len();
        Self {
            source_set,
            destination_set,
            cut_edge_set,
            size,
        }
    }
}
