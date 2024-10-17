use egui::Pos2;

#[derive(Clone, Copy)]
pub struct BezierData {
    inner_points: [Pos2; 2],
}

impl BezierData {
    pub fn new(inner_points: [Pos2; 2]) -> Self {
        Self { inner_points }
    }

    pub fn inner_points(&self) -> &[Pos2; 2] {
        &self.inner_points
    }

    pub fn update_inner_point_position(&mut self, index: usize, new_position: Pos2) {
        self.inner_points[index] = new_position;
    }
}
