mod cuts;
mod visualization;

use petgraph::graph::UnGraph;

fn main() {
    // Example from CLRS book
    let mut graph = UnGraph::<(), ()>::new_undirected();
    let source = graph.add_node(()); // 0
    let _ = graph.add_node(());
    let _ = graph.add_node(());
    let _ = graph.add_node(());
    let _ = graph.add_node(());
    let destination = graph.add_node(()); // 5
    graph.extend_with_edges(&[
        (0, 1, ()), // 0
        (0, 2, ()), // 1
        (1, 2, ()), // 2
        (1, 3, ()), // 3
        (2, 4, ()), // 4
        (3, 2, ()), // 5
        (3, 5, ()), // 6
        (4, 3, ()), // 7
        (4, 5, ()), // 8
    ]);

    let k = 3usize;
    let cuts = cuts::generate_cuts(&graph, source, destination, k);
    println!("Cuts with k = {}: {:?}", k, cuts);

    let important_cuts = cuts::filter_important_cuts(&cuts);
    println!("Important cuts: {:?}", important_cuts);

    // For now, draw each cut separately
    for cut in important_cuts {
        visualization::draw_graph(graph.clone(), cut);
    }
}
