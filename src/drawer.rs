use egui::{Color32, Pos2};

use crate::edge::Edge;

pub struct Drawer;

impl Drawer {
    pub fn draw_points(points: &[Pos2], painter: &egui::Painter, color: Color32, width: f32) {
        for point in points {
            painter.circle(*point, width, color, egui::Stroke { color, width });
        }
    }

    pub fn draw_polygon_builtin(
        points: &[Pos2],
        edges: &[Edge],
        selected_edge: Option<usize>,
        painter: &egui::Painter,
        color: Color32,
        special_color: Color32,
        width: f32,
    ) {
        for (id, edge) in edges.iter().enumerate() {
            let current_color = if id == selected_edge.unwrap_or(usize::MAX) {
                special_color
            } else {
                color
            };
            painter.line_segment(
                [points[edge.start_index], points[edge.end_index]],
                egui::Stroke {
                    color: current_color,
                    width,
                },
            );
        }
    }

    pub fn draw_polygon_bresenham(
        points: &[Pos2],
        edges: &[Edge],
        selected_edge: Option<usize>,
        painter: &egui::Painter,
        color: Color32,
        special_color: Color32,
    ) {
        const WIDTH: f32 = 1.0;
        for (id, edge) in edges.iter().enumerate() {
            let current_color = if id == selected_edge.unwrap_or(usize::MAX) {
                special_color
            } else {
                color
            };
            Self::draw_line_bresenham(
                painter,
                current_color,
                points[edge.start_index],
                points[edge.end_index],
                WIDTH,
            );
        }
    }

    fn draw_line_bresenham(
        painter: &egui::Painter,
        color: Color32,
        start: Pos2,
        end: Pos2,
        width: f32,
    ) {
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
