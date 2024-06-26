use crate::cuts::path_residual::{Path, ResidualGraph};
use petgraph::graph::NodeIndex;
use petgraph::prelude::Bfs;
use petgraph::visit::NodeIndexable;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub struct Cut {
    pub source_set: Vec<usize>,
    pub destination_set: Vec<usize>,
    pub cut_edge_set: Vec<usize>,
    pub size: usize,
}

impl Cut {
    pub fn new(
        source_set: Vec<usize>,
        destination_set: Vec<usize>,
        cut_edge_set: Vec<usize>,
    ) -> Self {
        let size = cut_edge_set.len();
        Self {
            source_set,
            destination_set,
            cut_edge_set,
            size,
        }
    }
}

fn generate_minimum_cut_closest_to_destination(
    paths: Vec<Path>,
    residual_graph_reverse: ResidualGraph,
) -> Cut {
    assert!(!paths.is_empty());
    // we assume that the given paths are valid for the given residual graph, hence this works
    let destination = NodeIndex::from(
        *paths[0]
            .vertices
            .last()
            .expect("The vertices of a path cannot be empty"),
    );
    let mut destination_set = HashSet::<usize>::new();
    // find reachable region starting from destination using BFS
    let mut bfs = Bfs::new(&residual_graph_reverse, destination);
    while let Some(node) = bfs.next(&residual_graph_reverse) {
        destination_set.insert(NodeIndexable::to_index(&residual_graph_reverse, node));
    }
    let mut source_set = HashSet::<usize>::from_iter(0..residual_graph_reverse.node_count());
    source_set = source_set
        .difference(&destination_set)
        .map(|i| *i)
        .collect();

    let mut cut_edges = vec![];
    for path in paths {
        let find_index = (0..(path.vertices.len() - 1)).find(|&i| {
            source_set.contains(&path.vertices[i])
                && destination_set.contains(&path.vertices[i + 1])
        });
        match find_index {
            None => panic!("Every path should have one edge in the minimum cut"),
            Some(index) => cut_edges.push(path.edges[index]),
        }
    }

    Cut::new(
        source_set.into_iter().collect(),
        destination_set.into_iter().collect(),
        cut_edges,
    )
}

#[cfg(test)]
mod tests {
    use crate::cuts::cut::generate_minimum_cut_closest_to_destination;
    use crate::cuts::path_residual::{
        get_augmenting_paths_and_residual_graph, Path, ResidualGraph,
    };
    use petgraph::graph::UnGraph;
    use petgraph::visit::NodeIndexable;

    fn all_contained(lhs: Vec<usize>, rhs: Vec<usize>) -> bool {
        lhs.iter().all(|elem| rhs.contains(elem))
    }

    #[test]
    fn correct_minimum_graph_generation() {
        // TODO Maybe this test (and the one below) could benefit from a visualization?
        let residual_graph_reverse = ResidualGraph::from_edges(&[
            // bidirectional edges
            (0, 1),
            (1, 0),
            (1, 2),
            (2, 1),
            (1, 4),
            (4, 1),
            (2, 3),
            (3, 2),
            // path edges
            (0, 2),
            (2, 4),
            (4, 7),
            (0, 3),
            (3, 5),
            (5, 6),
            (6, 7),
        ]);
        let paths = vec![
            Path {
                vertices: vec![0, 2, 4, 7],
                edges: vec![1, 6, 8],
            },
            Path {
                vertices: vec![0, 3, 5, 6, 7],
                edges: vec![2, 7, 9, 10],
            },
        ];

        let cut = generate_minimum_cut_closest_to_destination(paths, residual_graph_reverse);

        let expected_source_set: Vec<usize> = vec![0, 1, 2, 3, 4, 5, 6];
        let expected_destination_set: Vec<usize> = vec![7];
        let expected_cut_edge_set: Vec<usize> = vec![8, 10];

        assert_eq!(2, cut.size);
        assert!(all_contained(expected_source_set, cut.source_set));
        assert!(all_contained(expected_destination_set, cut.destination_set));
        assert!(all_contained(expected_cut_edge_set, cut.cut_edge_set));
    }

    #[test]
    fn correct_minimum_graph_generation_from_graph() {
        let graph = UnGraph::<(), ()>::from_edges(&[
            (0, 1),
            (0, 2),
            (0, 3),
            (1, 2),
            (2, 3),
            (1, 4),
            (2, 4),
            (3, 5),
            (4, 7),
            (5, 6),
            (6, 7),
        ]);
        let source = NodeIndexable::from_index(&graph, 0);
        let destination = NodeIndexable::from_index(&graph, 7);

        if let Some((paths, residual_reverse)) =
            get_augmenting_paths_and_residual_graph(&graph, source, destination, 2)
        {
            let cut_r_max = generate_minimum_cut_closest_to_destination(paths, residual_reverse);

            let expected_source_set_rev: Vec<usize> = vec![0, 1, 2, 3, 4, 5, 6];
            let expected_destination_set_rev: Vec<usize> = vec![7];
            let expected_cut_edge_set_rev: Vec<usize> = vec![8, 10];

            assert_eq!(2, cut_r_max.size);
            assert!(all_contained(expected_source_set_rev, cut_r_max.source_set));
            assert!(all_contained(
                expected_destination_set_rev,
                cut_r_max.destination_set
            ));
            assert!(all_contained(
                expected_cut_edge_set_rev,
                cut_r_max.cut_edge_set
            ));
        } else {
            assert!(false);
        }
    }
}
