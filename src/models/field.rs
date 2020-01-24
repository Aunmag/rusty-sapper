use crate::models::cell::Cell;
use crate::models::cell::CellState;
use crate::models::sapper::Sapper;
use crate::utils;
use termwiz::cell::AttributeChange;
use termwiz::color::AnsiColor;
use termwiz::surface::Change;
use termwiz::surface::Surface;

pub struct Field {
    pub size: usize,
    pub cells: Vec<Cell>,
    pub mines: usize,
    pub mines_density: f64,
    pub is_cleaned: bool,
}

impl Field {

    pub fn new(size: usize, mines_density: f64) -> Field {
        let mut cells = Vec::new();

        for _ in 0..(size * size) {
            cells.push(Cell::new(false));
        }

        return Field {
            size,
            cells,
            mines: 0,
            mines_density,
            is_cleaned: false,
        };
    }

    fn generate_mines(&mut self, excepting_position: usize) {
        let excepting_positions = self.around(excepting_position);

        for (i, cell) in &mut self.cells.iter_mut().enumerate() {
            if utils::is_chance(self.mines_density) && !excepting_positions.contains(&i) {
                cell.is_mined = true;
                self.mines += 1;
            }
        }
    }

    pub fn discover(&mut self, position: usize) -> bool {
        if self.mines == 0 {
            self.generate_mines(position);
        }

        let cell = &mut self.cells[position];

        if cell.is_mined {
            return false;
        } else {
            cell.discover();

            if cell.state == CellState::Discovered && self.count_mines_around(position) == 0 {
                for i in self.around(position) {
                    if self.cells[i].state == CellState::Undiscovered {
                        self.discover(i);
                    }
                }
            }

            return true;
        }
    }

    pub fn count_mines_around(&self, position: usize) -> u32 {
        let mut mines = 0;

        for i in self.around(position) {
            if self.cells[i].is_mined {
                mines += 1;
            }
        }

        return mines;
    }

    pub fn around(&self, center: usize) -> Vec<usize> {
        // TODO: Find a way to a call lambda while iteration instead of returning an array

        let a: Vec<i32> = vec![-1, 0, 1]; // TODO: To constant
        let mut positions = Vec::new();

        for y in &a {
            for x in &a {
                if let Option::Some(moved) = self.move_position(center, *x, *y) {
                    positions.push(moved);
                }
            }
        }

        return positions;
    }

    pub fn move_position(&self, i: usize, shift_x: i32, shift_y: i32) -> Option<usize> {
        // TODO: Try to optimize and simplify
        let size = self.size as i32;
        let (mut x, mut y) = self.to_position(i as i32);
        x += shift_x;
        y += shift_y;

        if 0 <= x && x < size && 0 <= y && y < size {
            return Option::Some((size * y + x) as usize);
        } else {
            return Option::None;
        }
    }

    pub fn to_position(&self, i: i32) -> (i32, i32) {
        let size = self.size as i32;
        return (i % size, i / size);
    }

    pub fn render(&self, sapper: &Sapper) -> Surface {
        let mut surface = Surface::new(self.size * 2 + 1, self.size + 1);

        for (i, cell) in self.cells.iter().enumerate() {
            let mark = cell.get_mark(&self, i, &sapper);

            surface.add_change(Change::Attribute(AttributeChange::Foreground(Cell::get_color(mark).into())));
            surface.add_change(Change::Attribute(AttributeChange::Reverse(Cell::is_reversed(mark))));
            surface.add_change(format!("{}", mark));
            surface.add_change(Change::Attribute(AttributeChange::Reverse(false)));
            surface.add_change(" ");

            if (i + 1) % self.size == 0 {
                surface.add_change("\r\n");
            }
        }

        surface.add_change(Change::Attribute(AttributeChange::Foreground(AnsiColor::Silver.into())));

        return surface;
    }

    pub fn update_is_cleaned(&mut self) {
        self.is_cleaned = true;

        for cell in &self.cells {
            if !cell.is_cleaned() {
                self.is_cleaned = false;
                break;
            }
        }
    }

}
