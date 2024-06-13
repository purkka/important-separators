use eframe::{run_native, App, CreationContext};
use egui::{Context, Style, Visuals};
use egui_graphs;
use egui_graphs::{DefaultEdgeShape, GraphView, SettingsInteraction, SettingsStyle};
use petgraph;
use petgraph::Directed;
use petgraph::visit::{EdgeRef, IntoNodeReferences};
use petgraph::prelude::{StableGraph};
use petgraph::stable_graph::DefaultIx;
use crate::node::{CustomNodeShape, NodeData};

pub struct GraphApp {
    graph: egui_graphs::Graph<NodeData, u8, Directed, DefaultIx, CustomNodeShape>,
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

fn generate_graph(graph: &petgraph::Graph<(), u8>) -> egui_graphs::Graph<NodeData, u8, Directed, DefaultIx, CustomNodeShape> {
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
        Box::new(|cc| {
            // Set to dark mode always
            let style = Style {
                visuals: Visuals::dark(),
                ..Style::default()
            };
            cc.egui_ctx.set_style(style);
            Box::new(GraphApp::new(graph, cc))
        }),
    )
        .unwrap();
}
