use egui::{Color32, Pos2};

pub struct PolygonEditor {
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
        egui::CentralPanel::default().show(ctx, |ui| {
            let painter = ui.painter();

            // Important: Order here matters!
            self.draw_polygon_builtin(painter, Color32::LIGHT_GREEN, 1.0);
            self.draw_points(painter, Color32::DARK_BLUE, 4.0);
            // ctrl + LMB
            self.handle_dragging_polygon(ctx);
            // LMB
            self.handle_dragging_points(ctx);
        });
    }
}
