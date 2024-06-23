use petgraph::graph::NodeIndex;
use petgraph::{Directed, Graph};
use std::collections::{HashSet, VecDeque};

use crate::cuts::Cut;
use petgraph::visit::{
    Bfs, EdgeCount, EdgeIndexable, EdgeRef, IntoEdgeReferences, IntoEdges, NodeCount,
    NodeIndexable, VisitMap, Visitable,
};

// Based on petgraph::algo::ford_fulkerson

/// Gets the other endpoint of graph edge, if any, otherwise panics.
fn other_endpoint<G>(graph: G, edge: G::EdgeRef, vertex: G::NodeId) -> G::NodeId
where
    G: NodeIndexable + IntoEdges,
{
    if vertex == edge.source() {
        edge.target()
    } else if vertex == edge.target() {
        edge.source()
    } else {
        let end_point = NodeIndexable::to_index(&graph, vertex);
        panic!("Illegal endpoint {}", end_point);
    }
}

fn has_augmenting_path<G>(
    graph: G,
    source: G::NodeId,
    destination: G::NodeId,
    next_edge: &mut [Option<G::EdgeRef>],
    availability: &[bool],
) -> bool
where
    G: NodeIndexable + EdgeIndexable + Visitable + IntoEdges,
{
    let mut visited = graph.visit_map();
    let mut queue: VecDeque<G::NodeId> = VecDeque::new();
    visited.visit(source);
    queue.push_back(source);

    // do a BFS through the graph
    while let Some(vertex) = queue.pop_front() {
        for edge in graph.edges(vertex) {
            let next = other_endpoint(&graph, edge, vertex);
            let edge_index: usize = EdgeIndexable::to_index(&graph, edge.id());
            let edge_available = availability[edge_index];
            if !visited.is_visited(&next) && edge_available {
                next_edge[NodeIndexable::to_index(&graph, next)] = Some(edge);
                if next == destination {
                    // we've found an augmenting path
                    return true;
                }
                visited.visit(next);
                queue.push_back(next);
            }
        }
    }

    false
}

fn generate_initial_residual_graph<G>(graph: G) -> Graph<(), (), Directed, usize>
where
    G: IntoEdgeReferences + NodeIndexable,
{
    // we assume the input graph to not contain any lone vertices, hence we may generate the residual
    // graph from only the edges
    let mut residual_graph_edges = vec![];
    for edge in graph.edge_references() {
        let source_index = NodeIndexable::to_index(&graph, edge.source());
        let target_index = NodeIndexable::to_index(&graph, edge.target());
        residual_graph_edges.push((source_index, target_index, ()));
        residual_graph_edges.push((target_index, source_index, ()));
    }
    Graph::from_edges(residual_graph_edges)
}

/// Get augmenting paths and residual graph of graph if there exists a minimum cut of size at most k
///
/// The residual graph is built such that each edge that is part of an s-t path points from the
/// destination to the source. Every other edge gets two edges that point in both directions
fn get_augmenting_paths_and_residual_graph<G>(
    graph: G,
    source: G::NodeId,
    destination: G::NodeId,
    k: usize,
) -> Option<(
    Vec<Vec<<G as IntoEdgeReferences>::EdgeRef>>,
    Graph<(), (), Directed, usize>,
)>
where
    G: NodeIndexable
        + EdgeIndexable
        + NodeCount
        + EdgeCount
        + Visitable
        + IntoEdges
        + IntoEdgeReferences,
{
    let mut availability = vec![true; graph.edge_count()];
    let mut next_edge = vec![None; graph.node_count()];
    let mut residual = generate_initial_residual_graph(&graph);

    let mut paths: Vec<Vec<<G as IntoEdgeReferences>::EdgeRef>> = vec![];

    while has_augmenting_path(&graph, source, destination, &mut next_edge, &availability) {
        // get path corresponding to current state of `next_edge`
        let mut path = vec![];
        let mut vertex = destination;
        let mut vertex_index = NodeIndexable::to_index(&graph, vertex);
        while let Some(edge) = next_edge[vertex_index] {
            // While traversing, save the indices of the edge for removing the correct edge from
            // the residual graph. Our paths are saved from the destination to the source, hence
            // the first index is the target and the second the source. Refer to docstring for how
            // the residual graph will look like in the end.
            path.push(edge);
            let rm_edge_target_index = vertex_index;
            vertex = other_endpoint(&graph, edge, vertex);
            vertex_index = NodeIndexable::to_index(&graph, vertex);
            let rm_edge_source_index = vertex_index;
            // for each edge in the path, mark it as unavailable
            let edge_index = EdgeIndexable::to_index(&graph, edge.id());
            availability[edge_index] = false;
            // and adjust residual graph
            let removed_edge = residual.find_edge(
                NodeIndex::from(rm_edge_source_index),
                NodeIndex::from(rm_edge_target_index),
            );
            match removed_edge {
                None => panic!("Should always find an edge to remove in the residual graph"),
                Some(removed_edge_index) => {
                    let _ = residual.remove_edge(removed_edge_index);
                }
            }
        }

        // flip order of path to have it start from the source and add to paths
        paths.push(path.into_iter().rev().collect());
    }

    if paths.len() <= k {
        Some((paths, residual))
    } else {
        None
    }
}

