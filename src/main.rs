mod drawer;
mod point;
mod polygon_editor;
mod popups;

use polygon_editor::PolygonEditor;

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
