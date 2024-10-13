use egui::{Color32, Pos2, Rounding, Vec2};

use crate::{drawer::Drawer, edge::Edge, point::Point};

#[derive(PartialEq)]
enum LineDrawingAlgorithm {
    Bultin,
    Bresenham,
}

pub struct PolygonEditor {
    /// Which line drawing algorithm to use
    line_drawing_algorithm: LineDrawingAlgorithm,
    /// List of all polygon points
    points: Vec<Point>,
    /// List of all polygon edges
    edges: Vec<Edge>,
    /// Id of point inside points that is currently being dragged by user
    dragged_index: Option<usize>,
    /// Id of point inside points that is currently used to dragg whole polygon
    polygon_dragged_index: Option<usize>,
    /// Id of edge currently selected for context menu
    selected_edge: Option<usize>,
}

impl PolygonEditor {
    pub fn handle_dragging_points(&mut self, ctx: &egui::Context) {
        let mouse_pos = ctx.pointer_interact_pos();
        if let Some(pos) = mouse_pos {
            // Check user is holding LMB
            if ctx.input(|i| i.pointer.button_down(egui::PointerButton::Primary)) {
                // If already dragging then move point
                if let Some(index) = self.dragged_index {
                    Point::update_position(index, pos, &mut self.points, &self.edges);
                } else {
                    for (i, point) in self.points.iter().enumerate() {
                        // Start dragging the point if it's close enough
                        if (*point.pos() - pos).length() < 10.0 {
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

    // We are moving whole polygon, so we dont have to check restrictions here
    // As the relative positions of points is unchanged
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
                    let diff = pos - *previous_pos.pos();
                    Point::update_position_all(&mut self.points, diff);
                } else {
                    for (i, point) in self.points.iter().enumerate() {
                        // Start dragging the point if it's close enough
                        if (*point.pos() - pos).length() < 10.0 {
                            self.polygon_dragged_index = Some(i);
                        }
                    }
                }
            } else {
                self.polygon_dragged_index = None;
            }
        }
    }

    pub fn handle_selecting_edge(&mut self, ctx: &egui::Context) {
        let mouse_pos = ctx.pointer_hover_pos();
        if let Some(pos) = mouse_pos {
            if ctx.input(|i| i.pointer.button_down(egui::PointerButton::Secondary)) {
                let mut selected_now = false;
                for (id, edge) in self.edges.iter().enumerate() {
                    if edge.contains_point(&self.points, &pos) {
                        self.selected_edge = Some(id);
                        selected_now = true;
                        break;
                    }
                }
                if !selected_now {
                    self.selected_edge = None;
                }
            }
        }
    }

    pub fn handle_removing_point(&mut self, ctx: &egui::Context) {
        if self.points.len() <= 3 {
            return;
        }
        let mouse_pos = ctx.pointer_hover_pos();
        if let Some(pos) = mouse_pos {
            if ctx.input(|i| i.pointer.button_down(egui::PointerButton::Primary) && i.modifiers.alt)
            {
                let mut id: Option<usize> = None;
                for (i, point) in self.points.iter().enumerate() {
                    if (*point.pos() - pos).length() < 10.0 {
                        id = Some(i)
                    }
                }
                if let Some(id) = id {
                    Point::remove_at(&mut self.points, &mut self.edges, id);
                }
            }
        }
    }

    pub fn show_context_menu_for_selected_edge(&mut self, ctx: &egui::Context, ui: &egui::Ui) {
        const CONTEXT_MENU_MIN_WDITH: f32 = 120.0;
        if let Some(selected_id) = self.selected_edge {
            let edge = &self.edges[selected_id];

            let neighbour_has_vertical_or_horizontal_restriction =
                Edge::neighours_have_vertical_or_horizontal_restriction(&self.edges, selected_id);

            let can_add_restriction = !edge.has_restriction();
            let number_of_buttons = if can_add_restriction { 4 } else { 2 };

            let container_pos = Point::get_middle_point(
                &self.points[edge.start_index],
                &self.points[edge.end_index],
            ) - Vec2::new(
                CONTEXT_MENU_MIN_WDITH / 2.0,
                ui.spacing().interact_size.y * number_of_buttons as f32 / 2.0,
            );
            egui::containers::Area::new("edge_context_menu".into())
                .fixed_pos(container_pos)
                .show(ctx, |ui| {
                    egui::Frame::popup(ui.style())
                        .outer_margin(0.0)
                        .inner_margin(0.0)
                        .fill(Color32::TRANSPARENT)
                        .show(ui, |ui| {
                            ui.set_min_width(CONTEXT_MENU_MIN_WDITH);
                            ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                            ui.with_layout(
                                egui::Layout::top_down_justified(egui::Align::LEFT),
                                |ui| {
                                    if ui
                                        .add(egui::Button::new("Add midpoint").rounding(Rounding {
                                            sw: 0.0,
                                            se: 0.0,
                                            ..Default::default()
                                        }))
                                        .clicked()
                                    {
                                        Point::add_on_edge(
                                            &mut self.points,
                                            &mut self.edges,
                                            selected_id,
                                        );
                                        self.selected_edge = None;
                                    }
                                    if can_add_restriction {
                                        if ui
                                            .add_enabled(
                                                !neighbour_has_vertical_or_horizontal_restriction,
                                                egui::Button::new("Make horizontal")
                                                    .rounding(Rounding::ZERO),
                                            )
                                            .clicked()
                                        {
                                            self.edges[selected_id].apply_horizontal_restriction();
                                            self.points[self.edges[selected_id].start_index]
                                                .pos_mut()
                                                .y = self.points[self.edges[selected_id].end_index]
                                                .pos()
                                                .y;
                                            self.selected_edge = None;
                                        }
                                        if ui
                                            .add_enabled(
                                                !neighbour_has_vertical_or_horizontal_restriction,
                                                egui::Button::new("Make vertical")
                                                    .rounding(Rounding::ZERO),
                                            )
                                            .clicked()
                                        {
                                            self.edges[selected_id].apply_vertical_restriction();
                                            self.points[self.edges[selected_id].start_index]
                                                .pos_mut()
                                                .x = self.points[self.edges[selected_id].end_index]
                                                .pos()
                                                .x;
                                            self.selected_edge = None;
                                        }
                                        if ui
                                            .add(egui::Button::new("Make constant width").rounding(
                                                Rounding {
                                                    nw: 0.0,
                                                    ne: 0.0,
                                                    ..Default::default()
                                                },
                                            ))
                                            .clicked()
                                        {
                                            self.edges[selected_id].apply_width_restriction();
                                            self.selected_edge = None;
                                        }
                                    } else if ui
                                        .add(egui::Button::new("Remove restriction").rounding(
                                            Rounding {
                                                nw: 0.0,
                                                ne: 0.0,
                                                ..Default::default()
                                            },
                                        ))
                                        .clicked()
                                    {
                                        self.edges[selected_id].remove_restriction();
                                        self.selected_edge = None;
                                    }
                                },
                            );
                        });
                });
        }
    }
}

impl Default for PolygonEditor {
    fn default() -> Self {
        let points = vec![
            Point::new(Pos2::new(50.0, 50.0)),
            Point::new(Pos2::new(100.0, 50.0)),
            Point::new(Pos2::new(75.0, 100.0)),
        ];
        let edges = Edge::from_points(&points);
        Self {
            line_drawing_algorithm: LineDrawingAlgorithm::Bresenham,
            points,
            edges,
            dragged_index: None,
            polygon_dragged_index: None,
            selected_edge: None,
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
                ui.vertical_centered(|ui| {
                    if ui.button("Restore default state").clicked() {
                        *self = Self::default();
                    }
                });
                ui.separator();
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let painter = ui.painter();

            // Important: Order here matters!
            match self.line_drawing_algorithm {
                LineDrawingAlgorithm::Bultin => Drawer::draw_polygon_builtin(
                    &self.points,
                    &self.edges,
                    self.selected_edge,
                    painter,
                    Color32::LIGHT_GREEN,
                    Color32::ORANGE,
                    1.0,
                ),
                LineDrawingAlgorithm::Bresenham => Drawer::draw_polygon_bresenham(
                    &self.points,
                    &self.edges,
                    self.selected_edge,
                    painter,
                    Color32::YELLOW,
                    Color32::ORANGE,
                ),
            };
            Drawer::draw_points(&self.points, painter, Color32::DARK_BLUE, 4.0);
            // ctrl + LMB on point
            self.handle_dragging_polygon(ctx);
            // alt + LMB on point
            self.handle_removing_point(ctx);
            // LMB on point
            self.handle_dragging_points(ctx);
            // RMB on edge
            self.handle_selecting_edge(ctx);
            self.show_context_menu_for_selected_edge(ctx, ui);
        });
    }
}
