use std::cmp::{max, min};
use std::collections::HashMap;
use std::collections::VecDeque;

use petgraph::graph::NodeIndex;
use petgraph::visit::{
    EdgeCount, EdgeIndexable, EdgeRef, IntoEdgeReferences, IntoEdges, NodeCount, NodeIndexable,
    VisitMap, Visitable,
};
use petgraph::{Directed, Graph, Undirected};

// Based on petgraph::algo::ford_fulkerson

pub struct Path {
    pub vertices: Vec<usize>,
    pub edges: Vec<usize>,
}

pub type ResidualGraph = Graph<(), (), Directed, usize>;

type UnGraph = Graph<(), (), Undirected, usize>;

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

fn generate_initial_residual_graph<G>(graph: G) -> ResidualGraph
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

fn remove_edge_from_residual_graph(
    residual_graph: &mut ResidualGraph,
    source_index: usize,
    target_index: usize,
) {
    let removed_edge =
        residual_graph.find_edge(NodeIndex::from(source_index), NodeIndex::from(target_index));
    match removed_edge {
        None => panic!("Should always find an edge to remove in the residual graph"),
        Some(removed_edge_index) => {
            let _ = residual_graph.remove_edge(removed_edge_index);
        }
    }
}

/// Get augmenting paths and reverse residual graph of graph if there exists a minimum cut of size at most k
///
/// The reverse residual graph is built such that each edge that is part of an s-t path points from the
/// source to the destination. Every other edge gets two edges that point in both directions
pub fn get_augmenting_paths_and_residual_graph<G>(
    graph: G,
    source: G::NodeId,
    destination: G::NodeId,
    k: usize,
) -> Option<(Vec<Path>, ResidualGraph)>
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
    // we build the reverse of the residual graph as we use it to find the minimum cut closest
    // to the target
    let mut residual_graph_reverse = generate_initial_residual_graph(&graph);

    let mut paths: Vec<Path> = vec![];

    while has_augmenting_path(&graph, source, destination, &mut next_edge, &availability) {
        // get path corresponding to current state of `next_edge`
        let mut vertex = destination;
        let mut vertex_index = NodeIndexable::to_index(&graph, vertex);
        let mut path_vertices = vec![vertex_index];
        let mut path_edges = vec![];
        while let Some(edge) = next_edge[vertex_index] {
            // While traversing, save the indices of the edge for removing the correct edge from
            // the residual graph. Our paths are saved from the destination to the source, hence
            // the first index is the source and the second the target. Refer to docstring for how
            // the residual graph will look like in the end.
            let rm_edge_source_index = vertex_index;
            vertex = other_endpoint(&graph, edge, vertex);
            vertex_index = NodeIndexable::to_index(&graph, vertex);
            let rm_edge_target_index = vertex_index;
            // for each edge in the path, mark it as unavailable
            let edge_index = EdgeIndexable::to_index(&graph, edge.id());
            availability[edge_index] = false;
            // add vertex and edge to path
            path_vertices.push(vertex_index);
            path_edges.push(edge_index);
            // and adjust the reverse residual graph
            remove_edge_from_residual_graph(
                &mut residual_graph_reverse,
                rm_edge_source_index,
                rm_edge_target_index,
            );
        }

        // flip order of path vertices/edges to have them start from the source and add to paths
        path_vertices = path_vertices.into_iter().rev().collect();
        path_edges = path_edges.into_iter().rev().collect();
        paths.push(Path {
            vertices: path_vertices,
            edges: path_edges,
        });
    }

    if paths.len() <= k {
        Some((paths, residual_graph_reverse))
    } else {
        None
    }
}

