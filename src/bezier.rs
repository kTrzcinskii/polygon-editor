use egui::Pos2;

use crate::point::Point;

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

    pub fn inner_points_mut(&mut self) -> &mut [Pos2; 2] {
        &mut self.inner_points
    }

    pub fn update_inner_point_position(&mut self, index: usize, new_position: Pos2) {
        self.inner_points[index] = new_position;
    }

    /// Returns point on bezier curve. For this usecase it should be enough to just draw straight lines between these points.
    pub fn get_bezier_curve_points(&self, start: &Point, end: &Point) -> Vec<Pos2> {
        let polynomial_base = self.bezier_point_in_polynomial_base(start, end);
        let points_count = start.pos().distance(*end.pos()) * 2.0;
        let step_diff = 1.0 / points_count;
        let mut points = Vec::with_capacity(points_count as usize);
        points.push(*start.pos());
        for i in 1..(points_count as usize - 1) {
            let t = i as f32 * step_diff;
            let p = ((polynomial_base[3] * t + polynomial_base[2].to_vec2()) * t
                + polynomial_base[1].to_vec2())
                * t
                + polynomial_base[0].to_vec2();
            points.push(p);
        }
        points.push(*end.pos());
        points
    }

    /// Returns coordinates in polynomial base, where at i-th index is i-th coordinate
    fn bezier_point_in_polynomial_base(&self, start: &Point, end: &Point) -> [Pos2; 4] {
        let v0 = *start.pos();
        let v1 = self.inner_points[0];
        let v2 = self.inner_points[1];
        let v3 = *end.pos();

        let a0 = v0;
        let a1 = (3.0 * (v1 - v0)).to_pos2();
        let a2 = (3.0 * (v2 - 2.0 * v1 + v0.to_vec2())).to_pos2();
        let a3 = (v3 - 3.0 * v2 + 3.0 * v1.to_vec2() - v0.to_vec2()).to_pos2();

        [a0, a1, a2, a3]
    }
}
