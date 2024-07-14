mod cuts;
mod visualization;

use crate::cuts::ImportantCut;
use petgraph::graph::UnGraph;

fn main() {
    let graph = UnGraph::<(), ()>::from_edges(&[(0, 1), (0, 2), (1, 3), (1, 4), (2, 5), (2, 6)]);
    let source_set = vec![0];
    let destination_set = vec![3, 4, 5, 6];
    let k = 3;

    // note that these cuts are 'unfiltered', so there are some extra elements in the result
    let important_cuts = cuts::important_cuts(&graph, source_set, destination_set, k);
    ImportantCut::print_important_cuts(&graph, important_cuts);

    // TODO Fix visualization and add here
}
