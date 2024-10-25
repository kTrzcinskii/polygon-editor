use egui::{Color32, Pos2, Rounding, Vec2};

use crate::{drawer::Drawer, point::Point, popups::Popups};

#[derive(PartialEq)]
enum LineDrawingAlgorithm {
    Bultin,
    Bresenham,
}

#[derive(PartialEq)]
enum PolygonMode {
    Drawing,
    Editing,
}

pub struct PolygonEditor {
    polygon_mode: PolygonMode,
    /// Which line drawing algorithm to use
    line_drawing_algorithm: LineDrawingAlgorithm,
    /// List of all polygon points
    /// At the same time, each point is the start of the edge and the next one is the end of it
    points: Vec<Point>,
    /// Id of point inside points that is currently being dragged by user
    dragged_index: Option<usize>,
    /// Bezier control point that is currenlty dragged: (point id, id of control point in that point bezier data)
    bezier_control_point_dragged: Option<(usize, usize)>,
    /// Id of point inside points that is currently used to dragg whole polygon
    polygon_dragged_index: Option<usize>,
    /// Id of edge (meaning id of the first vertex of it) currently selected for context menu
    selected_edge_start_index: Option<usize>,
    /// Id of point currently selected for context menu
    selected_point_index: Option<usize>,
    /// Data related to all popups
    popups: Popups,
}

impl PolygonEditor {
    const CONTEXT_MENU_MIN_WDITH: f32 = 150.0;

    pub fn new_with_drawing_mode() -> Self {
        Self {
            polygon_mode: PolygonMode::Drawing,
            points: vec![],
            ..Default::default()
        }
    }

    pub fn handle_dragging_points(&mut self, ctx: &egui::Context) {
        let mouse_pos = ctx.pointer_interact_pos();
        if let Some(pos) = mouse_pos {
            // Check user is holding LMB
            if ctx.input(|i| i.pointer.button_down(egui::PointerButton::Primary)) {
                // If already dragging then move point
                if let Some(index) = self.dragged_index {
                    Point::update_position(&mut self.points, index, pos);
                } else if let Some((point_index, inner_point_index)) =
                    self.bezier_control_point_dragged
                {
                    match self.points[point_index].bezier_data_mut() {
                        Some(bd) => {
                            bd.update_inner_point_position(inner_point_index, pos);
                            Point::update_position_after_control_point_moved(
                                &mut self.points,
                                point_index,
                                inner_point_index,
                            )
                        }

                        None => eprintln!(
                            "Trying to move bezier control point for point without bezier segment"
                        ),
                    }
                } else {
                    for (i, point) in self.points.iter().enumerate() {
                        // Start dragging the point if it's close enough
                        if (*point.pos() - pos).length() < 10.0 {
                            self.dragged_index = Some(i);
                            break;
                        }
                        if let Some(bezier_data) = point.bezier_data() {
                            for (ip, inner_point) in bezier_data.inner_points().iter().enumerate() {
                                if (*inner_point - pos).length() < 10.0 {
                                    self.bezier_control_point_dragged = Some((i, ip));
                                    break;
                                }
                            }
                        }
                    }
                }
            } else {
                // Stop dragging if LMB no longer hold
                self.dragged_index = None;
                self.bezier_control_point_dragged = None;
            }
        }
    }

    pub fn handle_adding_point_in_drawing_mode(
        &mut self,
        ctx: &egui::Context,
        main_panel_width: f32,
    ) {
        let mouse_pos = ctx.pointer_interact_pos();
        if let Some(pos) = mouse_pos {
            // If clicking outside the panel (on controls panel) ignore this click
            if pos.x > main_panel_width {
                return;
            }
            if ctx.input(|i| i.pointer.button_clicked(egui::PointerButton::Primary)) {
                if self.points.len() >= 3 && (*self.points[0].pos() - pos).length() < 10.0 {
                    self.polygon_mode = PolygonMode::Editing;
                }
                // It means that we didnt change the mode, so user wants to add new point
                if self.polygon_mode == PolygonMode::Drawing {
                    self.points.push(Point::new(pos));
                }
            }
        }
    }

