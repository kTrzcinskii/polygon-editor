use egui::Pos2;

pub struct Edge {
    /// Index of point that is start of the edge
    pub start_index: usize,
    /// Index of point that is end of the edge
    pub end_index: usize,
}

impl Edge {
    pub fn new(start_index: usize, end_index: usize) -> Self {
        Edge {
            start_index,
            end_index,
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
}
