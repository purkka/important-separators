use crate::cuts::Cut;
use petgraph::prelude::Bfs;
use petgraph::visit::{
    EdgeIndexable, EdgeRef, IntoEdges, IntoNeighbors, IntoNodeReferences, NodeCount, NodeIndexable,
    Visitable,
};

/// Get cuts between `source` and `destination` of size at most `k`
pub fn generate_cuts<G>(graph: G, source: G::NodeId, destination: G::NodeId, k: usize) -> Vec<Cut>
where
    G: EdgeIndexable
        + NodeIndexable
        + Visitable
        + NodeCount
        + IntoNodeReferences
        + IntoNeighbors
        + IntoEdges,
{
    let mut ret: Vec<Cut> = vec![];

    // TODO Consider improving used data structure
    let mut visited: Vec<usize> = vec![];

    // Traverse nodes using BFS
    let mut bfs = Bfs::new(&graph, source);
    while let Some(node) = bfs.next(&graph) {
        if node != destination {
            // never mark the destination as visited
            visited.push(NodeIndexable::to_index(&graph, node));
        }

        let mut cut_edges: Vec<usize> = vec![];
        // TODO Maybe we don't have to go through every edge of every node here?
        for &visited_node in visited.iter() {
            for edge in graph.edges(NodeIndexable::from_index(&graph, visited_node)) {
                let edge_source_id = NodeIndexable::to_index(&graph, edge.source());
                let edge_target_id = NodeIndexable::to_index(&graph, edge.target());
                // We add the edge to the cut if one of its endpoints is visited and the other is not
                if visited.contains(&edge_source_id) ^ visited.contains(&edge_target_id) {
                    let edge_id = EdgeIndexable::to_index(&graph, edge.id());
                    if !cut_edges.contains(&edge_id) {
                        cut_edges.push(edge_id);
                    }
                }
            }
        }

        if cut_edges.len() <= k {
            let dest_set = (0usize..graph.node_count())
                .filter(|n| !visited.contains(&n))
                .collect();
            let cut = Cut::new(visited.clone(), dest_set, cut_edges);
            if !ret.contains(&cut) {
                ret.push(cut);
            }
        }
    }

    ret
}

pub fn filter_important_cuts(cuts: &Vec<Cut>) -> Vec<Cut> {
    // TODO Consider writing this a bit nicer using combinations or something similar
    cuts.iter()
        .filter(|&cut_i| {
            cuts.iter().any(|cut_j| {
                cut_j.size <= cut_i.size && cut_j.source_set.len() < cut_i.source_set.len()
            })
        })
        .map(|c| c.clone())
        .collect()
}
