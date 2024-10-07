use eframe::egui;
use egui::{Color32, Pos2};

struct PolygonEditor {
    /// List of all polygon points
    points: Vec<Pos2>,
    /// Id of point inside points that is currently being dragged by user
    dragged_index: Option<usize>,
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
        }
    }
}

impl eframe::App for PolygonEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let painter = ui.painter();

            self.draw_polygon_builtin(painter, Color32::LIGHT_GREEN, 1.0);
            self.draw_points(painter, Color32::DARK_BLUE, 4.0);

            // TODO: handle dragging point
        });
    }
}

fn main() {
    let app = PolygonEditor::default();
    let native_options = eframe::NativeOptions::default();
    let res = eframe::run_native(
        "Polygon Editor",
        native_options,
        Box::new(|_cc| Ok(Box::new(app))),
    );

    if let Err(e) = res {
        eprintln!("Error during `eframe::run_native`: {}", e)
    }
}
