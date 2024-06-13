use eframe::emath::{Pos2, Vec2};
use eframe::epaint::Shape;
use egui::Stroke;
use egui_graphs::{DisplayEdge, DisplayNode, DrawContext, EdgeProps, Node};
use petgraph::stable_graph::IndexType;
use petgraph::EdgeType;

#[derive(Clone)]
pub(crate) struct CustomEdgeShape {
    order: usize,
    selected: bool,
    label_text: String,

    width: f32,
}

impl<E: Clone> From<EdgeProps<E>> for CustomEdgeShape {
    fn from(edge_props: EdgeProps<E>) -> Self {
        assert_eq!(0usize, edge_props.order, "CustomEdgeShape only renders simple graphs (order 0)");
        Self {
            order: edge_props.order,
            selected: edge_props.selected,
            label_text: edge_props.label.to_string(),

            width: 2.,
        }
    }
}

impl<N: Clone, E: Clone, Ty: EdgeType, Ix: IndexType, D: DisplayNode<N, E, Ty, Ix>>
DisplayEdge<N, E, Ty, Ix, D> for CustomEdgeShape {
    fn shapes(&mut self, start: &Node<N, E, Ty, Ix, D>, end: &Node<N, E, Ty, Ix, D>, ctx: &DrawContext) -> Vec<Shape> {
        // Note that we assume the graphs we're working with to be simple directed graphs
        // TODO Modify this to work for undirected graphs as well
        let mut res = vec![];

        let style = match self.selected {
            true => ctx.ctx.style().visuals.widgets.active,
            false => ctx.ctx.style().visuals.widgets.inactive,
        };
        let color = style.fg_stroke.color;
        let mut stroke = Stroke::new(self.width, color);

        let dir = (end.location() - start.location()).normalized();
        let start_connector_point = start.display().closest_boundary_point(dir);
        let end_connector_point = end.display().closest_boundary_point(-dir);
        let mut points = vec![start_connector_point, end_connector_point];

        let metadata = ctx.meta;
        stroke.width = metadata.canvas_to_screen_size(stroke.width);
        points = points.iter().map(|pos2| metadata.canvas_to_screen_pos(*pos2)).collect();

        res.push(Shape::line_segment(
            [points[0], points[1]],
            stroke,
        ));

        // TODO Add tip

        res
    }

    fn update(&mut self, state: &EdgeProps<E>) {
        self.order = state.order;
        self.selected = state.selected;
        self.label_text = state.label.to_string();
    }

    fn is_inside(&self, start: &Node<N, E, Ty, Ix, D>, end: &Node<N, E, Ty, Ix, D>, pos: Pos2) -> bool {
        let pos_start = start.location();
        let pos_end = end.location();

        let distance = distance_segment_to_point(pos_start, pos_end, pos);
        distance <= self.width
    }
}

fn distance_segment_to_point(a: Pos2, b: Pos2, point: Pos2) -> f32 {
    let ac = point - a;
    let ab = b - a;
    let d = a + proj(ac, ab);
    let ad = d - a;

    let k = if ab.x.abs() > ab.y.abs() {
        ad.x / ab.x
    } else {
        ad.y / ab.y
    };

    if k <= 0.0 {
        return hypot2(point.to_vec2(), a.to_vec2()).sqrt();
    } else if k >= 1.0 {
        return hypot2(point.to_vec2(), b.to_vec2()).sqrt();
    }

    hypot2(point.to_vec2(), d.to_vec2()).sqrt()
}

fn hypot2(a: Vec2, b: Vec2) -> f32 {
    (a - b).dot(a - b)
}

fn proj(a: Vec2, b: Vec2) -> Vec2 {
    let k = a.dot(b) / b.dot(b);
    Vec2::new(k * b.x, k * b.y)
}
