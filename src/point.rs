use egui::{Pos2, Vec2};

use crate::bezier::BezierData;

#[derive(Clone, Copy)]
pub enum EdgeConstraint {
    Horizontal,
    Vertical,
    ConstWidth(i32),
}

#[derive(Clone, Copy)]
pub enum ContinuityType {
    G0,
    C1,
    G1,
}

// Each point is at the same time start of some edge
// Information about this edge are stored in this struct
#[derive(Clone, Copy)]
pub struct Point {
    pos: Pos2,
    /// Contraint that is applied to edge which starts in this point (and ends in the next one)
    constraint: Option<EdgeConstraint>,
    /// Data for bezier segment that starts in this point (and ends in the next one)
    bezier_data: Option<BezierData>,
    continuity_type: ContinuityType,
}

impl Point {
    pub fn new(pos: Pos2) -> Self {
        Self {
            pos,
            constraint: None,
            bezier_data: None,
            continuity_type: ContinuityType::G0,
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

    pub fn bezier_data(&self) -> &Option<BezierData> {
        &self.bezier_data
    }

    pub fn bezier_data_mut(&mut self) -> Option<&mut BezierData> {
        self.bezier_data.as_mut()
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

    pub fn is_start_of_bezier_segment(&self) -> bool {
        self.bezier_data.is_some()
    }

    pub fn is_end_of_bezier_segment(points: &[Point], point_index: usize) -> bool {
        points[Point::get_previous_index(points, point_index)].is_start_of_bezier_segment()
    }

    pub fn is_part_of_bezier_segment(points: &[Point], point_index: usize) -> bool {
        Self::is_end_of_bezier_segment(points, point_index)
            || points[point_index].is_start_of_bezier_segment()
    }

    pub fn init_bezier_data(&mut self, initial_pos: [Pos2; 2]) {
        self.bezier_data = Some(BezierData::new(initial_pos));
    }

    pub fn remove_bezier_data(&mut self) {
        self.bezier_data = None;
    }

    pub fn continuity_type(&self) -> &ContinuityType {
        &self.continuity_type
    }

    #[allow(non_snake_case)]
    pub fn apply_G0(&mut self) {
        self.continuity_type = ContinuityType::G0;
    }

    #[allow(non_snake_case)]
    pub fn apply_G1(&mut self) {
        self.continuity_type = ContinuityType::G1;
    }

    #[allow(non_snake_case)]
    pub fn apply_C1(&mut self) {
        self.continuity_type = ContinuityType::C1;
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
        let previous_position = *points[point_index].pos();
        points[point_index].pos = new_position;
        Self::adjust_adjacent_bezier_segments_control_points(
            points,
            point_index,
            previous_position,
        );
        Self::adjust_adjacent_edges_after_position_update(points, point_index);
    }

    fn adjust_adjacent_bezier_segments_control_points(
        points: &mut [Point],
        point_index: usize,
        previous_position: Pos2,
    ) {
        if points[point_index].is_start_of_bezier_segment() {
            Self::adjust_bezier_segment_control_points_from_start(
                points,
                point_index,
                previous_position,
            );
        }
        if Self::is_end_of_bezier_segment(points, point_index) {
            Self::adjust_bezier_segment_control_points_from_end(
                points,
                point_index,
                previous_position,
            );
        }
    }

    fn adjust_bezier_segment_control_points_from_start(
        points: &mut [Point],
        point_index: usize,
        previous_position: Pos2,
    ) {
        let bezier_data = points[point_index].bezier_data().expect(
            "This function should only be called for point which is the start of bezier segment",
        );
        let inner_points = *bezier_data.inner_points();
        let end_index = Self::get_next_index(points, point_index);
        let new_inner_points = Self::scale_and_rotate_bezier(
            &previous_position,
            points[point_index].pos(),
            &inner_points,
            points[end_index].pos(),
        );
        let bezier_data = points[point_index]
            .bezier_data_mut()
            .expect("Should never happen after first check");
        *bezier_data.inner_points_mut() = new_inner_points;
        match points[point_index].continuity_type() {
            ContinuityType::G0 => {}
            ContinuityType::C1 => todo!(),
            ContinuityType::G1 => Self::adjust_g1_coninuity_edge_end(points, point_index),
        }
    }

    fn adjust_bezier_segment_control_points_from_end(
        points: &mut [Point],
        point_index: usize,
        previous_position: Pos2,
    ) {
        let start_index = Self::get_previous_index(points, point_index);
        let bezier_data = points[start_index].bezier_data().expect(
            "This function should only be called for point which is the end of bezier segment",
        );
        let inner_points = *bezier_data.inner_points();
        let new_inner_points = Self::scale_and_rotate_bezier(
            &previous_position,
            &points[point_index].pos,
            &inner_points,
            points[start_index].pos(),
        );
        let bezier_data = points[start_index]
            .bezier_data_mut()
            .expect("Should never happen after first check");
        *bezier_data.inner_points_mut() = new_inner_points;
        match points[point_index].continuity_type() {
            ContinuityType::G0 => {}
            ContinuityType::C1 => todo!(),
            ContinuityType::G1 => Self::adjust_g1_coninuity_edge_start(points, point_index),
        }
    }

    fn scale_and_rotate_bezier(
        previous_position: &Pos2,
        current_position: &Pos2,
        inner_points: &[Pos2; 2],
        end_not_moved: &Pos2,
    ) -> [Pos2; 2] {
        let initial_distance = previous_position.distance(*end_not_moved);
        let new_distance = current_position.distance(*end_not_moved);
        let scale = new_distance / initial_distance;
        let new_angle = (*current_position - *end_not_moved).angle();
        let previos_angle = (*previous_position - *end_not_moved).angle();
        let angle_diff = new_angle - previos_angle;
        let c0_new_angle = (inner_points[0] - *end_not_moved).angle() + angle_diff;
        let new_c0 =
            (Vec2::angled(c0_new_angle) * (inner_points[0] - *end_not_moved).length() * scale)
                .to_pos2()
                + end_not_moved.to_vec2();
        let c1_new_angle = (inner_points[1] - *end_not_moved).angle() + angle_diff;
        let new_c1 =
            (Vec2::angled(c1_new_angle) * (inner_points[1] - *end_not_moved).length() * scale)
                .to_pos2()
                + end_not_moved.to_vec2();
        [new_c0, new_c1]
    }

    /// We adjust edge so that point with index `edge_end_index` keeps G1 continuity
    /// We assume that `edge_end_index` is start of bezier segment
    fn adjust_g1_coninuity_edge_end(points: &mut [Point], edge_end_index: usize) {
        // We have
        // edge <-> bezier segment
        let bs_data = points[edge_end_index]
            .bezier_data()
            .expect("It should be called only for start of bezier segment");
        let bs_vector = bs_data.inner_points()[0] - *points[edge_end_index].pos();

        let edge_start_index = Self::get_previous_index(points, edge_end_index);
        let is_bezier_segment = points[edge_start_index].is_start_of_bezier_segment();
        let e_vector_length = match is_bezier_segment {
            true => {
                let bs = points[edge_start_index].bezier_data().unwrap();
                bs.inner_points()[1].distance(*points[edge_end_index].pos())
            }
            false => points[edge_end_index]
                .pos()
                .distance(*points[edge_start_index].pos()),
        };
        let new_point = *points[edge_end_index].pos() - bs_vector.normalized() * e_vector_length;
        match is_bezier_segment {
            true => {
                let bs = points[edge_start_index].bezier_data_mut().unwrap();
                bs.update_inner_point_position(1, new_point);
            }
            false => *points[edge_start_index].pos_mut() = new_point,
        }
    }

    /// We adjust edge so that point with index `edge_start_index` keeps G1 continuity
    /// We assume that `edge_start_index` is start of bezier segment
    fn adjust_g1_coninuity_edge_start(points: &mut [Point], edge_start_index: usize) {
        // We have
        // bezier segment <-> edge
        let bs_start_index = Self::get_previous_index(points, edge_start_index);
        let bs_data = points[bs_start_index]
            .bezier_data()
            .expect("It should be called only for start of bezier segment");
        let bs_vector = bs_data.inner_points()[1] - *points[edge_start_index].pos();

        let edge_end_index = Self::get_next_index(points, edge_start_index);
        let is_bezier_segment = points[edge_start_index].is_start_of_bezier_segment();
        let e_vector_length = match is_bezier_segment {
            true => {
                let bs = points[edge_start_index].bezier_data().unwrap();
                bs.inner_points()[0].distance(*points[edge_start_index].pos())
            }
            false => points[edge_end_index]
                .pos()
                .distance(*points[edge_start_index].pos()),
        };
        let new_point = *points[edge_start_index].pos() - bs_vector.normalized() * e_vector_length;
        match is_bezier_segment {
            true => {
                let bs = points[edge_start_index].bezier_data_mut().unwrap();
                bs.update_inner_point_position(0, new_point);
            }
            false => *points[edge_end_index].pos_mut() = new_point,
        }
    }

    // TODO:
    // - should properly handle C1 and G1
    pub fn adjust_adjacent_edges_after_position_update(points: &mut [Point], point_index: usize) {
        #[cfg(feature = "show_debug_info")]
        {
            println!("================================================");
            println!("Starting adjustment process from: {}", point_index);
        }

        let mut left = point_index;
        let mut left_stop = !points[Self::get_previous_index(points, left)].has_constraint()
            && !Self::is_part_of_bezier_segment(points, left);

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
            let is_bezier_segment_or_next_bezier_segment =
                Self::is_part_of_bezier_segment(points, left)
                    || Self::is_part_of_bezier_segment(points, next_edge_start);
            if !constraint_and_next_width_constraint
                && !width_contraint_and_next_constraint
                && !is_bezier_segment_or_next_bezier_segment
            {
                #[cfg(feature = "show_debug_info")]
                println!("Left stops at: {}", left);
                left_stop = true;
            }
            i_left += 1;
        }

        let mut right = point_index;
        let mut right_stop =
            !points[right].has_constraint() && !Self::is_part_of_bezier_segment(points, right);

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
            let is_bezier_segment_or_next_bezier_segment =
                Self::is_part_of_bezier_segment(points, current_edge_index)
                    || Self::is_part_of_bezier_segment(points, right);

            if !constraint_and_next_width_constraint
                && !width_contraint_and_next_constraint
                && !is_bezier_segment_or_next_bezier_segment
            {
                #[cfg(feature = "show_debug_info")]
                println!("Right stops at: {}", right);
                right_stop = true;
            }
            i_right += 1;
        }
    }

    fn adjust_moved_point_edge_start(points: &mut [Point], edge_start_index: usize) {
        let constraint = *points[edge_start_index].constraint();
        let edge_end_index = Self::get_next_index(points, edge_start_index);
        let previous_pos = *points[edge_end_index].pos();
        if let Some(constraint) = constraint {
            #[cfg(feature = "show_debug_info")]
            println!(
                "Adjust edge (from start point): {}-{}",
                edge_start_index, edge_end_index
            );
            Self::apply_constraint_diff(points, edge_end_index, edge_start_index, &constraint);
        }
        Self::adjust_adjacent_bezier_segments_control_points(points, edge_end_index, previous_pos);
    }

    fn adjust_moved_point_edge_end(points: &mut [Point], edge_end_index: usize) {
        let edge_start_index = Self::get_previous_index(points, edge_end_index);
        let constraint = *points[edge_start_index].constraint();
        let previous_pos = *points[edge_start_index].pos();
        if let Some(constraint) = constraint {
            #[cfg(feature = "show_debug_info")]
            println!(
                "Adjust edge (from end point): {}-{}",
                edge_start_index, edge_end_index
            );
            Self::apply_constraint_diff(points, edge_start_index, edge_end_index, &constraint);
        }
        Self::adjust_adjacent_bezier_segments_control_points(
            points,
            edge_start_index,
            previous_pos,
        );
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
        // Same goes for bezier segment
        points[edge_start_index].remove_constraint();
        points[edge_start_index].remove_bezier_data();
        let next_index = Self::get_next_index(points, edge_start_index);

        // Adding new edge is just inserting a point at correct index
        let new_point = Self::get_middle_point(&points[edge_start_index], &points[next_index]);
        points.insert(next_index, Point::new(new_point));
    }

    pub fn remove_at(points: &mut Vec<Point>, point_index: usize) {
        // If the point behind it has any restriction, we remove it
        // Restrisction on the removed point is removed with it, so we dont care about it
        // Same goes for bezier data
        let previous_index = Self::get_previous_index(points, point_index);
        points[previous_index].remove_constraint();
        points[previous_index].remove_bezier_data();
        points.remove(point_index);
    }

    pub fn update_position_all(points: &mut [Point], diff: Vec2) {
        for point in points {
            *point.pos_mut() += diff;
            if let Some(bd) = point.bezier_data_mut() {
                bd.inner_points_mut()[0] += diff;
                bd.inner_points_mut()[1] += diff;
            }
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

    pub fn get_points_between_for_initial_bezier(start: &Point, end: &Point) -> [Pos2; 2] {
        const OFFSET: f32 = 20.0;
        let diff = *end.pos() - *start.pos();
        let p = Vec2::new(-diff.y, diff.x).normalized() * OFFSET;
        let a = *start.pos() + diff * 1.0 / 3.0 + p;
        let b = *start.pos() + diff * 2.0 / 3.0 + p;
        [a, b]
    }
}
