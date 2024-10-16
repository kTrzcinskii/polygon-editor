use egui::{Pos2, Vec2};

#[derive(Clone, Copy)]
pub enum EdgeConstraint {
    Horizontal,
    Vertical,
    ConstWidth(i32),
}

// Each point is at the same time start of some edge
// Information about this edge are stored in this struct
#[derive(Clone, Copy)]
pub struct Point {
    pos: Pos2,
    constraint: Option<EdgeConstraint>,
}

impl Point {
    pub fn new(pos: Pos2) -> Self {
        Self {
            pos,
            constraint: None,
        }
    }

    pub fn pos(&self) -> &Pos2 {
        &self.pos
    }

    pub fn pos_mut(&mut self) -> &mut Pos2 {
        &mut self.pos
    }

    pub fn constraint(&self) -> &Option<EdgeConstraint> {
        &self.constraint
    }

    pub fn has_constraint(&self) -> bool {
        self.constraint.is_some()
    }

    pub fn has_horizontal_constraint(&self) -> bool {
        match self.constraint() {
            Some(res) => matches!(res, EdgeConstraint::Horizontal),
            None => false,
        }
    }

    pub fn has_vertical_constraint(&self) -> bool {
        match self.constraint() {
            Some(res) => matches!(res, EdgeConstraint::Vertical),
            None => false,
        }
    }

    pub fn has_width_constraint(&self) -> bool {
        match self.constraint() {
            Some(res) => matches!(res, EdgeConstraint::ConstWidth(_)),
            None => false,
        }
    }

    pub fn remove_constraint(&mut self) {
        self.constraint = None;
    }

    pub fn apply_horizontal_constraint(&mut self) {
        self.constraint = Some(EdgeConstraint::Horizontal);
    }

    pub fn apply_vertical_constraint(&mut self) {
        self.constraint = Some(EdgeConstraint::Vertical);
    }

    pub fn apply_width_constraint(&mut self, width: i32) {
        self.constraint = Some(EdgeConstraint::ConstWidth(width));
    }

    pub fn update_position(points: &mut [Point], point_index: usize, new_position: Pos2) {
        points[point_index].pos = new_position;
        Self::adjust_adjacent_edges_after_position_update(points, point_index);
    }

    pub fn adjust_adjacent_edges_after_position_update(points: &mut [Point], point_index: usize) {
        #[cfg(feature = "show_debug_info")]
        {
            println!("================================================");
            println!("Starting adjustment process from: {}", point_index);
        }

        let mut left = point_index;
        let mut left_stop = !points[Self::get_previous_index(points, left)].has_constraint();

        #[allow(unused_variables)]
        let mut i_left = 0;
        while !left_stop {
            #[cfg(feature = "show_debug_info")]
            println!("Step: {}, Left: {}", i_left, left);
            if Self::get_previous_index(points, left) == point_index {
                break;
            }
            Self::adjust_moved_point_edge_end(points, left);
            left = Self::get_previous_index(points, left);
            let next_edge_start = Self::get_previous_index(points, left);
            let constraint_and_next_width_constraint =
                points[left].has_constraint() && points[next_edge_start].has_width_constraint();
            let width_contraint_and_next_constraint =
                points[left].has_width_constraint() && points[next_edge_start].has_constraint();
            if !constraint_and_next_width_constraint && !width_contraint_and_next_constraint {
                #[cfg(feature = "show_debug_info")]
                println!("Left stops at: {}", left);
                left_stop = true;
            }
            i_left += 1;
        }

        let mut right = point_index;
        let mut right_stop = !points[right].has_constraint();

        #[allow(unused_variables)]
        let mut i_right = 0;
        while !right_stop {
            #[cfg(feature = "show_debug_info")]
            println!("Step: {}, Right: {}", i_right, right);

            if Self::get_next_index(points, right) == point_index {
                break;
            }

            Self::adjust_moved_point_edge_start(points, right);
            let current_edge_index = right;
            right = Self::get_next_index(points, right);
            let constraint_and_next_width_constraint =
                points[current_edge_index].has_constraint() && points[right].has_width_constraint();
            let width_contraint_and_next_constraint =
                points[current_edge_index].has_width_constraint() && points[right].has_constraint();

            if !constraint_and_next_width_constraint && !width_contraint_and_next_constraint {
                #[cfg(feature = "show_debug_info")]
                println!("Right stops at: {}", right);
                right_stop = true;
            }
            i_right += 1;
        }
    }

    fn adjust_moved_point_edge_start(points: &mut [Point], edge_start_index: usize) {
        let constraint = *points[edge_start_index].constraint();
        if let Some(constraint) = constraint {
            let edge_end_index = Self::get_next_index(points, edge_start_index);
            #[cfg(feature = "show_debug_info")]
            println!(
                "Adjust edge (from start point): {}-{}",
                edge_start_index, edge_end_index
            );
            Self::apply_constraint_diff(points, edge_end_index, edge_start_index, &constraint);
        }
    }

