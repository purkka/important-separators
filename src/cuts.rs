use petgraph::visit::{Bfs, EdgeIndexable, EdgeRef, IntoEdges, IntoNeighbors, NodeIndexable, Visitable};

/// Get cuts between `source` and `destination` of size at most `k`
pub fn generate_cuts<G>(graph: G,
                        source: G::NodeId,
                        destination: G::NodeId,
                        k: usize) -> Vec<Vec<G::EdgeId>> where G: EdgeIndexable + NodeIndexable + IntoNeighbors + IntoEdges + Visitable {
    // TODO Instead of cut edges, return cut with division of vertices as well as edges in the cut
    let mut ret: Vec<Vec<G::EdgeId>> = vec![];

    // TODO Consider improving used data structure
    let mut visited = vec![source];

    // Traverse nodes using BFS
    let mut bfs = Bfs::new(&graph, source);
    while let Some(node) = bfs.next(&graph) {
        let mut cut_edges: Vec<G::EdgeId> = vec![];
        // TODO Maybe we don't have to go through every edge of every node here?
        for &visited_node in visited.iter() {
            for edge in graph.edges(visited_node) {
                // We add the edge to the cut if one of its endpoints is visited and the other is not
                if visited.contains(&edge.source()) ^ visited.contains(&edge.target()) {
                    let edge_id = edge.id();
                    if !cut_edges.contains(&edge_id) {
                        cut_edges.push(edge_id);
                    }
                }
            }
        }

        if cut_edges.len() <= k && !ret.contains(&cut_edges) {
            ret.push(cut_edges);
        }

        if node != destination {  // never mark the destination as visited
            visited.push(node);
        }
    }

    ret
}
