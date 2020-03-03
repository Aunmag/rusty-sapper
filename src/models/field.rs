use crate::models::cell::Cell;
use crate::models::sapper::Sapper;
use crate::utils;
use termwiz::cell::AttributeChange;
use termwiz::color::ColorAttribute;
use termwiz::surface::Change;
use termwiz::surface::Surface;

const SHIFTS: [i32; 3] = [-1, 0, 1];

pub struct Field {
    pub size: usize,
    pub cells: Vec<Cell>, // TODO: Find a way to use fixed-size array
    pub mines: usize,
    pub mines_density: f64,
    pub is_cleaned: bool,
}

impl Field {
    pub fn new(size: usize, mines_density: f64) -> Self {
        let mut cells = Vec::new();

        for _ in 0..(size * size) {
            cells.push(Cell::new(false));
        }

        return Field {
            size,
            cells,
            mines: 0,
            mines_density: mines_density,
            is_cleaned: false,
        };
    }

    fn generate_mines(&mut self, excepting_position: usize) {
        let excepting_positions = self.around(excepting_position);

        for i in 0..(self.size * self.size) {
            if utils::is_chance(self.mines_density) && !excepting_positions.contains(&i) {
                self.cells[i].is_mined = true;
                self.mines += 1;

                for i_near in self.around(i) {
                    self.cells[i_near].mines_around += 1;
                }
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
            cell.is_discovered = true;

            if cell.mines_around == 0 {
                for i in self.around(position) {
                    if !self.cells[i].is_discovered {
                        self.discover(i);
                    }
                }
            }

            return true;
        }
    }

    pub fn around(&self, center: usize) -> Vec<usize> {
        // TODO: Find a way to a call lambda while iteration instead of returning an array
        let mut positions = Vec::new();

        for y in &SHIFTS {
            for x in &SHIFTS {
                if let Some(moved) = self.move_position(center, *x, *y) {
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
            return Some((size * y + x) as usize);
        } else {
            return None;
        }
    }

    pub fn to_position(&self, i: i32) -> (i32, i32) {
        let size = self.size as i32;
        return (i % size, i / size);
    }

    pub fn render(&self, sapper: &Sapper) -> Surface {
        let mut surface = Surface::new(self.size * 2 - 1, self.size);

        for (i, cell) in self.cells.iter().enumerate() {
            let mark = cell.get_mark(i, &sapper);

            surface.add_change(Change::Attribute(AttributeChange::Foreground(Cell::get_color(mark))));
            surface.add_change(Change::Attribute(AttributeChange::Background(Cell::get_color_background(mark))));
            surface.add_change(Change::Attribute(AttributeChange::Reverse(i == sapper.position)));
            surface.add_change(format!("{}", mark));

            if (i + 1) % self.size != 0 {
                surface.add_change(Change::Attribute(AttributeChange::Background(ColorAttribute::Default)));
                surface.add_change(Change::Attribute(AttributeChange::Reverse(false)));
                surface.add_change(" ");
            }
        }

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
