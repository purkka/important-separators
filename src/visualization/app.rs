use eframe::{run_native, App, CreationContext};
use egui::{Context, Style, Visuals};
use egui_graphs;
use egui_graphs::{GraphView, SettingsInteraction, SettingsStyle};
use petgraph;
use petgraph::Undirected;
use petgraph::visit::{EdgeRef, IntoNodeReferences};
use petgraph::prelude::StableUnGraph;
use petgraph::stable_graph::DefaultIx;
use crate::cuts::Cut;
use crate::visualization::edge::{CustomEdgeShape, EdgeData};
use crate::visualization::node::{CustomNodeShape, NodeData};

// TODO Implement toggling between directed and undirected graphs e.g. via generics

struct GraphApp {
    graph: egui_graphs::Graph<NodeData, EdgeData, Undirected, DefaultIx, CustomNodeShape, CustomEdgeShape>,
}

impl GraphApp {
    #[allow(dead_code)]
    pub(crate) fn new(graph: petgraph::Graph<(), (), Undirected>, cut: Cut<petgraph::Graph<(), (), Undirected>>, _: &CreationContext<'_>) -> Self {
        Self {
            graph: generate_graph(&graph, cut),
        }
    }
}

impl App for GraphApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        let settings_style = &SettingsStyle::new().with_labels_always(true);
        let interaction_settings = &SettingsInteraction::new().with_dragging_enabled(true).with_node_clicking_enabled(true).with_node_selection_enabled(true);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(
                &mut GraphView::<_, _, _, _, CustomNodeShape, CustomEdgeShape>::new(
                    &mut self.graph,
                ).with_styles(settings_style).with_interactions(interaction_settings),
            );
        });
    }
}

fn generate_graph(graph: &petgraph::Graph<(), (), Undirected>, cut: Cut<petgraph::Graph<(), (), Undirected>>) -> egui_graphs::Graph<NodeData, EdgeData, Undirected, DefaultIx, CustomNodeShape, CustomEdgeShape> {
    let mut g = StableUnGraph::with_capacity(graph.node_count(), graph.edge_count());

    graph.node_references().for_each(|(node_index, _)| {
        // Color vertices according to the cut
        if cut.source_set.contains(&node_index) {
            g.add_node(NodeData::new_source());
        } else if cut.destination_set.contains(&node_index) {
            g.add_node(NodeData::new_destination());
        } else {
            // This is unreachable for now, but we'll keep it for when cuts change to separators
            g.add_node(NodeData::new());
        }
    });

    graph.edge_references().for_each(|edge| {
        let is_colored = cut.cut_set.contains(&edge.id());
        g.add_edge(edge.source(), edge.target(), EdgeData::new(is_colored));
    });

    egui_graphs::Graph::from(&g)
}


pub fn draw_graph(graph: petgraph::Graph<(), (), Undirected>, cut_input: Cut<&petgraph::Graph<(), (), Undirected>>) {
    // TODO Find better solution for monkey patch below
    let cut: Cut<petgraph::Graph<(), (), Undirected>> = Cut::new(
        cut_input.source_set,
        cut_input.destination_set,
        cut_input.cut_set,
    );

    let native_options = eframe::NativeOptions::default();
    run_native(
        "Important Separator Project",
        native_options,
        Box::new(|cc| {
            // Set to dark mode always
            let style = Style {
                visuals: Visuals::dark(),
                ..Style::default()
            };
            cc.egui_ctx.set_style(style);
            Box::new(GraphApp::new(graph, cut, cc))
        }),
    ).unwrap();
}
