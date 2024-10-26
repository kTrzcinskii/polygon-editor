use egui::{Pos2, Vec2};

use crate::bezier::BezierData;

#[derive(Clone, Copy)]
pub enum EdgeConstraint {
    Horizontal,
    Vertical,
    ConstWidth(i32),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ContinuityType {
    G0,
    C1,
    G1,
}

#[derive(Debug, Clone, Copy)]
enum UpdateDirection {
    Left,
    Right,
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
        points[point_index].pos = new_position;
        let direction = match points[point_index].is_start_of_bezier_segment() {
            true => UpdateDirection::Left,
            false => UpdateDirection::Right,
        };
        Self::adjust_adjacent_bezier_segments_control_points(
            points,
            point_index,
            0,
            UpdateDirection::Left,
        );
        Self::adjust_adjacent_bezier_segments_control_points(
            points,
            point_index,
            0,
            UpdateDirection::Right,
        );

        match direction {
            UpdateDirection::Left => {
                Self::adjust_adjacent_edges_after_position_update(points, point_index)
            }
            UpdateDirection::Right => {
                Self::adjust_adjacent_edges_after_position_update_right_first(points, point_index)
            }
        }
    }

    pub fn update_position_after_control_point_moved(
        points: &mut [Point],
        point_index: usize,
        inner_point_index: usize,
    ) {
        let direction = match inner_point_index {
            0 => UpdateDirection::Left,
            1 => UpdateDirection::Right,
            _ => panic!("Invalid inner_point_index"),
        };

        match inner_point_index {
            0 => {
                let previous_index = Self::get_previous_index(points, point_index);
                let c = *points[point_index].continuity_type();
                if c != ContinuityType::G0 {
                    if let Some(constraint) = points[previous_index].constraint() {
                        let inner_point =
                            points[point_index].bezier_data().unwrap().inner_points()[0];
                        match constraint {
                            EdgeConstraint::Horizontal => {
                                points[point_index].pos_mut().y = inner_point.y;
                            }
                            EdgeConstraint::Vertical => {
                                points[point_index].pos_mut().x = inner_point.x;
                            }
                            EdgeConstraint::ConstWidth(_) => {}
                        }
                    };
                }
            }
            1 => {
                let next_index = Self::get_next_index(points, point_index);
                let c = *points[next_index].continuity_type();
                if c != ContinuityType::G0 {
                    if let Some(constraint) = points[next_index].constraint() {
                        let inner_point =
                            points[point_index].bezier_data().unwrap().inner_points()[1];
                        match constraint {
                            EdgeConstraint::Horizontal => {
                                points[next_index].pos_mut().y = inner_point.y;
                            }
                            EdgeConstraint::Vertical => {
                                points[next_index].pos_mut().x = inner_point.x;
                            }
                            EdgeConstraint::ConstWidth(_) => {}
                        }
                    };
                }
            }
            _ => panic!("Invalid inner_point_index"),
        };

        Self::adjust_adjacent_bezier_segments_control_points(
            points,
            point_index,
            inner_point_index,
            direction,
        );
        match direction {
            UpdateDirection::Left => {
                Self::adjust_adjacent_edges_after_position_update(points, point_index)
            }
            UpdateDirection::Right => {
                Self::adjust_adjacent_edges_after_position_update_right_first(points, point_index)
            }
        };
    }

    fn adjust_adjacent_bezier_segments_control_points(
        points: &mut [Point],
        point_index: usize,
        moved_control_point_id: usize,
        update_direction: UpdateDirection,
    ) {
        #[cfg(feature = "show_debug_info")]
        println!("Adjusting bezier segments control point in {point_index}");

        match moved_control_point_id {
            0 => {
                if Self::is_end_of_bezier_segment(points, point_index) {
                    Self::adjust_bezier_segment_control_points_from_end(
                        points,
                        point_index,
                        update_direction,
                    );
                }

                if points[point_index].is_start_of_bezier_segment() {
                    Self::adjust_bezier_segment_control_points_from_start(
                        points,
                        point_index,
                        update_direction,
                    );
                }
            }
            1 => {
                if points[point_index].is_start_of_bezier_segment() {
                    Self::adjust_bezier_segment_control_points_from_start(
                        points,
                        point_index,
                        update_direction,
                    );
                }
                if Self::is_end_of_bezier_segment(points, point_index) {
                    Self::adjust_bezier_segment_control_points_from_end(
                        points,
                        point_index,
                        update_direction,
                    );
                }
            }
            _ => eprintln!("Moved control point id should never be greater than 1"),
        }
    }

