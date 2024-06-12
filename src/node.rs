use eframe::emath::{Pos2, Vec2};
use eframe::epaint::{CircleShape, FontFamily, FontId, Shape, Stroke, TextShape};
use egui::Color32;
use egui_graphs::{DisplayNode, DrawContext, NodeProps};
use petgraph::EdgeType;
use petgraph::stable_graph::IndexType;

#[derive(Clone)]
pub struct CustomNodeShape {
    pos: Pos2,
    label_text: String,
    selected: bool,
    dragged: bool,

    radius: f32,
}

impl<N: Clone> From<NodeProps<N>> for CustomNodeShape {
    fn from(node_props: NodeProps<N>) -> Self {
        Self {
            pos: node_props.location,
            label_text: node_props.label.to_string(),
            selected: node_props.selected,
            dragged: node_props.dragged,
            radius: 5.0,
        }
    }
}

impl<N: Clone, E: Clone, Ty: EdgeType, Ix: IndexType> DisplayNode<N, E, Ty, Ix> for CustomNodeShape {
    fn closest_boundary_point(&self, dir: Vec2) -> Pos2 {
        closest_point_on_circle(self.pos, self.radius, dir)
    }

    fn shapes(&mut self, ctx: &DrawContext) -> Vec<Shape> {
        let mut res = Vec::with_capacity(2);

        let is_interacted = self.selected || self.dragged;

        let style = match is_interacted {
            true => ctx.ctx.style().visuals.widgets.active,
            false => ctx.ctx.style().visuals.widgets.inactive,
        };

        let color = style.fg_stroke.color;

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
