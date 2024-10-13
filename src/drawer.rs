use egui::{Color32, Pos2};

use crate::point::Point;

pub struct Drawer;

impl Drawer {
    pub fn draw_points(points: &[Point], painter: &egui::Painter, color: Color32, width: f32) {
        for point in points {
            painter.circle(*point.pos(), width, color, egui::Stroke { color, width });
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
            painter.line_segment(
                [
                    *points[id].pos(),
                    *points[Point::get_next_index(points, id)].pos(),
                ],
                egui::Stroke {
                    color: current_color,
                    width,
                },
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
            Self::draw_line_bresenham(
                painter,
                current_color,
                points[id],
                points[Point::get_next_index(points, id)],
                WIDTH,
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

    fn paint_pixel(painter: &egui::Painter, position: Pos2, width: f32, color: Color32) {
        let rect = egui::Rect::from_min_size(position, egui::Vec2::new(width, width));
        painter.rect_filled(rect, 0.0, color);
    }
}
