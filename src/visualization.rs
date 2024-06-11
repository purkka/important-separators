use eframe::{run_native, App, CreationContext};
use egui::Context;
use egui_graphs;
use egui_graphs::{DefaultEdgeShape, DefaultNodeShape, GraphView};
use petgraph;
use petgraph::visit::{EdgeRef, IntoNodeReferences};
use petgraph::prelude::StableGraph;

pub struct GraphApp {
    graph: egui_graphs::Graph<(), u8>,
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
        let settings_style = &egui_graphs::SettingsStyle::new().with_labels_always(true);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(
                &mut GraphView::<_, _, _, _, DefaultNodeShape, DefaultEdgeShape>::new(
                    &mut self.graph,
                )
                    .with_styles(settings_style),
            );
        });
    }
}

fn generate_graph(graph: &petgraph::Graph<(), u8>) -> egui_graphs::Graph<(), u8> {
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