    fn adjust_moved_point_edge_end(points: &mut [Point], edge_end_index: usize) {
        let edge_start_index = Self::get_previous_index(points, edge_end_index);
        let constraint = *points[edge_start_index].constraint();
        if let Some(constraint) = constraint {
            #[cfg(feature = "show_debug_info")]
            println!(
                "Adjust edge (from end point): {}-{}",
                edge_start_index, edge_end_index
            );
            Self::apply_constraint_diff(points, edge_start_index, edge_end_index, &constraint);
        }
    }

    fn apply_constraint_diff(
        points: &mut [Point],
        point_index: usize,
        other_edge_end_index: usize,
        constraint: &EdgeConstraint,
    ) {
        match constraint {
            EdgeConstraint::Horizontal => {
                points[point_index].pos_mut().y = points[other_edge_end_index].pos().y;
            }
            EdgeConstraint::Vertical => {
                points[point_index].pos_mut().x = points[other_edge_end_index].pos().x;
            }
            EdgeConstraint::ConstWidth(width) => {
                let angle_between = (points[point_index].pos().y
                    - points[other_edge_end_index].pos().y)
                    .atan2(points[point_index].pos().x - points[other_edge_end_index].pos().x);
                let new_position = Pos2 {
                    x: points[other_edge_end_index].pos().x + *width as f32 * angle_between.cos(),
                    y: points[other_edge_end_index].pos().y + *width as f32 * angle_between.sin(),
                };
                *points[point_index].pos_mut() = new_position;
            }
        }
    }

    pub fn get_middle_point(start: &Point, end: &Point) -> Pos2 {
        (start.pos + end.pos().to_vec2()) / 2.0
    }

    pub fn get_next_index(points: &[Point], point_index: usize) -> usize {
        (point_index + 1) % points.len()
    }

    pub fn get_previous_index(points: &[Point], point_index: usize) -> usize {
        if point_index == 0 {
            points.len() - 1
        } else {
            point_index - 1
        }
    }

    pub fn add_on_edge(points: &mut Vec<Point>, edge_start_index: usize) {
        // If the edge we are dividing had any constraint we remove it,
        // as there should be two new edges each without any constraint
        points[edge_start_index].constraint = None;
        let next_index = Self::get_next_index(points, edge_start_index);

        // Adding new edge is just inserting a point at correct index
        let new_point = Self::get_middle_point(&points[edge_start_index], &points[next_index]);
        points.insert(next_index, Point::new(new_point));
    }

    pub fn remove_at(points: &mut Vec<Point>, point_index: usize) {
        // If the point behind it has any restriction, we remove it
        // Restrisction on the removed point is removed with it, so we dont care about it
        let previous_index = Self::get_previous_index(points, point_index);
        points[previous_index].constraint = None;
        points.remove(point_index);
    }

    pub fn update_position_all(points: &mut [Point], diff: Vec2) {
        for point in points {
            *point.pos_mut() += diff;
        }
    }

    /// Returns true if the edge that starts in edge_start_index
    /// contains the given point
    pub fn contains_point(points: &[Point], edge_start_index: usize, point: &Pos2) -> bool {
        const TOLERANCE: f32 = 20.0;
        const TOLERANCE_SAME_DIM: f32 = 5.0;
        let start = points[edge_start_index].pos();
        let end = points[Self::get_next_index(points, edge_start_index)].pos();

        let min_x = start.x.min(end.x);
        let max_x = start.x.max(end.x);
        let min_y = start.y.min(end.y);
        let max_y = start.y.max(end.y);

        if start.x == end.x
            && (point.x - start.x).abs() <= TOLERANCE_SAME_DIM
            && point.y >= min_y
            && point.y <= max_y
        {
            return true;
        }

        if start.y == end.y
            && (point.y - start.y).abs() <= TOLERANCE_SAME_DIM
            && point.x >= min_x
            && point.x <= max_x
        {
            return true;
        }

        let dx_p = point.x - start.x;
        let dy_p = point.y - start.y;

        let dx_e = end.x - start.x;
        let dy_e = end.y - start.y;

        let squared_edge_length = dx_e * dx_e + dy_e * dy_e;

        let cross = dx_p * dy_e - dy_p * dx_e;

        if cross * cross / squared_edge_length > TOLERANCE {
            return false;
        }

        point.x >= min_x && point.x <= max_x && point.y >= min_y && point.y <= max_y
    }

    pub fn neighour_edges_have_vertical_constraint(points: &[Point], point_index: usize) -> bool {
        let previous_edge_start = Self::get_previous_index(points, point_index);
        let next_edge_start = Self::get_next_index(points, point_index);
        points[previous_edge_start].has_vertical_constraint()
            || points[next_edge_start].has_vertical_constraint()
    }

    pub fn neighour_edges_have_horizontal_constraint(points: &[Point], point_index: usize) -> bool {
        let previous_edge_start = Self::get_previous_index(points, point_index);
        let next_edge_start = Self::get_next_index(points, point_index);
        points[previous_edge_start].has_horizontal_constraint()
            || points[next_edge_start].has_horizontal_constraint()
    }
}
