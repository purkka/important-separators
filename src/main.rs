mod cuts;
mod visualization;

use petgraph::graph::UnGraph;
use crate::cuts::ImportantCut;

fn main() {
    let graph = UnGraph::<(), ()>::from_edges(&[
        (0, 1),
        (0, 2),
        (1, 2),
        (1, 3),
        (2, 4),
        (3, 2),
        (3, 5),
        (4, 3),
        (4, 5),
    ]);
    let source_set = vec![0, 1];
    let destination_set = vec![5];
    let k = 3;

    let important_cuts = cuts::important_cuts(&graph, source_set, destination_set, k);
    println!("Important cuts: {:?}", ImportantCut::vec_edge_indices(important_cuts));

    // TODO Fix visualization and add here
}
