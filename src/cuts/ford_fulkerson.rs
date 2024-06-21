use std::collections::VecDeque;

use petgraph::visit::{
    EdgeCount, EdgeIndexable, EdgeRef, IntoEdgeReferences, IntoEdges, NodeCount, NodeIndexable,
    VisitMap, Visitable,
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

fn get_augmenting_paths<G>(
    graph: G,
    source: G::NodeId,
    destination: G::NodeId,
    k: usize,
) -> Vec<Vec<<G as IntoEdgeReferences>::EdgeRef>>
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

    let mut paths: Vec<Vec<<G as IntoEdgeReferences>::EdgeRef>> = vec![];

    while has_augmenting_path(&graph, source, destination, &mut next_edge, &availability) {
        // get path
        let mut path = vec![];
        let mut vertex = destination;
        let mut vertex_index = NodeIndexable::to_index(&graph, vertex);
        while let Some(edge) = next_edge[vertex_index] {
            path.push(edge);
            vertex = other_endpoint(&graph, edge, vertex);
            vertex_index = NodeIndexable::to_index(&graph, vertex);
            // for each edge in the path, mark it as unavailable
            let edge_index = EdgeIndexable::to_index(&graph, edge.id());
            availability[edge_index] = false;
        }

        // flip order of path to have it start from the source and add to paths
        paths.push(path.into_iter().rev().collect());
    }

    if paths.len() <= k {
        paths
    } else {
        // no separators of size at most k, so return an empty vector
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use petgraph::graph::{EdgeReference, NodeIndex, UnGraph};
    use petgraph::visit::{EdgeRef, NodeIndexable};

    use crate::cuts::ford_fulkerson::{get_augmenting_paths, has_augmenting_path, other_endpoint};

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

        let paths = get_augmenting_paths(&graph, source, destination, 3);

        let expected_paths: Vec<Vec<(usize, usize)>> = vec![
            vec![(0, 1), (1, 2), (2, 6)],
            vec![(0, 3), (3, 6)],
            vec![(0, 4), (4, 5), (5, 6)],
        ];

        assert!(paths.iter().all(|path| {
            let path_node_tuples = get_node_tuples_for_path(path);
            expected_paths.contains(&path_node_tuples)
        }));
    }

    #[test]
    fn no_augmenting_paths_for_too_small_k() {
        let graph =
            UnGraph::<(), ()>::from_edges(&[(0, 1), (1, 4), (0, 2), (2, 4), (0, 3), (3, 4)]);
        let source = NodeIndexable::from_index(&graph, 0);
        let destination = NodeIndexable::from_index(&graph, 4);
        let k = 2;

        let paths = get_augmenting_paths(&graph, source, destination, k);
        assert!(paths.is_empty());
    }
}
