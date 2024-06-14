use std::iter::zip;
use eframe::{run_native, App, CreationContext};
use egui::{Context, Style, Visuals};
use egui_graphs;
use egui_graphs::{GraphView, SettingsInteraction, SettingsStyle};
use petgraph;
use petgraph::Directed;
use petgraph::visit::{EdgeRef, IntoNodeReferences};
use petgraph::prelude::{StableGraph};
use petgraph::stable_graph::DefaultIx;
use crate::visualization::edge::{CustomEdgeShape, EdgeData};
use crate::visualization::node::{CustomNodeShape, NodeData};

struct GraphApp {
    graph: egui_graphs::Graph<NodeData, EdgeData, Directed, DefaultIx, CustomNodeShape, CustomEdgeShape>,
}

impl GraphApp {
    #[allow(dead_code)]
    pub(crate) fn new(graph: petgraph::Graph<(), u8>, colored_edges: Vec<bool>, _: &CreationContext<'_>) -> Self {
        Self {
            graph: generate_graph(&graph, colored_edges),
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
                &mut GraphView::<_, _, _, _, CustomNodeShape, CustomEdgeShape>::new(
                    &mut self.graph,
                )
                    .with_styles(settings_style)
                    .with_interactions(interaction_settings),
            );
        });
    }
}

fn generate_graph(graph: &petgraph::Graph<(), u8>, colored_edges: Vec<bool>) -> egui_graphs::Graph<NodeData, EdgeData, Directed, DefaultIx, CustomNodeShape, CustomEdgeShape> {
    let mut g = StableGraph::new();

    graph.node_references().for_each(|(node_index, _)| {
        // For now have the first node be the source and the last node be the sink
        if node_index.index() == 0usize {
            g.add_node(NodeData::new_source());
        } else if node_index.index() == graph.node_count() - 1 {
            g.add_node(NodeData::new_sink());
        } else {
            g.add_node(NodeData::new());
        }
    });

    zip(graph.edge_references(), colored_edges).for_each(|(edge, is_colored)| {
        g.add_edge(edge.source(), edge.target(), EdgeData::new(is_colored));
    });

    egui_graphs::Graph::from(&g)
}


pub fn draw_graph(graph: petgraph::Graph<(), u8>, colored_edges: Vec<bool>) {
    assert_eq!(graph.edge_count(), colored_edges.len());

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
            Box::new(GraphApp::new(graph, colored_edges, cc))
        }),
    )
        .unwrap();
}
