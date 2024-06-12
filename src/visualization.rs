use eframe::{run_native, App, CreationContext};
use egui::Context;
use egui_graphs;
use egui_graphs::{DefaultEdgeShape, GraphView, SettingsInteraction, SettingsStyle};
use petgraph;
use petgraph::Directed;
use petgraph::visit::{EdgeRef, IntoNodeReferences};
use petgraph::prelude::{NodeIndex, StableGraph};
use petgraph::stable_graph::DefaultIx;
use crate::node::CustomNodeShape;

pub struct GraphApp {
    graph: egui_graphs::Graph<(), u8, Directed, DefaultIx, CustomNodeShape>,
}

impl GraphApp {
    #[allow(dead_code)]
    pub(crate) fn new(graph: petgraph::Graph<(), u8>, _: &CreationContext<'_>) -> Self {
        Self {
            graph: generate_graph(&graph),
        }
    }
}

impl App for GraphApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        let settings_style = &SettingsStyle::new().with_labels_always(true);
        let interaction_settings = &SettingsInteraction::new()
            .with_dragging_enabled(true)
            .with_node_clicking_enabled(true)
            .with_node_selection_enabled(true);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(
                &mut GraphView::<_, _, _, _, CustomNodeShape, DefaultEdgeShape>::new(
                    &mut self.graph,
                )
                    .with_styles(settings_style)
                    .with_interactions(interaction_settings),
            );
        });
    }
}

fn generate_graph(graph: &petgraph::Graph<(), u8>) -> egui_graphs::Graph<(), u8, Directed, DefaultIx, CustomNodeShape> {
    let mut g = StableGraph::new();

    graph.node_references().for_each(|_| {
        g.add_node(());
    });

    graph.edge_references().for_each(|edge| {
        g.add_edge(edge.source(), edge.target(), *edge.weight());
    });

    egui_graphs::Graph::from(&g)
}

pub fn draw_graph(graph: petgraph::Graph<(), u8>) {
    let native_options = eframe::NativeOptions::default();
    run_native(
        "Important Separator Project",
        native_options,
        Box::new(|cc| Box::new(GraphApp::new(graph, cc))),
    )
        .unwrap();
}