    fn adjust_bezier_segment_control_points_from_start(
        points: &mut [Point],
        point_index: usize,
        update_direction: UpdateDirection,
    ) {
        match points[point_index].continuity_type() {
            ContinuityType::G0 => {}
            ContinuityType::C1 => {
                Self::adjust_c1_continuity(points, point_index, update_direction);
            }
            ContinuityType::G1 => {
                Self::adjust_g1_coninuity(points, point_index, update_direction);
            }
        }
    }

    fn adjust_bezier_segment_control_points_from_end(
        points: &mut [Point],
        point_index: usize,
        update_direction: UpdateDirection,
    ) {
        match points[point_index].continuity_type() {
            ContinuityType::G0 => {}
            ContinuityType::C1 => {
                Self::adjust_c1_continuity(points, point_index, update_direction);
            }
            ContinuityType::G1 => {
                Self::adjust_g1_coninuity(points, point_index, update_direction);
            }
        }
    }

    fn new_position_for_adjusting_g1_continuity(
        coninuity_point: Pos2,
        end_to_stay: Pos2,
        end_to_update: Pos2,
    ) -> Pos2 {
        let unchanged_vector = end_to_stay - coninuity_point;
        let vector_length = end_to_update.distance(coninuity_point);
        coninuity_point - unchanged_vector.normalized() * vector_length
    }

    fn adjust_g1_coninuity(
        points: &mut [Point],
        point_index: usize,
        update_direction: UpdateDirection,
    ) {
        #[cfg(feature = "show_debug_info")]
        println!(
            "Adjusting G1 coninuity in {} (direction: {:?})",
            point_index, update_direction
        );

        let previous_point = Self::get_previous_index(points, point_index);
        let next_point = Self::get_next_index(points, point_index);

        let continuity_point = *points[point_index].pos();

        match update_direction {
            UpdateDirection::Left => {
                let c = points[point_index]
                    .constraint()
                    .unwrap_or(EdgeConstraint::ConstWidth(0));

                match c {
                    EdgeConstraint::Horizontal => match points[point_index].bezier_data_mut() {
                        Some(bs) => bs.inner_points_mut()[0].y = continuity_point.y,
                        None => points[next_point].pos_mut().y = continuity_point.y,
                    },
                    EdgeConstraint::Vertical => match points[point_index].bezier_data_mut() {
                        Some(bs) => bs.inner_points_mut()[0].x = continuity_point.x,
                        None => points[next_point].pos_mut().x = continuity_point.x,
                    },
                    EdgeConstraint::ConstWidth(_) => {}
                }

                let end_to_stay = match points[point_index].bezier_data() {
                    Some(bs) => bs.inner_points()[0],
                    None => *points[next_point].pos(),
                };
                let end_to_update = match points[previous_point].bezier_data() {
                    Some(bs) => bs.inner_points()[1],
                    None => *points[previous_point].pos(),
                };

                let new_position = Self::new_position_for_adjusting_g1_continuity(
                    continuity_point,
                    end_to_stay,
                    end_to_update,
                );

                match points[previous_point].bezier_data_mut() {
                    Some(bs) => bs.update_inner_point_position(1, new_position),
                    None => *points[previous_point].pos_mut() = new_position,
                }
            }
            UpdateDirection::Right => {
                let c = points[point_index]
                    .constraint()
                    .unwrap_or(EdgeConstraint::ConstWidth(0));

                match c {
                    EdgeConstraint::Horizontal => match points[previous_point].bezier_data_mut() {
                        Some(bs) => bs.inner_points_mut()[1].y = continuity_point.y,
                        None => points[previous_point].pos_mut().y = continuity_point.y,
                    },
                    EdgeConstraint::Vertical => match points[previous_point].bezier_data_mut() {
                        Some(bs) => bs.inner_points_mut()[1].x = continuity_point.x,
                        None => points[previous_point].pos_mut().x = continuity_point.x,
                    },
                    EdgeConstraint::ConstWidth(_) => {}
                }

                let end_to_stay = match points[previous_point].bezier_data {
                    Some(bs) => bs.inner_points()[1],
                    None => *points[previous_point].pos(),
                };
                let end_to_update = match points[point_index].bezier_data() {
                    Some(bs) => bs.inner_points()[0],
                    None => *points[next_point].pos(),
                };

                let new_position = Self::new_position_for_adjusting_g1_continuity(
                    continuity_point,
                    end_to_stay,
                    end_to_update,
                );

                match points[point_index].bezier_data_mut() {
                    Some(bs) => bs.update_inner_point_position(0, new_position),
                    None => *points[next_point].pos_mut() = new_position,
                }
            }
        }
    }

    fn new_position_for_adjusting_c1_continuity(
        coninuity_point: Pos2,
        end_to_stay: Pos2,
        end_to_preserve_length: Pos2,
        scale: f32,
    ) -> Pos2 {
        let unchanged_vector = end_to_stay - coninuity_point;
        let vector_length = end_to_preserve_length.distance(coninuity_point) * scale;
        coninuity_point - unchanged_vector.normalized() * vector_length
    }

