use egui::{Color32, Pos2};

#[derive(PartialEq)]
enum LineDrawingAlgorithm {
    Bultin,
    Bresenham,
}

pub struct PolygonEditor {
    /// Which line drawing algorithm to use
    line_drawing_algorithm: LineDrawingAlgorithm,
    /// List of all polygon points
    points: Vec<Pos2>,
    /// Id of point inside points that is currently being dragged by user
    dragged_index: Option<usize>,
    /// Id of point inside points that is currently used to dragg whole polygon
    polygon_dragged_index: Option<usize>,
}

impl PolygonEditor {
    pub fn draw_points(&self, painter: &egui::Painter, color: Color32, width: f32) {
        for point in &self.points {
            painter.circle(*point, width, color, egui::Stroke { color, width });
        }
    }

    pub fn draw_polygon_builtin(&self, painter: &egui::Painter, color: Color32, width: f32) {
        for i in 0..self.points.len() {
            let start = self.points[i];
            let end = self.points[(i + 1) % self.points.len()];
            painter.line_segment([start, end], egui::Stroke { color, width });
        }
    }

    fn paint_pixel(&self, painter: &egui::Painter, position: Pos2, width: f32, color: Color32) {
        let rect = egui::Rect::from_min_size(position, egui::Vec2::new(width, width));
        painter.rect_filled(rect, 0.0, color);
    }

    fn draw_line_bresenham(
        &self,
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

        self.paint_pixel(
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
                self.paint_pixel(
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
                self.paint_pixel(
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

    pub fn draw_polygon_bresenham(&self, painter: &egui::Painter, color: Color32) {
        const WIDTH: f32 = 1.0;
        for i in 0..self.points.len() {
            let start = self.points[i];
            let end = self.points[(i + 1) % self.points.len()];
            self.draw_line_bresenham(painter, color, start, end, WIDTH);
        }
    }

    pub fn handle_dragging_points(&mut self, ctx: &egui::Context) {
        let mouse_pos = ctx.pointer_interact_pos();
        if let Some(pos) = mouse_pos {
            // Check user is holding LMB
            if ctx.input(|i| i.pointer.button_down(egui::PointerButton::Primary)) {
                // If already dragging then move point
                if let Some(index) = self.dragged_index {
                    self.points[index] = pos;
                } else {
                    for (i, point) in self.points.iter().enumerate() {
                        // Start dragging the point if it's close enough
                        if (*point - pos).length() < 10.0 {
                            self.dragged_index = Some(i);
                        }
                    }
                }
            } else {
                // Stop dragging if LMB no longer hold
                self.dragged_index = None;
            }
        }
    }

    pub fn handle_dragging_polygon(&mut self, ctx: &egui::Context) {
        let mouse_pos = ctx.pointer_interact_pos();
        if let Some(pos) = mouse_pos {
            // Check if user is holding ctrl + LMB
            if ctx
                .input(|i| i.pointer.button_down(egui::PointerButton::Primary) && i.modifiers.ctrl)
            {
                // If already dragging then move all points
                if let Some(index) = self.polygon_dragged_index {
                    let previous_pos = self.points[index];
                    let diff = pos - previous_pos;
                    for point in &mut self.points {
                        *point += diff;
                    }
                } else {
                    for (i, point) in self.points.iter().enumerate() {
                        // Start dragging the point if it's close enough
                        if (*point - pos).length() < 10.0 {
                            self.polygon_dragged_index = Some(i);
                        }
                    }
                }
            } else {
                self.polygon_dragged_index = None;
            }
        }
    }
}

impl Default for PolygonEditor {
    fn default() -> Self {
        Self {
            line_drawing_algorithm: LineDrawingAlgorithm::Bresenham,
            points: vec![
                Pos2::new(50.0, 50.0),
                Pos2::new(100.0, 50.0),
                Pos2::new(75.0, 100.0),
            ],
            dragged_index: None,
            polygon_dragged_index: None,
        }
    }
}

impl eframe::App for PolygonEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::right("right_panel")
            .resizable(false)
            .frame(
                egui::Frame::none()
                    .fill(egui::Color32::from_gray(40))
                    .inner_margin(egui::Margin::same(10.0)),
            )
            .show(ctx, |ui| {
                ui.heading("Controls");
                ui.separator();
                ui.label("Choose line drawing algorithm");
                ui.radio_value(
                    &mut self.line_drawing_algorithm,
                    LineDrawingAlgorithm::Bresenham,
                    "Bresenham Algorithm",
                );
                ui.radio_value(
                    &mut self.line_drawing_algorithm,
                    LineDrawingAlgorithm::Bultin,
                    "Builtin Algorithm",
                );
                ui.separator();
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let painter = ui.painter();

            // Important: Order here matters!
            match self.line_drawing_algorithm {
                LineDrawingAlgorithm::Bultin => {
                    self.draw_polygon_builtin(painter, Color32::LIGHT_GREEN, 1.0)
                }
                LineDrawingAlgorithm::Bresenham => {
                    self.draw_polygon_bresenham(painter, Color32::YELLOW)
                }
            };
            self.draw_points(painter, Color32::DARK_BLUE, 4.0);
            // ctrl + LMB
            self.handle_dragging_polygon(ctx);
            // LMB
            self.handle_dragging_points(ctx);
        });
    }
}
