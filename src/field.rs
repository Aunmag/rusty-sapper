use crate::cell::Cell;
use crate::sapper::Sapper;
use crate::utils;
use rand::prelude::*;
use std::collections::HashSet;
use termwiz::cell::AttributeChange;
use termwiz::color::AnsiColor;
use termwiz::color::ColorAttribute;
use termwiz::surface::Change;
use termwiz::surface::Surface;

const SHIFTS: [i32; 3] = [-1, 0, 1];

pub struct Field {
    size: usize,
    cells: Vec<Cell>,
    mines: HashSet<usize>,
    mines_density: f64,
    cells_discovered_count: usize,
}

impl Field {
    pub fn new(size: usize, mines_density: f64) -> Self {
        let mut cells = Vec::with_capacity(size * size);

        for _ in 0..(size * size) {
            cells.push(Cell::new());
        }

        return Field {
            size,
            cells,
            mines: HashSet::new(), // TODO: Optimize allocation
            mines_density: mines_density,
            cells_discovered_count: 0,
        };
    }

    fn generate_mines(&mut self, excepting_position: usize) {
        self.mines.clear(); // TODO: Optimize reallocation

        let excepting_positions = self.around(excepting_position, true);

        for position in 0..(self.size * self.size) {
            if utils::is_chance(self.mines_density) && !excepting_positions.contains(&position) {
                self.mines.insert(position);
            }
        }
    }

    pub fn explode_mines(&mut self) {
        for position in self.mines.iter() {
            self.cells[*position].is_exploded = true;
        }

        self.mines.clear();
    }

    pub fn discover(&mut self, position: usize) -> bool {
        if self.mines.is_empty() {
            self.generate_mines(position);
        }

        if self.mines.remove(&position) {
            self.cells[position].is_exploded = true;
            return false;
        }

        let cell = &self.cells[position];

        if !cell.is_discovered() {
            let near_positions = self.around(position, false);
            let mut mines_around = 0;

            for position_near in near_positions.iter() {
                if self.is_mined(*position_near) {
                    mines_around += 1;
                }
            }

            self.cells[position].mines_around = Some(mines_around);
            self.cells_discovered_count += 1;

            if mines_around == 0 {
                for position_near in near_positions.iter() {
                    self.discover(*position_near);
                }
            }
        }

        return true;
    }

    pub fn around(&self, center: usize, include_center: bool) -> Vec<usize> {
        // TODO: Find a way to a call lambda while iteration instead of returning an array
        let capacity;

        if include_center {
            capacity = 9;
        } else {
            capacity = 8;
        }

        let mut positions = Vec::with_capacity(capacity);

        for y in SHIFTS.iter() {
            for x in SHIFTS.iter() {
                let x = *x;
                let y = *y;

                if include_center || x != 0 || y != 0 {
                    if let Some(moved) = self.move_position(center, x, y) {
                        positions.push(moved);
                    }
                }
            }
        }

        return positions;
    }

    // TODO: Try to optimize
    pub fn move_position(&self, position: usize, shift_x: i32, shift_y: i32) -> Option<usize> {
        let size = self.size as i32;
        let (x, y) = self.to_coordinate(position);
        let x = x as i32 + shift_x;
        let y = y as i32 + shift_y;

        if 0 <= x && x < size && 0 <= y && y < size {
            return Some((size * y + x) as usize);
        } else {
            return None;
        }
    }

    pub fn to_coordinate(&self, position: usize) -> (usize, usize) {
        return (position % self.size, position / self.size);
    }

    // TODO: Try to optimize
    pub fn to_distance(&self, p1: usize, p2: usize) -> usize {
        let (x1, y1) = self.to_coordinate(p1);
        let (x2, y2) = self.to_coordinate(p2);
        let distance_x = utils::difference(x1, x2);
        let distance_y = utils::difference(y1, y2);
        return distance_x + distance_y;
    }

    pub fn generate_random_position(&self) -> usize {
        return rand::thread_rng().gen_range(0, self.size * self.size);
    }

    pub fn render(&self, sappers: &Vec<Sapper>) -> Surface {
        let mut surface = Surface::new((self.size * 2).saturating_sub(1), self.size);
        let mut player = None;
        let mut sapper_positions = HashSet::with_capacity(sappers.len());

        for sapper in sappers.iter() {
            if sapper.is_alive() {
                if sapper.is_player() {
                    player = Some(sapper);
                } else {
                    sapper_positions.insert(sapper.get_position());
                }
            }
        }

        for (i, cell) in self.cells.iter().enumerate() {
            let is_player_point = player.map(|s| s.get_position() == i).unwrap_or(false);
            let mut mark = cell.get_mark(player.map(|o| o.has_marked(i)).unwrap_or(false));

            if !is_player_point && sapper_positions.contains(&i) {
                mark.background = AnsiColor::Grey.into();
            }

            surface.add_change(Change::Attribute(AttributeChange::Foreground(
                mark.foreground,
            )));

            surface.add_change(Change::Attribute(AttributeChange::Background(
                mark.background,
            )));

            surface.add_change(Change::Attribute(AttributeChange::Reverse(is_player_point)));

            surface.add_change(format!("{}", mark.symbol));

            if (i + 1) % self.size != 0 {
                surface.add_change(Change::Attribute(AttributeChange::Background(
                    ColorAttribute::Default,
                )));

                surface.add_change(Change::Attribute(AttributeChange::Reverse(false)));
                surface.add_change(" ");
            }
        }

        return surface;
    }

    fn is_mined(&self, position: usize) -> bool {
        return self.mines.contains(&position);
    }

    pub fn is_cleaned(&self) -> bool {
        return self.get_cells_undiscovered_count() == 0;
    }

    pub fn get_cells(&self) -> &Vec<Cell> {
        return &self.cells;
    }

    pub fn get_mines_count(&self) -> usize {
        return self.mines.len();
    }

    pub fn get_cells_count(&self) -> usize {
        return self.cells.len();
    }

    pub fn get_cells_discovered_count(&self) -> usize {
        return self.cells_discovered_count;
    }

    pub fn get_cells_undiscovered_count(&self) -> usize {
        return self.get_cells_count() - self.get_mines_count() - self.cells_discovered_count;
    }
}
