use eframe::emath::{Pos2, Vec2};
use eframe::epaint::{CircleShape, FontFamily, FontId, Shape, Stroke, TextShape};
use egui::Color32;
use egui_graphs::{DisplayNode, DrawContext, NodeProps};
use petgraph::EdgeType;
use petgraph::stable_graph::IndexType;

trait SourceDestinationInfo {
    fn get_node_type(&self) -> NodeType;
}

#[derive(Clone, Debug)]
enum NodeType {
    SOURCE,
    DESTINATION,
    OTHER,
}

#[derive(Clone, Debug)]
pub(crate) struct NodeData {
    node_type: NodeType,
}

impl NodeData {
    pub(crate) fn new() -> Self {
        Self {
            node_type: NodeType::OTHER
        }
    }

    pub(crate) fn new_source() -> Self {
        Self {
            node_type: NodeType::SOURCE
        }
    }

    pub(crate) fn new_destination() -> Self {
        Self {
            node_type: NodeType::DESTINATION
        }
    }
}

impl SourceDestinationInfo for NodeData {
    fn get_node_type(&self) -> NodeType {
        self.node_type.clone()
    }
}

struct SourceDestinationColor;

impl SourceDestinationColor {
    const SOURCE: Color32 = Color32::from_rgb(0x80, 0x80, 0xFF);
    const SOURCE_INTERACTED: Color32 = Color32::from_rgb(0xB0, 0xB0, 0xFF);
    const DESTINATION: Color32 = Color32::from_rgb(0xFF, 0x80, 0x80);
    const DESTINATION_INTERACTED: Color32 = Color32::from_rgb(0xFF, 0xB0, 0xB0);

    fn get_source_color(is_interacted: bool) -> Color32 {
        match is_interacted {
            true => Self::SOURCE_INTERACTED,
            false => Self::SOURCE,
        }
    }

    fn get_destination_color(is_interacted: bool) -> Color32 {
        match is_interacted {
            true => Self::DESTINATION_INTERACTED,
            false => Self::DESTINATION,
        }
    }
}

#[derive(Clone)]
pub(crate) struct CustomNodeShape {
    pos: Pos2,
    label_text: String,
    selected: bool,
    dragged: bool,

    radius: f32,
    node_type: NodeType,
}

impl<N: Clone + SourceDestinationInfo> From<NodeProps<N>> for CustomNodeShape {
    fn from(node_props: NodeProps<N>) -> Self {
        Self {
            pos: node_props.location,
            label_text: node_props.label.to_string(),
            selected: node_props.selected,
            dragged: node_props.dragged,
            radius: 5.0,
            node_type: node_props.payload.get_node_type(),
        }
    }
}

impl<N: Clone + SourceDestinationInfo, E: Clone, Ty: EdgeType, Ix: IndexType> DisplayNode<N, E, Ty, Ix> for CustomNodeShape {
    fn closest_boundary_point(&self, dir: Vec2) -> Pos2 {
        closest_point_on_circle(self.pos, self.radius, dir)
    }

    fn shapes(&mut self, ctx: &DrawContext) -> Vec<Shape> {
        let mut res = Vec::with_capacity(2);

        let is_interacted = self.selected || self.dragged;

        let color = match self.node_type {
            NodeType::SOURCE => SourceDestinationColor::get_source_color(is_interacted),
            NodeType::DESTINATION => SourceDestinationColor::get_destination_color(is_interacted),
            NodeType::OTHER => {
                let style = match is_interacted {
                    true => ctx.ctx.style().visuals.widgets.active,
                    false => ctx.ctx.style().visuals.widgets.inactive,
                };
                style.fg_stroke.color
            }
        };

        let circle_center = ctx.meta.canvas_to_screen_pos(self.pos);
        let circle_radius = ctx.meta.canvas_to_screen_size(self.radius);
        let circle_shape = CircleShape {
            center: circle_center,
            radius: circle_radius,
            fill: color,
            stroke: Stroke::default(),
        };
        res.push(circle_shape.into());

        let black = Color32::BLACK;

        let galley = ctx.ctx.fonts(|f| {
            f.layout_no_wrap(
                self.label_text.clone(),
                FontId::new(circle_radius, FontFamily::Monospace),
                black,
            )
        });

        // display label in the middle of the circle
        let label_pos = Pos2::new(
            circle_center.x - galley.size().x / 2.,
            circle_center.y - galley.size().y / 2.,
        );

        let label_shape = TextShape::new(label_pos, galley, black);
        res.push(label_shape.into());

        res
    }

    fn update(&mut self, state: &NodeProps<N>) {
        self.pos = state.location;
        self.label_text = state.label.to_string();
        self.selected = state.selected;
        self.dragged = state.dragged;
    }

    fn is_inside(&self, pos: Pos2) -> bool {
        is_inside_circle(self.pos, self.radius, pos)
    }
}

fn closest_point_on_circle(center: Pos2, radius: f32, dir: Vec2) -> Pos2 {
    center + dir.normalized() * radius
}

fn is_inside_circle(center: Pos2, radius: f32, pos: Pos2) -> bool {
    let dir = pos - center;
    dir.length() <= radius
}
