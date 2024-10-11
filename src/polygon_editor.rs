use egui::{Color32, Pos2, Rounding, Vec2};

use crate::edge::{self, Edge};

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
    pub fn draw_points(&self, painter: &egui::Painter, color: Color32, width: f32) {
        for point in &self.points {
            painter.circle(*point, width, color, egui::Stroke { color, width });
        }
    }

    pub fn draw_polygon_builtin(
        &self,
        painter: &egui::Painter,
        color: Color32,
        special_color: Color32,
        width: f32,
    ) {
        for (id, edge) in self.edges.iter().enumerate() {
            let current_color = if id == self.selected_edge.unwrap_or(usize::MAX) {
                special_color
            } else {
                color
            };
            painter.line_segment(
                [self.points[edge.start_index], self.points[edge.end_index]],
                egui::Stroke {
                    color: current_color,
                    width,
                },
            );
        }
    }

    fn paint_pixel(&self, painter: &egui::Painter, position: Pos2, width: f32, color: Color32) {
        let rect = egui::Rect::from_min_size(position, egui::Vec2::new(width, width));
        painter.rect_filled(rect, 0.0, color);
    }

    // TODO: i think its not working when x or y of start and end are equal
    pub fn edge_contains_point(&self, edge: &Edge, point: &Pos2) -> bool {
        const TOLERANCE: f32 = 15.0;
        let start = self.points[edge.start_index];
        let end = self.points[edge.end_index];

        let dx_p = point.x - start.x;
        let dy_p = point.y - start.y;

        let dx_e = end.x - start.x;
        let dy_e = end.y - start.y;

        let squared_edge_length = dx_e * dx_e + dy_e * dy_e;

        let cross = dx_p * dy_e - dy_p * dx_e;

        if cross * cross / squared_edge_length > TOLERANCE {
            return false;
        }

        let min_x = start.x.min(end.x);
        let max_x = start.x.max(end.x);
        let min_y = start.y.min(end.y);
        let max_y = start.y.max(end.y);

        point.x >= min_x && point.x <= max_x && point.y >= min_y && point.y <= max_y
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

    pub fn draw_polygon_bresenham(
        &self,
        painter: &egui::Painter,
        color: Color32,
        special_color: Color32,
    ) {
        const WIDTH: f32 = 1.0;
        for (id, edge) in self.edges.iter().enumerate() {
            let current_color = if id == self.selected_edge.unwrap_or(usize::MAX) {
                special_color
            } else {
                color
            };
            self.draw_line_bresenham(
                painter,
                current_color,
                self.points[edge.start_index],
                self.points[edge.end_index],
                WIDTH,
            );
        }
    }

    fn move_point(&mut self, point_index: usize, new_position: Pos2) {
        let delta = new_position - self.points[point_index];
        self.points[point_index] = new_position;
        let edges = self.find_point_adjacent_edges(point_index);
        match edges {
            (None, None) => eprintln!("Trying to remove point that belongs to only one edge"),
            (None, Some(_)) => eprintln!("Trying to remove point that belongs to only one edge"),
            (Some(_), None) => eprintln!("Trying to remove point that belongs to only one edge"),
            (Some(first), Some(second)) => {
                self.adjust_moved_point_edge(point_index, first, &delta);
                self.adjust_moved_point_edge(point_index, second, &delta);
            }
        }
    }

    fn adjust_moved_point_edge(&mut self, point_index: usize, edge_index: usize, delta: &Vec2) {
        if let Some(restriction) = self.edges[edge_index].restriciton() {
            match restriction {
                edge::EdgeRestriction::Horizontal => {
                    let other_point = self.edges[edge_index].take_other_point(point_index);
                    self.points[other_point].y += delta.y;
                }
                edge::EdgeRestriction::Vertical => {
                    let other_point = self.edges[edge_index].take_other_point(point_index);
                    self.points[other_point].x += delta.x;
                }
                edge::EdgeRestriction::Width(_) => todo!(),
            }
        }
    }

    pub fn handle_dragging_points(&mut self, ctx: &egui::Context) {
        let mouse_pos = ctx.pointer_interact_pos();
        if let Some(pos) = mouse_pos {
            // Check user is holding LMB
            if ctx.input(|i| i.pointer.button_down(egui::PointerButton::Primary)) {
                // If already dragging then move point
                if let Some(index) = self.dragged_index {
                    self.move_point(index, pos);
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

    pub fn handle_selecting_edge(&mut self, ctx: &egui::Context) {
        let mouse_pos = ctx.pointer_hover_pos();
        if let Some(pos) = mouse_pos {
            if ctx.input(|i| i.pointer.button_down(egui::PointerButton::Secondary)) {
                let mut selected_now = false;
                for (id, edge) in self.edges.iter().enumerate() {
                    if self.edge_contains_point(edge, &pos) {
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
                    // Start dragging the point if it's close enough
                    if (*point - pos).length() < 10.0 {
                        id = Some(i)
                    }
                }
                if let Some(id) = id {
                    self.remove_point(id);
                }
            }
        }
    }

    pub fn show_context_menu_for_selected_edge(&mut self, ctx: &egui::Context, ui: &egui::Ui) {
        const CONTEXT_MENU_MIN_WDITH: f32 = 120.0;
        if let Some(selected_id) = self.selected_edge {
            let edge = &self.edges[selected_id];

            let container_pos = self.get_middle_point(edge.start_index, edge.end_index)
                - Vec2::new(
                    CONTEXT_MENU_MIN_WDITH / 2.0,
                    ui.spacing().interact_size.y * 3.0 / 2.0,
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
                                        self.add_point_on_edge(selected_id);
                                        self.selected_edge = None;
                                    }
                                    // TODO: make it disables when adjacent edge already has any restriciton
                                    if ui
                                        .add(
                                            egui::Button::new("Make horizontal")
                                                .rounding(Rounding::ZERO),
                                        )
                                        .clicked()
                                    {
                                        self.edges[selected_id].apply_horizontal_restriction();
                                        self.points[self.edges[selected_id].start_index].y =
                                            self.points[self.edges[selected_id].end_index].y;
                                        self.selected_edge = None;
                                    }
                                    // TODO: make it disables when adjacent edge already has any restriciton

                                    if ui
                                        .add(egui::Button::new("Make vertical").rounding(
                                            Rounding {
                                                nw: 0.0,
                                                ne: 0.0,
                                                ..Default::default()
                                            },
                                        ))
                                        .clicked()
                                    {
                                        self.edges[selected_id].apply_vertical_restriction();
                                        self.points[self.edges[selected_id].start_index].x =
                                            self.points[self.edges[selected_id].end_index].x;
                                        self.selected_edge = None;
                                    }
                                },
                            );
                        });
                });
        }
    }

    fn get_middle_point(&self, start: usize, end: usize) -> Pos2 {
        (self.points[start] + self.points[end].to_vec2()) / 2.0
    }

    fn add_point_on_edge(&mut self, edge_index: usize) {
        let edge = self.edges.remove(edge_index);
        let id_smaller = edge.start_index.min(edge.end_index);
        let id_bigger = edge.start_index.max(edge.end_index);
        let new_point = self.get_middle_point(id_smaller, id_bigger);
        self.points.push(new_point);
        let first_edge = Edge::new(id_smaller, self.points.len() - 1);
        let second_edge = Edge::new(self.points.len() - 1, id_bigger);
        self.edges.push(first_edge);
        self.edges.push(second_edge);
    }

    fn remove_point(&mut self, point_index: usize) {
        self.points.remove(point_index);
        let adjacent_edges = self.find_point_adjacent_edges(point_index);
        for edge in self.edges.iter_mut() {
            if edge.start_index > point_index {
                edge.start_index -= 1;
            }
            if edge.end_index > point_index {
                edge.end_index -= 1;
            }
        }
        match adjacent_edges {
            // Those should never happen (?)
            (None, None) => eprintln!("Trying to remove point that belongs to only one edge"),
            (None, Some(_)) => eprintln!("Trying to remove point that belongs to only one edge"),
            (Some(_), None) => eprintln!("Trying to remove point that belongs to only one edge"),
            (Some(first_id), Some(second_id)) => {
                let first = self.edges.remove(first_id);
                // -1 becasue we already removed one item
                let second = self.edges.remove(second_id - 1);
                let from_first = if first.start_index == point_index {
                    first.end_index
                } else {
                    first.start_index
                };
                let from_second = if second.start_index == point_index {
                    second.end_index
                } else {
                    second.start_index
                };
                let new_edge = Edge::new(from_first, from_second);
                self.edges.push(new_edge);
            }
        }
    }

    fn find_point_adjacent_edges(&self, point_index: usize) -> (Option<usize>, Option<usize>) {
        let mut first: Option<usize> = None;
        let mut second: Option<usize> = None;
        for (id, edge) in self.edges.iter().enumerate() {
            if edge.start_index == point_index || edge.end_index == point_index {
                if first.is_none() {
                    first = Some(id);
                } else {
                    second = Some(id);
                }
            }
        }
        (first, second)
    }
}

impl Default for PolygonEditor {
    fn default() -> Self {
        let points = vec![
            Pos2::new(50.0, 50.0),
            Pos2::new(100.0, 50.0),
            Pos2::new(75.0, 100.0),
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
                LineDrawingAlgorithm::Bultin => {
                    self.draw_polygon_builtin(painter, Color32::LIGHT_GREEN, Color32::ORANGE, 1.0)
                }
                LineDrawingAlgorithm::Bresenham => {
                    self.draw_polygon_bresenham(painter, Color32::YELLOW, Color32::ORANGE)
                }
            };
            self.draw_points(painter, Color32::DARK_BLUE, 4.0);
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
