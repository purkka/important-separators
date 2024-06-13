mod visualization;

use petgraph::algo::ford_fulkerson;
use petgraph::Graph;

fn main() {
    // Example from CLRS book
    let mut graph = Graph::<(), u8>::new();
    let source = graph.add_node(());  // 0
    let _ = graph.add_node(());
    let _ = graph.add_node(());
    let _ = graph.add_node(());
    let _ = graph.add_node(());
    let destination = graph.add_node(());  // 5
    graph.extend_with_edges(&[
        (0, 1, 16),  // 0
        (0, 2, 13),  // 1
        (1, 2, 10),  // 2
        (1, 3, 12),  // 3
        (2, 4, 14),  // 4
        (3, 2, 9),  // 5
        (3, 5, 20),  // 6
        (4, 3, 7),  // 7
        (4, 5, 4),  // 8
    ]);
    let (max_flow, _) = ford_fulkerson(&graph, source, destination);
    assert_eq!(23, max_flow);

    visualization::draw_graph(graph);
}
