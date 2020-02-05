use crate::models::field::Field;
use crate::models::sapper::Sapper;
use std::char;
use termwiz::color::AnsiColor;
use termwiz::color::ColorAttribute;

const MARK_UNDISCOVERED: char = '.';
const MARK_DISCOVERED: char = ' ';
const MARK_MARKED: char = '!';
const MARK_MINED: char = '#';

pub struct Cell {
    pub is_mined: bool,
    pub state: CellState,
}

impl Cell {
    pub fn new(is_mined: bool) -> Self {
        return Cell {
            is_mined,
            state: CellState::Undiscovered,
        };
    }

    pub fn toggle_mark(&mut self) {
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

    pub fn get_mark(&self, field: &Field, position: usize, sapper: &Sapper) -> char {
        let mut mark;

        if self.is_mined && (sapper.is_admin || !sapper.is_alive) {
            mark = MARK_MINED;
        } else {
            mark = match self.state {
                CellState::Undiscovered => MARK_UNDISCOVERED,
                CellState::Discovered => char::from_digit(field.count_mines_around(position), 10).unwrap_or('9'),
                CellState::Marked => MARK_MARKED,
            };

            if mark == '0' {
                mark = MARK_DISCOVERED;
            }
        }

        return mark;
    }

    pub fn get_color(mark: char) -> ColorAttribute {
        return match mark {
            '1' => AnsiColor::Blue.into(),
            '2' => AnsiColor::Green.into(),
            '3' => AnsiColor::Red.into(),
            '4' => AnsiColor::Navy.into(),
            '5' => AnsiColor::Maroon.into(),
            '6' => AnsiColor::Aqua.into(),
            '7' | '8' | '9' => AnsiColor::Purple.into(),
            _ => ColorAttribute::Default,
        };
    }

    pub fn get_color_background(mark: char) -> ColorAttribute {
        return match mark {
            MARK_MARKED | MARK_MINED => AnsiColor::Maroon.into(),
            _ => ColorAttribute::Default,
        };
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
