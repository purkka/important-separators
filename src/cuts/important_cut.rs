use petgraph::prelude::EdgeRef;
use petgraph::visit::{IntoEdgeReferences, NodeIndexable};

use crate::cuts::cut::{generate_minimum_cut_closest_to_destination_with_mapping, ImportantCut};
use crate::cuts::path_residual::{get_augmenting_paths_and_residual_graph_for_sets, UnGraph};

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
        original_graph: &UnGraph,
        source_set: Vec<usize>,
        destination_set: Vec<usize>,
        k: usize,
        edges_in_use: Vec<bool>,
        edges_in_cut: Vec<usize>,
        important_cuts: &mut Vec<ImportantCut>,
    ) {
        match get_augmenting_paths_and_residual_graph_for_sets(
            &original_graph,
            source_set,
            destination_set.clone(),
            k,
            &edges_in_use,
        ) {
            Some((paths, residual, index_mapping, edge_weights)) => {
                let min_cut = generate_minimum_cut_closest_to_destination_with_mapping(
                    &paths,
                    residual,
                    index_mapping,
                    edge_weights,
                );

                // Report C u Z
                important_cuts.push(ImportantCut::from(
                    [min_cut.cut_edge_set.clone(), edges_in_cut.clone()].concat(),
                ));

                // return branch if k == 0 or if the min cut is of size k
                if k == 0 || min_cut.size == k {
                    return;
                }

                // pick arbitrary edge from cut
                let (edge, destination_side_vertex) = min_cut.arbitrary_edge(&original_graph);

                // branch into two cases
                // 1. the arbitrary edge is *not* part of an important cut

                // the new source set is the source set of the min cut together with the destination
                // side vertex of our chosen edge
                important_cut_inner(
                    &original_graph,
                    [min_cut.source_set.clone(), vec![destination_side_vertex]].concat(),
                    destination_set.clone(),
                    k,
                    edges_in_use.clone(),
                    edges_in_cut.clone(),
                    important_cuts,
                );

                // 2. the arbitrary edge is part of an important cut

                // in this case we disable the edge by marking it not in use anymore
                let mut new_edges_in_use = edges_in_use.clone();
                new_edges_in_use[edge] = false;

                // the new source is the source set of the min cut, and now that we've added an edge
                // to an important cut, we reduce k by one
                important_cut_inner(
                    &original_graph,
                    min_cut.source_set,
                    destination_set.clone(),
                    k - 1,
                    new_edges_in_use,
                    [edges_in_cut, vec![edge]].concat(),
                    important_cuts,
                );
            }
            None => {
                // no more augmenting paths
                return;
            }
        }
    }

    let original_graph_edges = original_graph.edge_references().map(|edge| {
        let source_index = NodeIndexable::to_index(&original_graph, edge.source());
        let target_index = NodeIndexable::to_index(&original_graph, edge.target());
        (source_index, target_index)
    });

    let original_graph_as_un_graph = UnGraph::from_edges(original_graph_edges);

    let mut cuts = vec![];
    let initial_edges_in_use = vec![true; original_graph_as_un_graph.edge_count()];

    important_cut_inner(
        &original_graph_as_un_graph,
        source_set,
        destination_set,
        k,
        initial_edges_in_use,
        vec![],
        &mut cuts,
    );

    cuts
}
