use eframe::emath::{Pos2, Vec2};
use eframe::epaint::Shape;
use egui::{Color32, Stroke};
use egui_graphs::{DisplayEdge, DisplayNode, DrawContext, EdgeProps, Metadata, Node};
use petgraph::stable_graph::IndexType;
use petgraph::EdgeType;

// Based on DefaultEdgeShape

trait SeparatorInfo {
    fn get_is_separator(&self) -> bool;
}

#[derive(Clone, Debug)]
pub(crate) struct EdgeData {
    is_separator: bool,
}

impl EdgeData {
    pub(crate) fn new(is_separator: bool) -> Self {
        Self {
            is_separator,
        }
    }
}

impl SeparatorInfo for EdgeData {
    fn get_is_separator(&self) -> bool {
        self.is_separator
    }
}

const SEPARATOR: Color32 = Color32::from_rgb(0x90, 0xEE, 0x90);

#[derive(Clone)]
pub(crate) struct CustomEdgeShape {
    order: usize,
    selected: bool,
    label_text: String,

    width: f32,
    tip_size: f32,
    tip_angle: f32,
    is_separator: bool,
}

impl<E: Clone + SeparatorInfo> From<EdgeProps<E>> for CustomEdgeShape {
    fn from(edge_props: EdgeProps<E>) -> Self {
        assert_eq!(0usize, edge_props.order, "CustomEdgeShape only renders simple graphs (order 0)");
        Self {
            order: edge_props.order,
            selected: edge_props.selected,
            label_text: edge_props.label.to_string(),

            width: 2.,
            tip_size: 12.5,
            tip_angle: std::f32::consts::TAU / 30.,
            is_separator: edge_props.payload.get_is_separator(),
        }
    }
}

impl CustomEdgeShape {
    fn get_tip_points(&mut self, is_directed: bool, start: Pos2, end: Pos2, line_points: &mut Vec<Pos2>) -> Vec<Pos2> {
        if !is_directed {
            return vec![];
        }

        let tip_dir = (end - start).normalized();
        let tip_angle = self.tip_angle;
        let tip_size = self.tip_size;

        let arrow_tip_dir_1 = rotate_vector(tip_dir, tip_angle) * tip_size;
        let arrow_tip_dir_2 = rotate_vector(tip_dir, -tip_angle) * tip_size;

        let tip_start_1 = end - arrow_tip_dir_1;
        let tip_start_2 = end - arrow_tip_dir_2;

        // replace end of an edge with start of tip
        *line_points.get_mut(1).unwrap() = end - tip_size * tip_dir;

        vec![end, tip_start_1, tip_start_2]
    }

    fn scale_stroke(metadata: &Metadata, stroke: &mut Stroke) {
        stroke.width = metadata.canvas_to_screen_size(stroke.width);
    }

    fn scale_points(metadata: &Metadata, points: &mut Vec<Pos2>) {
        for i in 0..points.len() {
            *points.get_mut(i).unwrap() = metadata.canvas_to_screen_pos(points[i]);
        }
    }
}

impl<N: Clone, E: Clone + SeparatorInfo, Ty: EdgeType, Ix: IndexType, D: DisplayNode<N, E, Ty, Ix>>
DisplayEdge<N, E, Ty, Ix, D> for CustomEdgeShape {
    fn shapes(&mut self, start_node: &Node<N, E, Ty, Ix, D>, end_node: &Node<N, E, Ty, Ix, D>, ctx: &DrawContext) -> Vec<Shape> {
        // Note that we assume the graphs we're working with to be simple graphs
        let mut res = vec![];

        let color = match self.is_separator {
            true => SEPARATOR,
            false => {
                let style = match self.selected {
                    true => ctx.ctx.style().visuals.widgets.active,
                    false => ctx.ctx.style().visuals.widgets.inactive,
                };
                style.fg_stroke.color
            }
        };

        let mut stroke = Stroke::new(self.width, color);

        let dir = (end_node.location() - start_node.location()).normalized();
        let start = start_node.display().closest_boundary_point(dir);
        let end = end_node.display().closest_boundary_point(-dir);

        let mut line_points = vec![start, end];
        let mut tip_points = self.get_tip_points(ctx.is_directed, start, end, &mut line_points);

        Self::scale_stroke(ctx.meta, &mut stroke);
        Self::scale_points(ctx.meta, &mut line_points);
        Self::scale_points(ctx.meta, &mut tip_points);

        res.push(Shape::line_segment(
            [line_points[0], line_points[1]],
            stroke,
        ));

        if ctx.is_directed {
            res.push(Shape::convex_polygon(
                tip_points,
                stroke.color,
                Stroke::default(),
            ));
        }

        // we don't draw the label

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

fn rotate_vector(vec: Vec2, angle: f32) -> Vec2 {
    let cos = angle.cos();
    let sin = angle.sin();
    Vec2::new(cos * vec.x - sin * vec.y, sin * vec.x + cos * vec.y)
}