    // We are moving whole polygon, so we dont have to check constraints here
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

    pub fn handle_selecting_edge_or_point(&mut self, ctx: &egui::Context) {
        let mouse_pos = ctx.pointer_hover_pos();
        if let Some(pos) = mouse_pos {
            if ctx.input(|i| i.pointer.button_down(egui::PointerButton::Secondary)) {
                let mut edge_selected_now = false;
                let mut point_selected_now = false;
                for id in 0..self.points.len() {
                    if self.points[id].pos().distance(pos) < 10.0
                        && Point::is_part_of_bezier_segment(&self.points, id)
                    {
                        self.selected_point_index = Some(id);
                        point_selected_now = true;
                        break;
                    }
                    if Point::contains_point(&self.points, id, &pos) {
                        self.selected_edge_start_index = Some(id);
                        edge_selected_now = true;
                        break;
                    }
                }
                if !point_selected_now {
                    self.selected_point_index = None;
                }
                if !edge_selected_now {
                    self.selected_edge_start_index = None;
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
                    Point::remove_at(&mut self.points, id);
                }
            }
        }
    }

    pub fn show_context_menu_for_selected_edge(&mut self, ctx: &egui::Context, ui: &egui::Ui) {
        if let Some(selected_id) = self.selected_edge_start_index {
            let can_add_constraint_or_bezier_segment = !self.points[selected_id].has_constraint()
                && !self.points[selected_id].is_start_of_bezier_segment();
            let number_of_buttons = if can_add_constraint_or_bezier_segment {
                5
            } else {
                2
            };

            let container_pos = Point::get_middle_point(
                &self.points[selected_id],
                &self.points[Point::get_next_index(&self.points, selected_id)],
            ) - Vec2::new(
                Self::CONTEXT_MENU_MIN_WDITH / 2.0,
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
                            ui.set_min_width(Self::CONTEXT_MENU_MIN_WDITH);
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
                                        Point::add_on_edge(&mut self.points, selected_id);
                                        self.selected_edge_start_index = None;
                                    }
                                    if can_add_constraint_or_bezier_segment {
                                        // Bezier button
                                        if ui
                                            .add(
                                                egui::Button::new("Change into bezier segment")
                                                    .rounding(Rounding::ZERO),
                                            )
                                            .clicked()
                                        {
                                            let initial_points =
                                                Point::get_points_between_for_initial_bezier(
                                                    &self.points[selected_id],
                                                    &self.points[Point::get_next_index(
                                                        &self.points,
                                                        selected_id,
                                                    )],
                                                );
                                            self.points[selected_id]
                                                .init_bezier_data(initial_points);
                                            let same_pos = *self.points[selected_id].pos();
                                            Point::update_position(
                                                &mut self.points,
                                                selected_id,
                                                same_pos,
                                            );
                                            self.selected_edge_start_index = None;
                                        }
                                        // Horizontal button
                                        if ui
                                            .add_enabled(
                                                !Point::neighour_edges_have_horizontal_constraint(
                                                    &self.points,
                                                    selected_id,
                                                ),
                                                egui::Button::new("Make horizontal")
                                                    .rounding(Rounding::ZERO),
                                            )
                                            .clicked()
                                        {
                                            self.points[selected_id].apply_horizontal_constraint();
                                            let same_pos = *self.points[selected_id].pos();
                                            Point::update_position(
                                                &mut self.points,
                                                selected_id,
                                                same_pos,
                                            );
                                            self.selected_edge_start_index = None;
                                        }
                                        // Vertical button
                                        if ui
                                            .add_enabled(
                                                !Point::neighour_edges_have_vertical_constraint(
                                                    &self.points,
                                                    selected_id,
                                                ),
                                                egui::Button::new("Make vertical")
                                                    .rounding(Rounding::ZERO),
                                            )
                                            .clicked()
                                        {
                                            self.points[selected_id].apply_vertical_constraint();
                                            let same_pos = *self.points[selected_id].pos();
                                            Point::update_position(
                                                &mut self.points,
                                                selected_id,
                                                same_pos,
                                            );
                                            self.selected_edge_start_index = None;
                                        }
                                        // Const width button
                                        let const_width_button = ui.add(
                                            egui::Button::new("Make constant width").rounding(
                                                Rounding {
                                                    nw: 0.0,
                                                    ne: 0.0,
                                                    ..Default::default()
                                                },
                                            ),
                                        );
                                        self.popups
                                            .render_const_width_constraint_popup_below_widget(
                                                ui,
                                                &const_width_button,
                                            );
                                        if const_width_button.clicked() {
                                            let selected_edge_end_index =
                                                Point::get_next_index(&self.points, selected_id);
                                            let width = self.points[selected_id].pos().distance(
                                                *self.points[selected_edge_end_index].pos(),
                                            );

                                            self.popups
                                                .open_const_width_constraint_popup_below_widget(
                                                    ui,
                                                    width.round() as i32,
                                                );
                                        }
                                        if self.popups.const_width_constraint_submitted() {
                                            let new_width =
                                                self.popups.const_width_constraint_user_input();
                                            self.points[selected_id]
                                                .apply_width_constraint(new_width);
                                            let same_pos = *self.points[selected_id].pos();
                                            Point::update_position(
                                                &mut self.points,
                                                selected_id,
                                                same_pos,
                                            );
                                            self.selected_edge_start_index = None;
                                            self.popups.reset_const_width_constraint_submitted();
                                        }
                                    } else if self.points[selected_id].has_constraint() {
                                        let response = ui.add(
                                            egui::Button::new("Remove constraint").rounding(
                                                Rounding {
                                                    nw: 0.0,
                                                    ne: 0.0,
                                                    ..Default::default()
                                                },
                                            ),
                                        );
                                        if response.clicked() {
                                            self.points[selected_id].remove_constraint();
                                            self.selected_edge_start_index = None;
                                        }
                                    }
                                },
                            );
                        });
                });
        }
    }

    pub fn show_context_menu_for_selected_point(&mut self, ctx: &egui::Context) {
        // Only dispaly context menu for point that is either start or end of bezier segment
        if let Some(selected_id) = self.selected_point_index {
            if !Point::is_part_of_bezier_segment(&self.points, selected_id) {
                return;
            }

            let container_pos = *self.points[selected_id].pos() + Vec2::new(10.0, 10.0);
            let display_remove_bezier_button =
                self.points[selected_id].is_start_of_bezier_segment();

            egui::containers::Area::new("edge_context_menu".into())
                .fixed_pos(container_pos)
                .show(ctx, |ui| {
                    egui::Frame::popup(ui.style())
                        .outer_margin(0.0)
                        .inner_margin(0.0)
                        .fill(Color32::TRANSPARENT)
                        .show(ui, |ui| {
                            ui.set_min_width(Self::CONTEXT_MENU_MIN_WDITH);
                            ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                            ui.with_layout(
                                egui::Layout::top_down_justified(egui::Align::LEFT),
                                |ui| {
                                    // G0 button
                                    if ui
                                        .add(egui::Button::new("Apply G0").rounding(Rounding {
                                            sw: 0.0,
                                            se: 0.0,
                                            ..Default::default()
                                        }))
                                        .clicked()
                                    {
                                        self.points[selected_id].apply_G0();
                                        let same_pos = *self.points[selected_id].pos();
                                        Point::update_position(
                                            &mut self.points,
                                            selected_id,
                                            same_pos,
                                        );
                                        self.selected_point_index = None;
                                    }
                                    // G1 button
                                    if ui
                                        .add(egui::Button::new("Apply G1").rounding(Rounding::ZERO))
                                        .clicked()
                                    {
                                        self.points[selected_id].apply_G1();
                                        let same_pos = *self.points[selected_id].pos();
                                        Point::update_position(
                                            &mut self.points,
                                            selected_id,
                                            same_pos,
                                        );
                                        self.selected_point_index = None;
                                    }
                                    // C1 button
                                    if ui
                                        .add(egui::Button::new("Apply C1").rounding(Rounding::ZERO))
                                        .clicked()
                                    {
                                        self.points[selected_id].apply_C1();
                                        let same_pos = *self.points[selected_id].pos();
                                        Point::update_position(
                                            &mut self.points,
                                            selected_id,
                                            same_pos,
                                        );
                                        self.selected_point_index = None;
                                    }
                                    // Remove bezier segment button
                                    if display_remove_bezier_button
                                        && ui
                                            .add(
                                                egui::Button::new("Remove bezier segment")
                                                    .rounding(Rounding::ZERO),
                                            )
                                            .clicked()
                                    {
                                        self.points[selected_id].remove_bezier_data();
                                        let same_pos = *self.points[selected_id].pos();
                                        Point::update_position(
                                            &mut self.points,
                                            selected_id,
                                            same_pos,
                                        );

                                        self.selected_point_index = None;
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
        Self {
            polygon_mode: PolygonMode::Editing,
            line_drawing_algorithm: LineDrawingAlgorithm::Bresenham,
            points,
            dragged_index: None,
            bezier_control_point_dragged: None,
            polygon_dragged_index: None,
            selected_edge_start_index: None,
            selected_point_index: None,
            popups: Popups::default(),
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
                    if ui.button("Draw new polygon").clicked() {
                        *self = Self::new_with_drawing_mode();
                    }
                });
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
            match self.polygon_mode {
                PolygonMode::Drawing => {
                    // Important: Order here matters!
                    match self.line_drawing_algorithm {
                        LineDrawingAlgorithm::Bultin => Drawer::draw_incomplete_polygon_builtin(
                            &self.points,
                            painter,
                            Color32::LIGHT_GREEN,
                            1.0,
                        ),
                        LineDrawingAlgorithm::Bresenham => {
                            Drawer::draw_incomplete_polygon_bresenham(
                                &self.points,
                                painter,
                                Color32::YELLOW,
                            )
                        }
                    };
                    Drawer::draw_points(
                        &self.points,
                        None,
                        painter,
                        Color32::DARK_BLUE,
                        Color32::DARK_GREEN,
                    );
                    // LMB on plane
                    self.handle_adding_point_in_drawing_mode(ctx, ui.min_rect().width());
                }
                PolygonMode::Editing => {
                    // Important: Order here matters!
                    match self.line_drawing_algorithm {
                        LineDrawingAlgorithm::Bultin => Drawer::draw_polygon_builtin(
                            &self.points,
                            self.selected_point_index,
                            self.selected_edge_start_index,
                            painter,
                            Color32::LIGHT_GREEN,
                            Color32::ORANGE,
                            1.0,
                        ),
                        LineDrawingAlgorithm::Bresenham => Drawer::draw_polygon_bresenham(
                            &self.points,
                            self.selected_point_index,
                            self.selected_edge_start_index,
                            painter,
                            Color32::YELLOW,
                            Color32::ORANGE,
                        ),
                    };
                    Drawer::draw_points(
                        &self.points,
                        self.selected_point_index,
                        painter,
                        Color32::DARK_BLUE,
                        Color32::DARK_GREEN,
                    );
                    // ctrl + LMB on point
                    self.handle_dragging_polygon(ctx);
                    // alt + LMB on point
                    self.handle_removing_point(ctx);
                    // LMB on point
                    self.handle_dragging_points(ctx);
                    // RMB on edge/point
                    self.handle_selecting_edge_or_point(ctx);
                    self.show_context_menu_for_selected_edge(ctx, ui);
                    self.show_context_menu_for_selected_point(ctx);
                }
            }
        });
    }
}
