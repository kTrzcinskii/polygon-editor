use egui::{Color32, Pos2};

use crate::point::{EdgeConstraint, Point};

const POINT_WIDTH: f32 = 4.0;
const BEZIER_POINT_COLOR: Color32 = Color32::from_rgb(252, 15, 192);

pub struct Drawer;

impl Drawer {
    pub fn draw_points(points: &[Point], painter: &egui::Painter, color: Color32) {
        #[allow(unused_variables)]
        for (id, point) in points.iter().enumerate() {
            painter.circle(
                *point.pos(),
                POINT_WIDTH,
                color,
                egui::Stroke {
                    color,
                    width: POINT_WIDTH,
                },
            );
            #[cfg(feature = "show_debug_info")]
            {
                painter.text(
                    *point.pos(),
                    egui::Align2::LEFT_TOP,
                    id,
                    egui::FontId::default(),
                    Color32::WHITE,
                );
            }
        }
    }

    pub fn draw_polygon_builtin(
        points: &[Point],
        selected_edge_start_index: Option<usize>,
        painter: &egui::Painter,
        color: Color32,
        special_color: Color32,
        width: f32,
    ) {
        for id in 0..points.len() {
            let current_color = if id == selected_edge_start_index.unwrap_or(usize::MAX) {
                special_color
            } else {
                color
            };
            let id_next = Point::get_next_index(points, id);
            painter.line_segment(
                [*points[id].pos(), *points[id_next].pos()],
                egui::Stroke {
                    color: current_color,
                    width,
                },
            );
            Self::draw_edge_info(points, id, painter);
            if points[id].is_start_of_bezier_segment() {
                Self::draw_brezier_segment(&points[id], &points[id_next], painter);
            }
        }
    }

    pub fn draw_incomplete_polygon_builtin(
        points: &[Point],
        painter: &egui::Painter,
        color: Color32,
        width: f32,
    ) {
        if points.is_empty() {
            return;
        }
        for id in 0..points.len() - 1 {
            painter.line_segment(
                [
                    *points[id].pos(),
                    *points[Point::get_next_index(points, id)].pos(),
                ],
                egui::Stroke { color, width },
            );
        }
    }

    pub fn draw_polygon_bresenham(
        points: &[Point],
        selected_edge_start_index: Option<usize>,
        painter: &egui::Painter,
        color: Color32,
        special_color: Color32,
    ) {
        const WIDTH: f32 = 1.0;
        for id in 0..points.len() {
            let current_color = if id == selected_edge_start_index.unwrap_or(usize::MAX) {
                special_color
            } else {
                color
            };
            let id_next = Point::get_next_index(points, id);
            Self::draw_line_bresenham(painter, current_color, points[id], points[id_next], WIDTH);
            Self::draw_edge_info(points, id, painter);
            if points[id].is_start_of_bezier_segment() {
                Self::draw_brezier_segment(&points[id], &points[id_next], painter);
            }
        }
    }

    pub fn draw_incomplete_polygon_bresenham(
        points: &[Point],
        painter: &egui::Painter,
        color: Color32,
    ) {
        const WIDTH: f32 = 1.0;
        if points.is_empty() {
            return;
        }
        for id in 0..points.len() - 1 {
            Self::draw_line_bresenham(
                painter,
                color,
                points[id],
                points[Point::get_next_index(points, id)],
                WIDTH,
            );
            Self::draw_edge_info(points, id, painter);
        }
    }

    fn draw_edge_info(points: &[Point], id: usize, painter: &egui::Painter) {
        let id_next = Point::get_next_index(points, id);
        let mut pos = Point::get_middle_point(&points[id], &points[id_next]);

        let text = match points[id].constraint() {
            Some(c) => match c {
                EdgeConstraint::Horizontal => "H",
                EdgeConstraint::Vertical => {
                    pos.x += 10.0;
                    "V"
                }
                EdgeConstraint::ConstWidth(width) => &format!("C({})", width),
            },
            None => "",
        };

        painter.text(
            pos,
            egui::Align2::CENTER_TOP,
            text,
            egui::FontId::monospace(24.0),
            Color32::LIGHT_BLUE,
        );

        #[cfg(feature = "show_debug_info")]
        {
            pos.x -= 10.0;
            pos.y -= 10.0;
            let width = points[id].pos().distance(*points[id_next].pos());
            painter.text(
                pos,
                egui::Align2::LEFT_TOP,
                width,
                egui::FontId::default(),
                Color32::WHITE,
            );
        }
    }

