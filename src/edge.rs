use egui::Pos2;

pub enum EdgeRestriction {
    Horizontal,
    Vertical,
    Width(i32),
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

    pub fn from_points(points: &[Pos2]) -> Vec<Self> {
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

    pub fn restriciton(&self) -> &Option<EdgeRestriction> {
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
}