fn generate_minimum_cut<G>(
    paths: Vec<Vec<<G as IntoEdgeReferences>::EdgeRef>>,
    residual: Graph<(), (), Directed, usize>,
    source: usize,
) -> Cut
where
    G: NodeIndexable + IntoEdgeReferences,
{
    let mut source_set = HashSet::<usize>::new();
    // find reachable region using BFS
    let mut bfs = Bfs::new(&residual, NodeIndex::from(source));
    while let Some(node) = bfs.next(&residual) {
        source_set.insert(NodeIndexable::to_index(&residual, node));
    }
    let mut destination_set = HashSet::<usize>::from_iter(0..residual.node_count());
    destination_set = destination_set.difference(&source_set).map(|i| *i).collect();

    // TODO Add cut edges
    Cut::new(
        source_set.into_iter().collect(),
        destination_set.into_iter().collect(),
        Vec::new(),
    )
}

#[cfg(test)]
mod tests {
    use petgraph::graph::{EdgeReference, NodeIndex, UnGraph};
    use petgraph::visit::{EdgeRef, NodeIndexable};

    use crate::cuts::ford_fulkerson::{
        get_augmenting_paths_and_residual_graph, has_augmenting_path, other_endpoint,
    };

    fn get_path_node_tuples(
        graph: &UnGraph<(), ()>,
        path: &[Option<EdgeReference<()>>],
        start: NodeIndex,
    ) -> Vec<(usize, usize)> {
        let mut path_node_tuples = vec![];
        let mut vertex = start;
        let mut vertex_index = NodeIndexable::to_index(&graph, vertex);
        while let Some(edge) = path[vertex_index] {
            let source_index = edge.source().index();
            let target_index = edge.target().index();
            path_node_tuples.push((source_index, target_index));
            vertex = other_endpoint(&graph, edge, vertex);
            vertex_index = NodeIndexable::to_index(&graph, vertex);
        }
        path_node_tuples
    }

    #[test]
    fn simple_augmenting_path() {
        let graph = UnGraph::<(), ()>::from_edges(&[(0, 1), (1, 2), (2, 3), (3, 4)]);
        let source = NodeIndexable::from_index(&graph, 0);
        let destination = NodeIndexable::from_index(&graph, 4);
        let mut path = vec![None; graph.node_count()];
        let availability = vec![true; graph.edge_count()];

        // check that we find a path
        let found_path = has_augmenting_path(&graph, source, destination, &mut path, &availability);
        assert!(found_path);

        // check the correctness of the path
        let path_node_tuples = get_path_node_tuples(&graph, &path, destination);
        let expected = vec![(3, 4), (2, 3), (1, 2), (0, 1)];
        assert_eq!(expected, path_node_tuples);
    }

    #[test]
    fn simple_augmenting_path_with_alternatives() {
        let graph =
            UnGraph::<(), ()>::from_edges(&[(0, 1), (1, 2), (2, 5), (0, 3), (3, 4), (4, 5)]);
        let source = NodeIndexable::from_index(&graph, 0);
        let destination = NodeIndexable::from_index(&graph, 5);
        let mut path = vec![None; graph.node_count()];
        let availability = vec![true; graph.edge_count()];

        let found_path = has_augmenting_path(&graph, source, destination, &mut path, &availability);
        assert!(found_path);

        let path_node_tuples = get_path_node_tuples(&graph, &path, destination);
        let accepted1 = vec![(2, 5), (1, 2), (0, 1)];
        let accepted2 = vec![(4, 5), (3, 4), (0, 3)];
        assert!(accepted1 == path_node_tuples || accepted2 == path_node_tuples);
    }

