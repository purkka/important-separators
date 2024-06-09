use petgraph::algo::ford_fulkerson;
use petgraph::Graph;

fn main() {
    // Example from CLRS book
    let mut graph = Graph::<(), u8>::new();
    let source = graph.add_node(());
    let _ = graph.add_node(());
    let _ = graph.add_node(());
    let _ = graph.add_node(());
    let _ = graph.add_node(());
    let destination = graph.add_node(());
    graph.extend_with_edges(&[
        (0, 1, 16),
        (0, 2, 13),
        (1, 2, 10),
        (1, 3, 12),
        (2, 1, 4),
        (2, 4, 14),
        (3, 2, 9),
        (3, 5, 20),
        (4, 3, 7),
        (4, 5, 4),
    ]);
    let (max_flow, _) = ford_fulkerson(&graph, source, destination);
    assert_eq!(23, max_flow);
}
