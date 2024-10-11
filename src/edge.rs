use egui::Pos2;

use crate::point::Point;

pub enum EdgeRestriction {
    Horizontal,
    Vertical,
    Width(i32),
}

pub enum EdgePointType {
    Start,
    End,
}

pub struct Edge {
    /// Index of point that is start of the edge
    pub start_index: usize,
    /// Index of point that is end of the edge
    pub end_index: usize,
    /// Restriction currently applied to the edge
    restriction: Option<EdgeRestriction>,
}

impl Edge {
    pub fn new(start_index: usize, end_index: usize) -> Self {
        Edge {
            start_index,
            end_index,
            restriction: None,
        }
    }

    pub fn from_points(points: &[Point]) -> Vec<Self> {
        let mut edges = vec![];
        for i in 0..points.len() {
            let start_index = i;
            let end_index = (i + 1) % points.len();
            edges.push(Self::new(start_index, end_index));
        }
        edges
    }

    /// Returns the opposite vertex of an edge (for edge a-b and param a returns b)
    pub fn take_other_point(&self, point_index: usize) -> usize {
        if self.start_index == point_index {
            self.end_index
        } else {
            self.start_index
        }
    }

    pub fn has_restriction(&self) -> bool {
        self.restriction.is_some()
    }

    pub fn has_horizontal_or_vertical_restriction(&self) -> bool {
        match self.restriction() {
            Some(res) => matches!(res, EdgeRestriction::Horizontal | EdgeRestriction::Vertical),
            None => false,
        }
    }

    pub fn restriction(&self) -> &Option<EdgeRestriction> {
        &self.restriction
    }

    pub fn remove_restriction(&mut self) {
        self.restriction = None;
    }

    pub fn apply_horizontal_restriction(&mut self) {
        self.restriction = Some(EdgeRestriction::Horizontal);
    }

    pub fn apply_vertical_restriction(&mut self) {
        self.restriction = Some(EdgeRestriction::Vertical);
    }

    pub fn apply_width_restriction(&mut self, width: i32) {
        self.restriction = Some(EdgeRestriction::Width(width));
    }

    pub fn contains_point(&self, points: &[Point], point: &Pos2) -> bool {
        const TOLERANCE: f32 = 20.0;
        const TOLERANCE_SAME_DIM: f32 = 5.0;
        let start = points[self.start_index].pos();
        let end = points[self.end_index].pos();

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

    pub fn find_edge_adjacent_edges(
        edge_index: usize,
        edges: &[Edge],
    ) -> (Option<usize>, Option<usize>) {
        (
            Self::find_edge_adjacent_edge_with_common_point(
                edge_index,
                edges,
                EdgePointType::Start,
            ),
            Self::find_edge_adjacent_edge_with_common_point(edge_index, edges, EdgePointType::End),
        )
    }

    pub fn find_edge_adjacent_edge_with_common_point(
        edge_index: usize,
        edges: &[Edge],
        point: EdgePointType,
    ) -> Option<usize> {
        let point_index = match point {
            EdgePointType::Start => edges[edge_index].start_index,
            EdgePointType::End => edges[edge_index].end_index,
        };
        let adjacents = Point::find_adjacent_edges(point_index, edges);

        match adjacents {
            (None, None) => {
                eprintln!("Trying to find adjacent edges with point that belongs to no edge");
                None
            }
            (None, Some(_)) => {
                eprintln!("Trying to find adjacent edges with point that belongs to only one edge");
                None
            }
            (Some(_), None) => {
                eprintln!("Trying to find adjacent edges with point that belongs to only one edge");
                None
            }
            (Some(f), Some(s)) => {
                if f == edge_index {
                    Some(s)
                } else {
                    Some(f)
                }
            }
        }
    }

    pub fn neighours_have_vertical_or_horizontal_restriction(
        edges: &[Edge],
        edge_index: usize,
    ) -> bool {
        let adjacent_edges = Self::find_edge_adjacent_edges(edge_index, edges);

        let first_with_restrictions = if let Some(id) = adjacent_edges.0 {
            edges[id].has_horizontal_or_vertical_restriction()
        } else {
            false
        };
        let second_with_restrictions = if let Some(id) = adjacent_edges.1 {
            edges[id].has_horizontal_or_vertical_restriction()
        } else {
            false
        };

        first_with_restrictions || second_with_restrictions
    }
}
