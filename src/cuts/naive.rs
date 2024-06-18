use petgraph::prelude::{Bfs};
use petgraph::visit::{EdgeIndexable, EdgeRef, IntoEdges, IntoNeighbors, IntoNodeReferences, NodeIndexable, NodeRef, Visitable};
use crate::cuts::Cut;

/// Get cuts between `source` and `destination` of size at most `k`
pub fn generate_cuts<G>(graph: G,
                        source: G::NodeId,
                        destination: G::NodeId,
                        k: usize) -> Vec<Cut<G>> where G: EdgeIndexable + NodeIndexable + Visitable + IntoNodeReferences + IntoNeighbors + IntoEdges {
    let mut ret: Vec<Cut<G>> = vec![];

    // TODO Consider improving used data structure
    let mut visited: Vec<G::NodeId> = vec![];

    // Traverse nodes using BFS
    let mut bfs = Bfs::new(&graph, source);
    while let Some(node) = bfs.next(&graph) {
        if node != destination {  // never mark the destination as visited
            visited.push(node);
        }

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

        if cut_edges.len() <= k {
            let dest_set = graph.node_references().filter_map(|node| match visited.contains(&node.id()) {
                true => None,
                false => Some(node.id())
            }).collect::<Vec<G::NodeId>>();

            let cut = Cut::new(
                visited.clone(),
                dest_set,
                cut_edges,
            );
            if !ret.contains(&cut) {
                ret.push(cut);
            }
        }
    }

    ret
}

pub fn filter_important_cuts<G>(cuts: &Vec<Cut<G>>) -> Vec<&Cut<G>>
    where G: EdgeIndexable + NodeIndexable {
    // TODO Consider writing this a bit nicer using combinations or something similar
    cuts.into_iter().filter(|cut_i| {
        cuts.iter().any(|cut_j| {
            cut_j.size <= cut_i.size && cut_j.source_set.len() < cut_i.source_set.len()
        })
    }).collect()
}