    fn adjust_c1_continuity(
        points: &mut [Point],
        point_index: usize,
        update_direction: UpdateDirection,
    ) {
        #[cfg(feature = "show_debug_info")]
        println!(
            "Adjusting C1 coninuity in {} (direction: {:?})",
            point_index, update_direction
        );

        let previous_point = Self::get_previous_index(points, point_index);
        let next_point = Self::get_next_index(points, point_index);

        let continuity_point = *points[point_index].pos();

        match update_direction {
            UpdateDirection::Left => {
                let c = points[point_index]
                    .constraint()
                    .unwrap_or(EdgeConstraint::ConstWidth(0));

                match c {
                    EdgeConstraint::Horizontal => match points[point_index].bezier_data_mut() {
                        Some(bs) => bs.inner_points_mut()[0].y = continuity_point.y,
                        None => points[next_point].pos_mut().y = continuity_point.y,
                    },
                    EdgeConstraint::Vertical => match points[point_index].bezier_data_mut() {
                        Some(bs) => bs.inner_points_mut()[0].x = continuity_point.x,
                        None => points[next_point].pos_mut().x = continuity_point.x,
                    },
                    EdgeConstraint::ConstWidth(_) => {}
                }

                let (end_to_stay, is_end_to_stay_bezier) = match points[point_index].bezier_data() {
                    Some(bs) => (bs.inner_points()[0], true),
                    None => (*points[next_point].pos(), false),
                };

                let is_end_to_update_bezier = points[previous_point].bezier_data().is_some();

                let scale = match (is_end_to_stay_bezier, is_end_to_update_bezier) {
                    (true, true) => 1.0,
                    (true, false) => 3.0,
                    (false, true) => 1.0 / 3.0,
                    (false, false) => panic!("One of them should be bezier"),
                };

                let new_position = Self::new_position_for_adjusting_c1_continuity(
                    continuity_point,
                    end_to_stay,
                    end_to_stay,
                    scale,
                );

                match points[previous_point].bezier_data_mut() {
                    Some(bs) => bs.update_inner_point_position(1, new_position),
                    None => *points[previous_point].pos_mut() = new_position,
                }
            }
            UpdateDirection::Right => {
                let c = points[point_index]
                    .constraint()
                    .unwrap_or(EdgeConstraint::ConstWidth(0));

                match c {
                    EdgeConstraint::Horizontal => match points[previous_point].bezier_data_mut() {
                        Some(bs) => bs.inner_points_mut()[1].y = continuity_point.y,
                        None => points[previous_point].pos_mut().y = continuity_point.y,
                    },
                    EdgeConstraint::Vertical => match points[previous_point].bezier_data_mut() {
                        Some(bs) => bs.inner_points_mut()[1].x = continuity_point.x,
                        None => points[previous_point].pos_mut().x = continuity_point.x,
                    },
                    EdgeConstraint::ConstWidth(w) => {
                        if w > 0 {
                            if let Some(bs) = points[previous_point].bezier_data_mut() {
                                let new_position = Self::calculate_position_for_keeping_width(
                                    w as f32 * 1.0 / 3.0,
                                    continuity_point,
                                    bs.inner_points()[1],
                                );
                                bs.update_inner_point_position(1, new_position);
                            }
                        }
                    }
                }

                let (end_to_stay, is_end_to_stay_bezier) = match points[previous_point].bezier_data
                {
                    Some(bs) => (bs.inner_points()[1], true),
                    None => (*points[previous_point].pos(), false),
                };

                let is_end_to_update_bezier = points[point_index].bezier_data().is_some();

                let scale = match (is_end_to_stay_bezier, is_end_to_update_bezier) {
                    (true, true) => 1.0,
                    (true, false) => 3.0,
                    (false, true) => 1.0 / 3.0,
                    (false, false) => panic!("One of them should be bezier"),
                };

                let new_position = Self::new_position_for_adjusting_c1_continuity(
                    continuity_point,
                    end_to_stay,
                    end_to_stay,
                    scale,
                );

                match points[point_index].bezier_data_mut() {
                    Some(bs) => bs.update_inner_point_position(0, new_position),
                    None => *points[next_point].pos_mut() = new_position,
                }
            }
        };
    }

    fn adjust_adjacent_edges_after_position_update(points: &mut [Point], point_index: usize) {
        #[cfg(feature = "show_debug_info")]
        {
            println!("================================================");
            println!(
                "Starting adjustment process (left first) from: {}",
                point_index
            );
        }

        Self::adjust_adjacent_edges_after_position_update_only_left(points, point_index);
        Self::adjust_adjacent_edges_after_position_update_only_right(points, point_index);

        #[cfg(feature = "show_debug_info")]
        println!("================================================");
    }