    fn draw_line_bresenham(
        painter: &egui::Painter,
        color: Color32,
        start: Point,
        end: Point,
        width: f32,
    ) {
        let x1 = start.pos().x as i32;
        let y1 = start.pos().y as i32;
        let x2 = end.pos().x as i32;
        let y2 = end.pos().y as i32;

        let dx = x2 - x1;
        let dy = y2 - y1;

        let abs_dx = dx.abs();
        let abs_dy = dy.abs();

        let mut x = x1;
        let mut y = y1;

        Self::paint_pixel(
            painter,
            Pos2 {
                x: x as f32,
                y: y as f32,
            },
            width,
            color,
        );

        if abs_dx > abs_dy {
            let mut d = 2 * abs_dy - abs_dx;
            for _ in 0..abs_dx {
                x = if dx < 0 { x - 1 } else { x + 1 };
                if d < 0 {
                    d += 2 * abs_dy
                } else {
                    y = if dy < 0 { y - 1 } else { y + 1 };
                    d += 2 * abs_dy - 2 * abs_dx;
                }
                Self::paint_pixel(
                    painter,
                    Pos2 {
                        x: x as f32,
                        y: y as f32,
                    },
                    width,
                    color,
                );
            }
        } else {
            let mut d = 2 * abs_dx - abs_dy;
            for _ in 0..abs_dy {
                y = if dy < 0 { y - 1 } else { y + 1 };
                if d < 0 {
                    d += 2 * abs_dx
                } else {
                    x = if dx < 0 { x - 1 } else { x + 1 };
                    d += 2 * abs_dx - 2 * abs_dy;
                }
                Self::paint_pixel(
                    painter,
                    Pos2 {
                        x: x as f32,
                        y: y as f32,
                    },
                    width,
                    color,
                );
            }
        }
    }

    fn draw_dashed_line_bresenham(
        painter: &egui::Painter,
        color: Color32,
        start: Pos2,
        end: Pos2,
        width: f32,
    ) {
        const STEP: i32 = 8;
        const STEP_MAX: i32 = 3;

        let x1 = start.x as i32;
        let y1 = start.y as i32;
        let x2 = end.x as i32;
        let y2 = end.y as i32;

        let dx = x2 - x1;
        let dy = y2 - y1;

        let abs_dx = dx.abs();
        let abs_dy = dy.abs();

        let mut x = x1;
        let mut y = y1;

        Self::paint_pixel(
            painter,
            Pos2 {
                x: x as f32,
                y: y as f32,
            },
            width,
            color,
        );

        let mut counter = 0;

        if abs_dx > abs_dy {
            let mut d = 2 * abs_dy - abs_dx;
            for _ in 0..abs_dx {
                x = if dx < 0 { x - 1 } else { x + 1 };
                if d < 0 {
                    d += 2 * abs_dy
                } else {
                    y = if dy < 0 { y - 1 } else { y + 1 };
                    d += 2 * abs_dy - 2 * abs_dx;
                }
                if counter % STEP <= STEP_MAX {
                    Self::paint_pixel(
                        painter,
                        Pos2 {
                            x: x as f32,
                            y: y as f32,
                        },
                        width,
                        color,
                    );
                }
                counter += 1;
            }
        } else {
            let mut d = 2 * abs_dx - abs_dy;
            for _ in 0..abs_dy {
                y = if dy < 0 { y - 1 } else { y + 1 };
                if d < 0 {
                    d += 2 * abs_dx
                } else {
                    x = if dx < 0 { x - 1 } else { x + 1 };
                    d += 2 * abs_dx - 2 * abs_dy;
                }
                if counter % STEP <= STEP_MAX {
                    Self::paint_pixel(
                        painter,
                        Pos2 {
                            x: x as f32,
                            y: y as f32,
                        },
                        width,
                        color,
                    );
                }
                counter += 1;
            }
        }
    }

    fn draw_brezier_segment(start: &Point, end: &Point, painter: &egui::Painter) {
        let bezier_data = start
            .bezier_data()
            .expect("draw_bezier_segment should only be call for point with bezier data");
        let inner_points = bezier_data.inner_points();
        for inner_point in inner_points {
            painter.circle(
                *inner_point,
                POINT_WIDTH,
                BEZIER_POINT_COLOR,
                egui::Stroke {
                    color: BEZIER_POINT_COLOR,
                    width: POINT_WIDTH,
                },
            );
        }
        let all_points = [*start.pos(), inner_points[0], inner_points[1], *end.pos()];
        for id in 0..all_points.len() {
            let id_next = (id + 1) % all_points.len();
            Self::draw_dashed_line_bresenham(
                painter,
                Color32::GRAY,
                all_points[id],
                all_points[id_next],
                1.0,
            );
        }
    }

    fn paint_pixel(painter: &egui::Painter, position: Pos2, width: f32, color: Color32) {
        let rect = egui::Rect::from_min_size(position, egui::Vec2::new(width, width));
        painter.rect_filled(rect, 0.0, color);
    }
}
