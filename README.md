# Important Cut Project

The goal of this project is to implement an algorithm that enumerates all _important 
cuts_ of size at most $k$ in some simple undirected graph. Eventually the aim is to
extend the algorithm for _important separators_ and test it on real-world graphs.
The implementation follows the approach described in
[_Parametrized Algorithms_ by Cygan et al.](https://doi.org/10.1007/978-3-319-21275-3).

## Usage

Define a graph, source and destination sets, as well as a desired $k$ value
in `main.rs`, then run and compile the application with:

```bash
cargo run
```

The graph can be any `petgraph` graph type as long as it is node indexable,
e.g. `petgraph::graph::UnGraph`. The program assumes that it's input is always
undirected, and node and edge weights are ignored.

## Acknowledgements

Special thanks to [Manuel Sorge](https://manyu.pro/)
for guidance and valuable feedback over the course of this project.