    fn adjust_adjacent_edges_after_position_update_right_first(
        points: &mut [Point],
        point_index: usize,
    ) {
        #[cfg(feature = "show_debug_info")]
        {
            println!("================================================");
            println!(
                "Starting adjustment process (right first) from: {}",
                point_index
            );
        }

        Self::adjust_adjacent_edges_after_position_update_only_right(points, point_index);
        Self::adjust_adjacent_edges_after_position_update_only_left(points, point_index);

        #[cfg(feature = "show_debug_info")]
        println!("================================================");
    }

    fn adjust_adjacent_edges_after_position_update_only_left(
        points: &mut [Point],
        point_index: usize,
    ) {
        let mut left = point_index;
        let mut left_stop = !points[Self::get_previous_index(points, left)].has_constraint()
            && !Self::is_part_of_bezier_segment(points, left)
            && !Self::is_part_of_bezier_segment(points, Self::get_previous_index(points, left));

        Self::adjust_adjacent_bezier_segments_control_points(
            points,
            point_index,
            0,
            UpdateDirection::Left,
        );

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
            let is_bezier_segment = Self::is_part_of_bezier_segment(points, left);
            if !constraint_and_next_width_constraint
                && !width_contraint_and_next_constraint
                && !is_bezier_segment
            {
                #[cfg(feature = "show_debug_info")]
                println!("Left stops at: {}", left);
                left_stop = true;
            }
            i_left += 1;
        }
    }

    fn adjust_adjacent_edges_after_position_update_only_right(
        points: &mut [Point],
        point_index: usize,
    ) {
        let mut right = point_index;
        let mut right_stop = !points[right].has_constraint()
            && !Self::is_part_of_bezier_segment(points, right)
            && !Self::is_part_of_bezier_segment(points, Point::get_next_index(points, right));

        Self::adjust_adjacent_bezier_segments_control_points(
            points,
            point_index,
            1,
            UpdateDirection::Right,
        );

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
            let is_bezier_segment = Self::is_part_of_bezier_segment(points, current_edge_index);

            if !constraint_and_next_width_constraint
                && !width_contraint_and_next_constraint
                && !is_bezier_segment
            {
                #[cfg(feature = "show_debug_info")]
                println!("Right stops at: {}", right);
                right_stop = true;
            }
            i_right += 1;
        }
    }

    fn adjust_moved_point_edge_start(points: &mut [Point], edge_start_index: usize) {
        #[cfg(feature = "show_debug_info")]
        println!("Adjusting point {edge_start_index} from start");

        let constraint = *points[edge_start_index].constraint();
        let edge_end_index = Self::get_next_index(points, edge_start_index);
        if let Some(constraint) = constraint {
            #[cfg(feature = "show_debug_info")]
            println!(
                "Applying constriaint (from start point): {}-{}",
                edge_start_index, edge_end_index
            );
            Self::apply_constraint_diff(points, edge_end_index, edge_start_index, &constraint);
        }
        Self::adjust_adjacent_bezier_segments_control_points(
            points,
            edge_end_index,
            0,
            UpdateDirection::Right,
        );
    }

    fn adjust_moved_point_edge_end(points: &mut [Point], edge_end_index: usize) {
        #[cfg(feature = "show_debug_info")]
        println!("Adjusting point {edge_end_index} from end");

        let edge_start_index = Self::get_previous_index(points, edge_end_index);
        let constraint = *points[edge_start_index].constraint();
        if let Some(constraint) = constraint {
            #[cfg(feature = "show_debug_info")]
            println!(
                "Applying constraint on edge (from end point): {}-{}",
                edge_start_index, edge_end_index
            );
            Self::apply_constraint_diff(points, edge_start_index, edge_end_index, &constraint);
        }
        Self::adjust_adjacent_bezier_segments_control_points(
            points,
            edge_start_index,
            0,
            UpdateDirection::Left,
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
                let new_position = Self::calculate_position_for_keeping_width(
                    *width as f32,
                    *points[other_edge_end_index].pos(),
                    *points[point_index].pos(),
                );
                *points[point_index].pos_mut() = new_position;
            }
        }
    }

    fn calculate_position_for_keeping_width(
        width: f32,
        pos_to_stay: Pos2,
        pos_to_update: Pos2,
    ) -> Pos2 {
        let angle_between =
            (pos_to_update.y - pos_to_stay.y).atan2(pos_to_update.x - pos_to_stay.x);
        Pos2 {
            x: pos_to_stay.x + width * angle_between.cos(),
            y: pos_to_stay.y + width * angle_between.sin(),
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