    #[test]
    fn no_augmenting_path() {
        let graph = UnGraph::<(), ()>::from_edges(&[(0, 1), (2, 3)]);
        let source = NodeIndexable::from_index(&graph, 0);
        let destination = NodeIndexable::from_index(&graph, 3);
        let mut path = vec![None; graph.node_count()];
        let availability = vec![true; graph.edge_count()];

        let found_path = has_augmenting_path(&graph, source, destination, &mut path, &availability);
        assert!(!found_path);
    }

    #[test]
    fn no_augmenting_path_available() {
        let graph = UnGraph::<(), ()>::from_edges(&[(0, 1), (1, 2), (2, 3)]);
        let source = NodeIndexable::from_index(&graph, 0);
        let destination = NodeIndexable::from_index(&graph, 3);
        let mut path = vec![None; graph.node_count()];
        let availability = vec![true, false, true];

        let found_path = has_augmenting_path(&graph, source, destination, &mut path, &availability);
        assert!(!found_path);
    }

    #[test]
    fn only_one_available_augmenting_path() {
        let graph = UnGraph::<(), ()>::from_edges(&[
            (0, 1),
            (1, 2),
            (2, 7),
            (0, 3),
            (3, 4),
            (4, 7),
            (0, 5),
            (5, 6),
            (6, 7),
        ]);
        let source = NodeIndexable::from_index(&graph, 0);
        let destination = NodeIndexable::from_index(&graph, 7);
        let mut path = vec![None; graph.node_count()];
        let mut availability = vec![true; graph.edge_count()];
        availability[2] = false;
        availability[4] = false;

        let found_path = has_augmenting_path(&graph, source, destination, &mut path, &availability);
        assert!(found_path);

        let path_node_tuples = get_path_node_tuples(&graph, &path, destination);
        let expected = vec![(6, 7), (5, 6), (0, 5)];
        assert_eq!(expected, path_node_tuples);
    }

    #[test]
    fn get_all_augmenting_paths() {
        fn get_node_tuples_for_path(path: &Vec<EdgeReference<()>>) -> Vec<(usize, usize)> {
            let mut path_node_tuples = vec![];
            for edge in path {
                let source_index = edge.source().index();
                let target_index = edge.target().index();
                path_node_tuples.push((source_index, target_index));
            }
            path_node_tuples
        }

        let graph = UnGraph::<(), ()>::from_edges(&[
            (0, 1),
            (1, 2),
            (2, 6),
            (0, 3),
            (3, 6),
            (0, 4),
            (4, 5),
            (5, 6),
        ]);
        let source = NodeIndexable::from_index(&graph, 0);
        let destination = NodeIndexable::from_index(&graph, 6);

        if let Some((paths, _)) =
            get_augmenting_paths_and_residual_graph(&graph, source, destination, 3)
        {
            let expected_paths = vec![
                vec![(0, 1), (1, 2), (2, 6)],
                vec![(0, 3), (3, 6)],
                vec![(0, 4), (4, 5), (5, 6)],
            ];

            assert!(paths.iter().all(|path| {
                let path_node_tuples = get_node_tuples_for_path(path);
                expected_paths.contains(&path_node_tuples)
            }));
        } else {
            assert!(false);
        }
    }

    #[test]
    fn no_augmenting_paths_for_too_small_k() {
        let graph =
            UnGraph::<(), ()>::from_edges(&[(0, 1), (1, 4), (0, 2), (2, 4), (0, 3), (3, 4)]);
        let source = NodeIndexable::from_index(&graph, 0);
        let destination = NodeIndexable::from_index(&graph, 4);
        let k = 2;

        let paths_and_residual =
            get_augmenting_paths_and_residual_graph(&graph, source, destination, k);
        assert!(paths_and_residual.is_none());
    }

    #[test]
    fn correct_residual_graph() {
        let graph = UnGraph::<(), ()>::from_edges(&[(0, 1), (1, 2), (0, 3)]);
        let source = NodeIndexable::from_index(&graph, 0);
        let destination = NodeIndexable::from_index(&graph, 2);

        if let Some((_, residual)) =
            get_augmenting_paths_and_residual_graph(&graph, source, destination, 1)
        {
            let residual_expected_edges = vec![(2, 1), (1, 0), (0, 3), (3, 0)];

            assert_eq!(4usize, residual.edge_count());
            assert!(residual.edge_references().all(|edge| {
                residual_expected_edges.contains(&(edge.source().index(), edge.target().index()))
            }));
        } else {
            assert!(false);
        }
    }
}
