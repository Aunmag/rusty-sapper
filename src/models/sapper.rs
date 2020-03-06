use crate::models::field::Field;
use std::collections::HashSet;

pub struct Sapper {
    pub position: usize,
    pub is_alive: bool,
    marks: HashSet<usize>,
    score: u16,
}

impl Sapper {
    pub fn new() -> Self {
        return Sapper {
            position: 0,
            is_alive: true,
            marks: HashSet::new(),
            score: 0,
        };
    }

    pub fn _move(&mut self, x: i32, y: i32, field: &Field) {
        if let Some(position) = field.move_position(self.position, x, y) {
            self.position = position;
        }
    }

    pub fn increase_score(&mut self) {
        self.score += 1;
    }

    pub fn toggle_mark(&mut self) {
        if self.has_marked(self.position) {
            self.marks.remove(&self.position);
        } else {
            self.marks.insert(self.position);
        }
    }

    pub fn has_marked(&self, position: usize) -> bool {
        return self.marks.contains(&position);
    }

    pub fn get_marks_count(&self) -> usize {
        return self.marks.len();
    }

    pub fn get_score(&self) -> u16 {
        return self.score;
    }
}
