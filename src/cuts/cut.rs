use std::collections::HashSet;

use itertools::Itertools;
use petgraph::graph::EdgeIndex;
use petgraph::prelude::Bfs;
use petgraph::visit::{EdgeIndexable, EdgeRef, IntoEdgeReferences, NodeIndexable};
use rand::prelude::SliceRandom;
use rand::thread_rng;

use crate::cuts::path_residual::{IndexMapping, Path, ResidualGraph, UnGraph};

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

#[derive(Debug)]
pub struct ImportantCut {
    pub edge_indices: Vec<usize>,
}

impl ImportantCut {
    pub fn from(edge_indices: Vec<usize>) -> Self {
        Self {
            edge_indices: edge_indices.into_iter().unique().collect(),
        }
    }

    #[allow(dead_code)]
    pub fn vertex_pairs<G>(&self, graph: G) -> Vec<(usize, usize)>
    where
        G: NodeIndexable + EdgeIndexable + IntoEdgeReferences,
    {
        self.edge_indices
            .iter()
            .map(|&edge_index| {
                match graph
                    .edge_references()
                    .find(|edge| EdgeIndexable::to_index(&graph, edge.id()) == edge_index)
                {
                    None => panic!("Edge does not exist in graph."),
                    Some(edge) => {
                        let edge_source_id = NodeIndexable::to_index(&graph, edge.source());
                        let edge_target_id = NodeIndexable::to_index(&graph, edge.target());
                        (edge_source_id, edge_target_id)
                    }
                }
            })
            .collect()
    }

    #[allow(dead_code)]
    pub fn vec_edge_indices(cuts: Vec<ImportantCut>) -> Vec<Vec<usize>> {
        cuts.iter().map(|ic| ic.edge_indices.clone()).collect()
    }

    pub fn vec_vertex_indices<G>(graph: G, cuts: Vec<ImportantCut>) -> Vec<Vec<(usize, usize)>>
    where
        G: NodeIndexable + EdgeIndexable + IntoEdgeReferences,
    {
        cuts.iter()
            .map(|ic| ic.vertex_pairs(&graph))
            .unique()
            .collect()
    }

    pub fn print_important_cuts<G>(graph: G, cuts: Vec<ImportantCut>)
        where
            G: NodeIndexable + EdgeIndexable + IntoEdgeReferences, {
        println!("Important cuts:");
        for ic_indices in ImportantCut::vec_vertex_indices(&graph, cuts) {
            println!("- {:?}", ic_indices);
        }
    }
}

