use petgraph::prelude::EdgeRef;
use petgraph::visit::{EdgeIndexable, IntoEdgeReferences, NodeIndexable};

use crate::cuts::cut::{generate_minimum_cut_closest_to_destination, ImportantCut};
use crate::cuts::path_residual::{get_augmenting_paths_and_residual_graph_for_sets, Path, UnGraph};

fn important_cuts<G>(
    original_graph: G,
    source_set: Vec<usize>,
    destination_set: Vec<usize>,
    k: usize,
) -> Vec<ImportantCut>
where
    G: NodeIndexable + IntoEdgeReferences,
{
    fn important_cut_inner(
        graph: &UnGraph,
        source_set: Vec<usize>,
        destination_set: Vec<usize>,
        k: usize,
        edges_in_cut: Vec<usize>,
        important_cuts: &mut Vec<ImportantCut>,
    ) {
        match get_augmenting_paths_and_residual_graph_for_sets(
            &graph,
            source_set,
            destination_set,
            k,
        ) {
            Some((new_graph, paths, residual)) => {
                let min_cut = generate_minimum_cut_closest_to_destination(&paths, residual);
                // TODO Report C u Z, current implementation just for test purposes
                important_cuts.push(ImportantCut::from(min_cut.cut_edge_set));

                // return branch if k == 0 or if the min cut is of size k
                if k == 0 || min_cut.cut_edge_set.len() == k {
                    return;
                }

                // pick arbitrary edge from cut
                let (edge, vertex_in_dest) = min_cut.arbitrary_edge(&graph);

                // branch into two cases
                // 1. the arbitrary edge is *not* part of the important cut
                let new_graph_destination = Path::get_destination(&paths);
                important_cut_inner(
                    &new_graph,
                    [min_cut.source_set.clone(), vec![vertex_in_dest]].concat(),
                    vec![new_graph_destination],
                    k,
                    edges_in_cut.clone(),
                    important_cuts,
                );
                // 2. the arbitrary edge is part of the important cut
                let mut new_graph_without_edge = new_graph.clone();
                new_graph_without_edge
                    .remove_edge(EdgeIndexable::from_index(&new_graph_without_edge, edge));
                important_cut_inner(
                    &new_graph_without_edge,
                    min_cut.source_set,
                    vec![new_graph_destination],
                    k - 1,
                    [edges_in_cut, vec![edge]].concat(),
                    important_cuts,
                );
            }
            None => {
                return;
            }
        }
    }

    let mut cuts = vec![];

    let original_graph_edges = original_graph.edge_references().map(|edge| {
        let source_index = NodeIndexable::to_index(&original_graph, edge.source());
        let target_index = NodeIndexable::to_index(&original_graph, edge.target());
        (source_index, target_index)
    });

    let original_graph_as_un_graph = UnGraph::from_edges(original_graph_edges);

    important_cut_inner(
        &original_graph_as_un_graph,
        source_set,
        destination_set,
        k,
        vec![],
        &mut cuts,
    );

    cuts
}
