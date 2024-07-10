use petgraph::prelude::EdgeRef;
use petgraph::visit::{IntoEdgeReferences, NodeIndexable};

use crate::cuts::cut::{generate_minimum_cut_closest_to_destination_with_mapping, ImportantCut};
use crate::cuts::path_residual::{get_augmenting_paths_and_residual_graph_for_sets, UnGraph};

pub fn important_cuts<G>(
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
            Some((paths, residual, index_mapping)) => {
                let min_cut = generate_minimum_cut_closest_to_destination_with_mapping(
                    &paths,
                    residual,
                    index_mapping,
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

#[cfg(test)]
mod tests {
    use crate::cuts::cut::ImportantCut;
    use crate::cuts::important_cut::important_cuts;
    use crate::cuts::path_residual::UnGraph;

    #[test]
    fn simple_line() {
        let graph = UnGraph::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 4)]);
        let source = vec![0];
        let destination = vec![4];
        let k = 1;

        important_cuts(&graph, source, destination, k)
            .iter()
            .for_each(|imp_cut| {
                assert_eq!(1, imp_cut.edge_indices.len());
                assert_eq!(3, imp_cut.edge_indices[0]);
                assert_eq!((3, 4), imp_cut.vertex_pairs(&graph)[0]);
            });
    }

    fn all_contained(lhs: Vec<usize>, rhs: Vec<usize>) -> bool {
        lhs.iter().all(|elem| rhs.contains(elem))
    }

    fn all_contained_vec(lhs: Vec<Vec<usize>>, rhs: Vec<Vec<usize>>) -> bool {
        lhs.iter().all(|lhs_elem| {
            rhs.iter()
                .find(|&rhs_elem| all_contained(lhs_elem.clone(), rhs_elem.clone()))
                .is_some()
        })
    }

    #[test]
    fn simple_y_shape() {
        let graph = UnGraph::from_edges(&[(0, 1), (1, 2), (1, 3)]);
        let source = vec![0];
        let destination = vec![2, 3];

        // for k = 1
        let k1 = 1;

        let result_1 = important_cuts(&graph, source.clone(), destination.clone(), k1);
        let result_1_edges = ImportantCut::vec_edge_indices(result_1);

        let expected_important_cuts_1 = vec![vec![0]];
        assert!(all_contained_vec(expected_important_cuts_1, result_1_edges));

        // for k = 2
        let k2 = 2;

        let result_2 = important_cuts(&graph, source, destination, k2);
        let result_2_edges = ImportantCut::vec_edge_indices(result_2);

        let expected_important_cuts_2 = vec![vec![0], vec![1, 2]];
        assert!(all_contained_vec(expected_important_cuts_2, result_2_edges));
    }

    #[test]
    fn simple_binary_tree() {
        fn create_binary_tree(levels: usize) -> UnGraph {
            assert!(levels > 0);
            let mut edges = vec![];
            let total_nodes_with_children = (2 << (levels - 2)) - 1;
            for i in 0..total_nodes_with_children {
                let left_child = 2 * i + 1;
                let right_child = 2 * i + 2;
                edges.push((i, left_child));
                edges.push((i, right_child));
            }
            UnGraph::from_edges(edges)
        }

        let graph = create_binary_tree(3);
        let source = vec![0];
        let destination = (3..=6).collect();
        let k = 3;

        let result = important_cuts(&graph, source, destination, k);
        let result_edges = ImportantCut::vec_edge_indices(result);

        let expected_important_cuts = vec![vec![0, 4, 5], vec![2, 3, 1]];
        assert!(all_contained_vec(expected_important_cuts, result_edges));
    }
}
