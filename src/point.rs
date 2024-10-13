use egui::{Pos2, Vec2};

use crate::edge::{self, Edge};

#[derive(Clone, Copy)]
pub struct Point {
    pos: Pos2,
}

impl Point {
    pub fn new(pos: Pos2) -> Self {
        Self { pos }
    }

    pub fn pos(&self) -> &Pos2 {
        &self.pos
    }

    pub fn pos_mut(&mut self) -> &mut Pos2 {
        &mut self.pos
    }

    pub fn update_position(
        point_index: usize,
        new_position: Pos2,
        points: &mut [Point],
        edges: &[Edge],
    ) {
        let delta = new_position - *points[point_index].pos();
        points[point_index].pos = new_position;
        Self::adjust_adjacent_edges_after_position_update(point_index, points, edges, delta);
    }

    fn adjust_adjacent_edges_after_position_update(
        point_index: usize,
        points: &mut [Point],
        edges: &[Edge],
        delta: Vec2,
    ) {
        let adjacent_edges = Self::find_adjacent_edges(point_index, edges);
        match adjacent_edges {
            (None, None) => eprintln!("Trying to move point that belongs to no edge"),
            (None, Some(_)) => eprintln!("Trying to move point that belongs to only one edge"),
            (Some(_), None) => eprintln!("Trying to move point that belongs to only one edge"),
            (Some(first), Some(second)) => {
                Self::adjust_moved_point_edge(points, edges, point_index, first, &delta);
                Self::adjust_moved_point_edge(points, edges, point_index, second, &delta);
            }
        }
    }

    fn adjust_moved_point_edge(
        points: &mut [Point],
        edges: &[Edge],
        point_index: usize,
        edge_index: usize,
        delta: &Vec2,
    ) {
        if let Some(restriction) = edges[edge_index].restriction() {
            match restriction {
                edge::EdgeRestriction::Horizontal => {
                    let other_point = edges[edge_index].take_other_point(point_index);
                    points[other_point].pos_mut().y += delta.y;
                }
                edge::EdgeRestriction::Vertical => {
                    let other_point = edges[edge_index].take_other_point(point_index);
                    points[other_point].pos_mut().x += delta.x;
                }
                edge::EdgeRestriction::ConstWidth => todo!(),
            }
        }
    }

    pub fn find_adjacent_edges(
        point_index: usize,
        edges: &[Edge],
    ) -> (Option<usize>, Option<usize>) {
        let mut first: Option<usize> = None;
        let mut second: Option<usize> = None;
        for (id, edge) in edges.iter().enumerate() {
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

    pub fn get_middle_point(start: &Point, end: &Point) -> Pos2 {
        (start.pos + end.pos().to_vec2()) / 2.0
    }

    pub fn add_on_edge(points: &mut Vec<Point>, edges: &mut Vec<Edge>, edge_index: usize) {
        let edge = edges.remove(edge_index);
        let id_smaller = edge.start_index.min(edge.end_index);
        let id_bigger = edge.start_index.max(edge.end_index);
        let new_point = Self::get_middle_point(&points[id_smaller], &points[id_bigger]);
        points.push(Self::new(new_point));
        let first_edge = Edge::new(id_smaller, points.len() - 1);
        let second_edge = Edge::new(points.len() - 1, id_bigger);
        edges.push(first_edge);
        edges.push(second_edge);
    }

    pub fn remove_at(points: &mut Vec<Point>, edges: &mut Vec<Edge>, point_index: usize) {
        points.remove(point_index);
        let adjacent_edges = Self::find_adjacent_edges(point_index, edges);
        for edge in edges.iter_mut() {
            if edge.start_index > point_index {
                edge.start_index -= 1;
            }
            if edge.end_index > point_index {
                edge.end_index -= 1;
            }
        }
        match adjacent_edges {
            // Those should never happen (?)
            (None, None) => eprintln!("Trying to remove point that belongs to no edge"),
            (None, Some(_)) => eprintln!("Trying to remove point that belongs to only one edge"),
            (Some(_), None) => eprintln!("Trying to remove point that belongs to only one edge"),
            (Some(first_id), Some(second_id)) => {
                let first = edges.remove(first_id);
                // -1 becasue we already removed one item
                let second = edges.remove(second_id - 1);
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
                edges.push(new_edge);
            }
        }
    }

    pub fn update_position_all(points: &mut [Point], diff: Vec2) {
        for point in points {
            *point.pos_mut() += diff;
        }
    }
}
