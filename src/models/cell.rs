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
    pub is_discovered: bool,
    pub mines_around: usize,
}

impl Cell {
    pub fn new(is_mined: bool) -> Self {
        return Cell {
            is_mined,
            is_discovered: false,
            mines_around: 0,
        };
    }

    pub fn get_mark(&self, position: usize, sapper: &Sapper) -> char {
        let mark;

        if self.is_mined && !sapper.is_alive {
            mark = MARK_MINED;
        } else {
            if self.is_discovered {
                if self.mines_around == 0 {
                    mark = MARK_DISCOVERED;
                } else {
                    mark = char::from_digit(self.mines_around as u32, 10).unwrap_or('9');
                }
            } else {
                if sapper.has_marked(position) {
                    mark = MARK_MARKED;
                } else {
                    mark = MARK_UNDISCOVERED;
                }
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
        return self.is_mined || self.is_discovered;
    }
}
