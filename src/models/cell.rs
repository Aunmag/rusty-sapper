use crate::models::field::Field;
use std::char;

pub struct Cell {
    pub is_mined: bool,
    pub state: CellState,
}

impl Cell {

    pub fn new(is_mined: bool) -> Cell {
        return Cell {
            is_mined,
            state: CellState::Undiscovered,
        };
    }

    pub fn mark(&mut self) {
        match self.state {
            CellState::Undiscovered => self.state = CellState::Marked,
            CellState::Marked => self.state = CellState::Undiscovered,
            _ => {},
        }
    }

    pub fn discover(&mut self) {
        if self.state == CellState::Undiscovered {
            self.state = CellState::Discovered;
        }
    }

    pub fn get_mark(&self, field: &Field, position: usize) -> char {
        let mut mark = match self.state {
            CellState::Marked => 'M',
            CellState::Undiscovered => '.',
            CellState::Discovered => char::from_digit(field.count_mines(position), 10).unwrap_or('+'),
        };

        if mark == '0' {
            mark = ' ';
        }

        return mark;
    }

    pub fn is_cleaned(&self) -> bool {
        return match self.state {
            CellState::Undiscovered => false,
            CellState::Discovered => true,
            CellState::Marked => self.is_mined,
        }
    }

}

#[derive(PartialEq)]
pub enum CellState {
    Undiscovered,
    Discovered,
    Marked,
}