fn create_contracted_graph<G>(
    original_graph: G,
    source_set: Vec<usize>,
    destination_set: Vec<usize>,
) -> (UnGraph, usize, usize)
where
    G: NodeIndexable + IntoEdgeReferences,
{
    fn transform_if_in_set(element: &mut usize, set: &Vec<usize>, target: usize) {
        if set.contains(&element) {
            *element = target;
        }
    }

    let &new_source = source_set.first().expect("Source set should be nonempty");
    let &new_destination = destination_set
        .first()
        .expect("Destination set should be nonempty");

    let mut new_edges: Vec<(usize, usize)> = vec![];

    // keep track of how many indices are kept to avoid creating extra vertices
    let mut index_mapping = HashMap::<usize, usize>::new();

    for edge in original_graph.edge_references() {
        let mut edge_source = NodeIndexable::to_index(&original_graph, edge.source());
        let mut edge_target = NodeIndexable::to_index(&original_graph, edge.target());

        transform_if_in_set(&mut edge_source, &source_set, new_source);
        transform_if_in_set(&mut edge_target, &source_set, new_source);
        transform_if_in_set(&mut edge_source, &destination_set, new_destination);
        transform_if_in_set(&mut edge_target, &destination_set, new_destination);

        if edge_source != edge_target {
            // add source and target indices to the index mapping in order if they don't exist already
            let smaller = min(edge_source, edge_target);
            let mut new_index = index_mapping.len();
            index_mapping.entry(smaller).or_insert(new_index);
            let bigger = max(edge_source, edge_target);
            new_index = index_mapping.len();
            index_mapping.entry(bigger).or_insert(new_index);

            match (
                index_mapping.get(&edge_source),
                index_mapping.get(&edge_target),
            ) {
                (Some(&s), Some(&t)) => {
                    if !new_edges.contains(&(s, t)) {
                        new_edges.push((s, t))
                    }
                }
                (_, _) => panic!("Edge source and target should always be in the index mapping"),
            }
        }
    }

    match (
        index_mapping.get(&new_source),
        index_mapping.get(&new_destination),
    ) {
        (Some(&s), Some(&t)) => (UnGraph::from_edges(new_edges), s, t),
        (_, _) => panic!("New edge source and target should always be in the index mapping"),
    }
}

#[cfg(test)]
mod tests {
    use petgraph::graph::{EdgeReference, NodeIndex, UnGraph};
    use petgraph::visit::{EdgeRef, NodeIndexable};

    use crate::cuts::path_residual::{
        create_contracted_graph, get_augmenting_paths_and_residual_graph, has_augmenting_path,
        other_endpoint,
    };

    fn get_path_vertex_tuples(
        graph: &UnGraph<(), ()>,
        path: &[Option<EdgeReference<()>>],
        start: NodeIndex,
    ) -> Vec<(usize, usize)> {
        let mut path_vertex_tuples = vec![];
        let mut vertex = start;
        let mut vertex_index = NodeIndexable::to_index(&graph, vertex);
        while let Some(edge) = path[vertex_index] {
            let source_index = edge.source().index();
            let target_index = edge.target().index();
            path_vertex_tuples.push((source_index, target_index));
            vertex = other_endpoint(&graph, edge, vertex);
            vertex_index = NodeIndexable::to_index(&graph, vertex);
        }
        path_vertex_tuples
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
        let path_vertex_tuples = get_path_vertex_tuples(&graph, &path, destination);
        let expected = vec![(3, 4), (2, 3), (1, 2), (0, 1)];
        assert_eq!(expected, path_vertex_tuples);
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

        let path_vertex_tuples = get_path_vertex_tuples(&graph, &path, destination);
        let accepted1 = vec![(2, 5), (1, 2), (0, 1)];
        let accepted2 = vec![(4, 5), (3, 4), (0, 3)];
        assert!(accepted1 == path_vertex_tuples || accepted2 == path_vertex_tuples);
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

        let path_vertex_tuples = get_path_vertex_tuples(&graph, &path, destination);
        let expected = vec![(6, 7), (5, 6), (0, 5)];
        assert_eq!(expected, path_vertex_tuples);
    }

    #[test]
    fn get_all_augmenting_paths() {
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
            let expected_paths = vec![vec![0, 1, 2, 6], vec![0, 3, 6], vec![0, 4, 5, 6]];
            assert!(paths
                .iter()
                .all(|path| { expected_paths.contains(&path.vertices) }));
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

        if let Some((_, residual_reverse)) =
            get_augmenting_paths_and_residual_graph(&graph, source, destination, 1)
        {
            let residual_reverse_expected_edges = vec![(1, 2), (0, 1), (0, 3), (3, 0)];

            assert_eq!(4usize, residual_reverse.edge_count());
            assert!(residual_reverse.edge_references().all(|edge| {
                residual_reverse_expected_edges
                    .contains(&(edge.source().index(), edge.target().index()))
            }));
        } else {
            assert!(false);
        }
    }

    #[test]
    fn correct_contracted_graph() {
        let graph =
            UnGraph::<(), ()>::from_edges(&[(0, 1), (0, 2), (1, 3), (1, 4), (2, 4), (3, 4)]);
        let source_set = vec![0, 1];
        let destination_set = vec![3, 4];

        let (graph, new_source, new_dest) =
            create_contracted_graph(&graph, source_set, destination_set);
        let edge_indices = graph
            .edge_references()
            .map(|edge| (edge.source().index(), edge.target().index()))
            .collect::<Vec<_>>();

        assert_eq!(3, edge_indices.len());
        assert!(edge_indices.contains(&(0, 1)));
        assert!(edge_indices.contains(&(0, 2)));
        assert!(edge_indices.contains(&(1, 2)));
        assert_eq!(0, new_source);
        assert_eq!(2, new_dest);
    }
}
