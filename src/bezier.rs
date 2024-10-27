use egui::Pos2;

use crate::point::Point;

#[derive(Debug, Clone, Copy)]
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
        let points_count = start.pos().distance(*end.pos()) * 6.0;
        let d = 1.0 / points_count;
        let mut points = Vec::with_capacity(points_count as usize);
        let mut t = 0.0;
        let mut p = polynomial_base[0];
        let mut p_delta = d
            * (polynomial_base[1]
                + d * (polynomial_base[2] + d * polynomial_base[3].to_vec2()).to_vec2());
        let mut p2_delta =
            2.0 * d * d * (3.0 * polynomial_base[3] * d + polynomial_base[2].to_vec2());
        let p3_delta = 6.0 * d * d * d * polynomial_base[3];
        while t <= 1.0 {
            points.push(p);
            p += p_delta.to_vec2();
            p_delta += p2_delta.to_vec2();
            p2_delta += p3_delta.to_vec2();
            t += d;
        }
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
