use crate::models::cell::Cell;
use crate::models::cell::CellState;
use crate::models::sapper::Sapper;
use crate::utils;

pub struct Field {
    pub size: usize,
    pub cells: Vec<Cell>,
    mines_density: f64,
    has_mines: bool,
}

impl Field {

    pub fn new(size: usize, mines_density: f64) -> Field {
        let mut cells = Vec::new();

        for _ in 0..(size * size) {
            cells.push(Cell::new(false));
        }

        return Field {size, cells, mines_density, has_mines: false};
    }

    fn generate_mines(&mut self, excepting_position: usize) {
        let excepting_positions = self.around(excepting_position);

        for (i, cell) in &mut self.cells.iter_mut().enumerate() {
            if utils::is_chance(self.mines_density) && !excepting_positions.contains(&i) {
                cell.is_mined = true;
            }
        }
    }

    pub fn discover(&mut self, position: usize) {
        if !self.has_mines {
            self.generate_mines(position);
            self.has_mines = true;
        }

        let cell = &mut self.cells[position];

        cell.discover();

        if cell.state == CellState::Discovered && self.count_mines(position) == 0 {
            for i in self.around(position) {
                if self.cells[i].state == CellState::Undiscovered {
                    self.discover(i)
                }
            }
        }
    }

    pub fn mark(&mut self, position: usize) {
        self.cells[position].mark();
    }

    pub fn count_mines(&self, position: usize) -> u32 {
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
        let s = self.size as i32;
        let x = shift_x + (i as i32 % s);
        let y = shift_y + (i as i32 / s);

        if 0 <= x && x < s && 0 <= y && y < s {
            return Option::Some((s * y + x) as usize);
        } else {
            return Option::None;
        }
    }

    pub fn render(&self, sapper: &Sapper) {
        for (i, cell) in self.cells.iter().enumerate() {
            let mut mark;

            if i == sapper.position {
                mark = 'X';
            } else {
                mark = cell.get_mark(&self, i);
            }

            if cell.is_mined && (sapper.is_admin || !sapper.is_alive) {
                mark = '#';
            }

            print!("{} ", mark);

            if (i + 1) % self.size == 0 {
                println!();
            }
        }
    }

}
