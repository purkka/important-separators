mod visualization;
mod cuts;

use petgraph::graph::UnGraph;

fn main() {
    // Example from CLRS book
    let mut graph = UnGraph::<(), ()>::new_undirected();
    let source = graph.add_node(());  // 0
    let _ = graph.add_node(());
    let _ = graph.add_node(());
    let _ = graph.add_node(());
    let _ = graph.add_node(());
    let destination = graph.add_node(());  // 5
    graph.extend_with_edges(&[
        (0, 1, ()),  // 0
        (0, 2, ()),  // 1
        (1, 2, ()),  // 2
        (1, 3, ()),  // 3
        (2, 4, ()),  // 4
        (3, 2, ()),  // 5
        (3, 5, ()),  // 6
        (4, 3, ()),  // 7
        (4, 5, ()),  // 8
    ]);

    let cuts = cuts::generate_cuts(&graph, source, destination, 3);
    println!("{:?}", cuts);

    // For now, draw each cut separately
    for cut in cuts {
        visualization::draw_graph(graph.clone(), cut);
    }
}
