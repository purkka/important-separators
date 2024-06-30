use std::collections::HashSet;

use petgraph::graph::EdgeIndex;
use petgraph::prelude::Bfs;
use petgraph::visit::NodeIndexable;
use rand::prelude::SliceRandom;
use rand::thread_rng;

use crate::cuts::path_residual::{Path, ResidualGraph, UnGraph};

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

    /// Pick arbitrary edge from cut. Returns a tuple of the edge index and the node index that lies
    /// in the destination set. Panics if edge does not exist, is not found or doesn't have
    /// endpoints in the source and destination sets.
    pub fn arbitrary_edge(&self, graph: &UnGraph) -> (usize, usize) {
        match self.cut_edge_set.choose(&mut thread_rng()) {
            None => panic!("Trying to get arbitrary edge from empty cut."),
            Some(&edge) => match graph.edge_endpoints(EdgeIndex::from(edge)) {
                None => panic!("Edge does not exist in graph."),
                Some((node_a, node_b)) => {
                    let node_a_index = NodeIndexable::to_index(&graph, node_a);
                    let node_b_index = NodeIndexable::to_index(&graph, node_b);
                    if self.source_set.contains(&node_a_index)
                        && self.destination_set.contains(&node_b_index)
                    {
                        (edge, node_b_index)
                    } else if self.source_set.contains(&node_b_index)
                        && self.destination_set.contains(&node_a_index)
                    {
                        (edge, node_a_index)
                    } else {
                        panic!("Picked edge does not have one endpoint in source set and one in destination set");
                    }
                }
            },
        }
    }
}

fn generate_minimum_cut_closest_to_destination(
    paths: Vec<Path>,
    residual_graph_reverse: ResidualGraph,
) -> Cut {
    // we assume that the given paths are valid for the given residual graph, hence this works
    let destination = Path::get_destination_node_index(&paths);
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
    use petgraph::graph;
    use petgraph::visit::NodeIndexable;

    use crate::cuts::cut::generate_minimum_cut_closest_to_destination;
    use crate::cuts::path_residual::{
        get_augmenting_paths_and_residual_graph, Path, ResidualGraph,
    };
    use crate::cuts::{path_residual, Cut};

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
        let graph = graph::UnGraph::<(), ()>::from_edges(&[
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

    #[test]
    fn test_get_arbitrary_edge() {
        let graph = path_residual::UnGraph::from_edges(&[(0, 1), (2, 1), (2, 3)]);
        let cut = Cut::new(vec![0, 1], vec![2, 3], vec![1]);

        let arbitrary_edge = cut.arbitrary_edge(&graph);
        assert_eq!((1, 2), arbitrary_edge);
    }
}