fn generate_minimum_cut_closest_to_destination(
    paths: &Vec<Path>,
    residual_graph_reverse: ResidualGraph,
) -> Cut {
    // we assume that the given paths are valid for the given residual graph, hence this works
    let destination = Path::get_destination_node_index(&paths);
    let source = Path::get_source_node_index(&paths);

    let mut destination_set = HashSet::<usize>::new();
    // find reachable region starting from destination using BFS
    let mut bfs = Bfs::new(&residual_graph_reverse, destination);
    while let Some(node) = bfs.next(&residual_graph_reverse) {
        // stop traversing graph when we hit the source node
        if node == source {
            continue;
        }
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

pub fn generate_minimum_cut_closest_to_destination_with_mapping(
    paths: &Vec<Path>,
    residual_graph_reverse: ResidualGraph,
    index_mapping: IndexMapping,
) -> Cut {
    let min_cut_contracted =
        generate_minimum_cut_closest_to_destination(paths, residual_graph_reverse);

    let mut source_set_mapped = vec![];
    let mut destination_set_mapped = vec![];
    let mut edge_set_mapped = vec![];

    for source_vertex in min_cut_contracted.source_set {
        match index_mapping
            .vertex_contracted_to_original
            .get(&source_vertex)
        {
            None => panic!("Index mapping missing entry for vertex {}", source_vertex),
            Some(values) => source_set_mapped.extend(values.clone()),
        }
    }

    for dest_vertex in min_cut_contracted.destination_set {
        match index_mapping
            .vertex_contracted_to_original
            .get(&dest_vertex)
        {
            None => panic!("Index mapping missing entry for vertex {}", dest_vertex),
            Some(values) => destination_set_mapped.extend(values.clone()),
        }
    }

    for cut_edge in min_cut_contracted.cut_edge_set {
        match index_mapping.edge_contracted_to_original.get(&cut_edge) {
            None => panic!("Index mapping missing entry for edge {}", cut_edge),
            Some(values) => edge_set_mapped.extend(values.clone()),
        }
    }

    Cut::new(source_set_mapped, destination_set_mapped, edge_set_mapped)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use petgraph::graph;
    use petgraph::graph::NodeIndex;
    use petgraph::visit::NodeIndexable;

    use crate::cuts::cut::{
        generate_minimum_cut_closest_to_destination,
        generate_minimum_cut_closest_to_destination_with_mapping, ImportantCut,
    };
    use crate::cuts::path_residual::{
        get_augmenting_paths_and_residual_graph, IndexMapping, Path, ResidualGraph,
    };
    use crate::cuts::{path_residual, Cut};

    fn all_contained(lhs: Vec<usize>, rhs: Vec<usize>) -> bool {
        lhs.iter().all(|elem| rhs.contains(elem))
    }

    fn all_pairs_contained(lhs: Vec<(usize, usize)>, rhs: Vec<(usize, usize)>) -> bool {
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

        let cut = generate_minimum_cut_closest_to_destination(&paths, residual_graph_reverse);

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

        if let Some((paths, residual_reverse)) = get_augmenting_paths_and_residual_graph(
            &graph,
            source,
            destination,
            2,
            &mut vec![1; graph.edge_count()],
        ) {
            let cut_r_max = generate_minimum_cut_closest_to_destination(&paths, residual_reverse);

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

    #[test]
    fn correct_minimum_cut_generation_with_mapping() {
        let contracted_graph = path_residual::UnGraph::from_edges(&[(0, 1), (0, 2), (1, 2)]);
        let source = NodeIndex::from(0);
        let destination = NodeIndex::from(2);
        let index_mapping = IndexMapping::from(
            HashMap::from([(0, vec![0, 1]), (1, vec![2]), (2, vec![3, 4])]),
            HashMap::from([(0, vec![1]), (1, vec![2, 3]), (2, vec![4])]),
        );

        if let Some((paths, residual_reverse)) = get_augmenting_paths_and_residual_graph(
            &contracted_graph,
            source,
            destination,
            3,
            &mut vec![1; contracted_graph.edge_count()],
        ) {
            let cut_r_max = generate_minimum_cut_closest_to_destination_with_mapping(
                &paths,
                residual_reverse,
                index_mapping,
            );

            let expected_source_set: Vec<usize> = vec![0, 1, 2];
            let expected_destination_set: Vec<usize> = vec![3, 4];
            let expected_cut_edge_set: Vec<usize> = vec![2, 3, 4];
            let expected_cut_size = 3;

            assert_eq!(expected_cut_size, cut_r_max.size);
            assert!(all_contained(expected_source_set, cut_r_max.source_set));
            assert!(all_contained(
                expected_destination_set,
                cut_r_max.destination_set
            ));
            assert!(all_contained(
                expected_cut_edge_set,
                cut_r_max.cut_edge_set.clone()
            ));
        } else {
            assert!(false);
        }
    }

    #[test]
    fn important_cut_get_vertex_pairs() {
        let graph =
            graph::UnGraph::<(), ()>::from_edges(&[(0, 1), (0, 2), (1, 4), (0, 3), (1, 2), (2, 3)]);

        let important_cut = ImportantCut::from(vec![0, 2, 3]);

        let pairs = important_cut.vertex_pairs(&graph);
        assert_eq!(3, pairs.len());
        let expected_pairs = vec![(0, 1), (1, 4), (0, 3)];
        assert!(all_pairs_contained(expected_pairs, pairs));
    }
}
