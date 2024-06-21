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

#[cfg(test)]
mod tests {
    use petgraph::graph::{EdgeReference, NodeIndex, UnGraph};
    use petgraph::visit::{EdgeRef, NodeIndexable};

    use crate::cuts::ford_fulkerson::{has_augmenting_path, other_endpoint};

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
}
