use crate::cell::Cell;
use crate::event::EventData;
use crate::event::EventManager;
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
    size: u8,
    cells: Vec<Cell>,
    mines: HashSet<u16>,
    mines_density: f64,
    cells_discovered_count: usize,
}

impl Field {
    pub fn new(size: u8, mines_density: f64) -> Self {
        let size_full = usize::from(size).pow(2);
        let mut cells = Vec::with_capacity(size_full);

        for _ in 0..size_full {
            cells.push(Cell::new());
        }

        return Self {
            size,
            cells,
            mines: HashSet::new(), // TODO: Optimize allocation
            mines_density,
            cells_discovered_count: 0,
        };
    }

    fn generate_mines(&mut self, excepting_position: u16) {
        self.mines.clear(); // TODO: Optimize reallocation

        let excepting_positions = self.around(excepting_position, true);

        for position in 0..self.get_size_full() {
            if utils::is_chance(self.mines_density) && !excepting_positions.contains(&position) {
                self.mines.insert(position);
            }
        }
    }

    pub fn explode_mines(&mut self) {
        for position in &self.mines {
            if let Some(cell) = self.cells.get_mut(usize::from(*position)) {
                cell.is_exploded = true;
            }
        }

        self.mines.clear();
    }

    pub fn discover(
        &mut self,
        position: u16,
        events: &mut EventManager,
    ) -> DiscoveryResult {
        if self.mines.is_empty() {
            self.generate_mines(position);
        }

        let cell = if let Some(cell) = self.cells.get_mut(usize::from(position)) {
            cell
        } else {
            // TODO: Log error
            return DiscoveryResult::AlreadyDiscovered;
        };

        if cell.is_exploded || cell.is_discovered() {
            return DiscoveryResult::AlreadyDiscovered;
        } else if self.mines.contains(&position) {
            // TODO: Probably I should delete the mine here too
            cell.is_exploded = true;

            events.fire(
                EventData::CellExplode {
                    position,
                },
                None,
                None,
            );

            return DiscoveryResult::Failure;
        } else {
            let near_positions = self.around(position, false);
            let mut mines_around = 0;

            for position_near in &near_positions {
                if self.is_mined(*position_near) {
                    mines_around += 1;
                }
            }

            if let Some(cell) = self.get_cell_mut(position) { // TODO: Simplify, since I already should have `cell`
                cell.mines_around = Some(mines_around);
            }

            self.cells_discovered_count += 1;

            events.fire(
                EventData::CellDiscover {
                    position,
                    mines_around,
                },
                None,
                None,
            );

            if mines_around == 0 {
                for position_near in &near_positions {
                    self.discover(*position_near, events);
                }
            }

            return DiscoveryResult::Success;
        }
    }

    // TODO: Return stack-allocated array
    pub fn around(&self, center: u16, include_center: bool) -> Vec<u16> {
        // TODO: Find a way to a call lambda while iteration instead of returning an array
        let mut positions = Vec::with_capacity(9);

        for y in &SHIFTS {
            for x in &SHIFTS {
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
    pub fn move_position(&self, position: u16, shift_x: i32, shift_y: i32) -> Option<u16> {
        let size = i32::from(self.size);
        let (x, y) = self.to_coordinate(position);
        let x = i32::from(x) + shift_x;
        let y = i32::from(y) + shift_y;

        if 0 <= x && x < size && 0 <= y && y < size {
            return u16::try_from(size * y + x).ok();
        } else {
            return None;
        }
    }

    #[allow(clippy::integer_division)] // TODO: Try to resolve
    pub fn to_coordinate(&self, position: u16) -> (u16, u16) {
        return (position % u16::from(self.size), position / u16::from(self.size));
    }

    // TODO: Try to optimize
    pub fn to_distance(&self, p1: u16, p2: u16) -> u16 {
        let (x1, y1) = self.to_coordinate(p1);
        let (x2, y2) = self.to_coordinate(p2);
        let distance_x = utils::difference(x1, x2);
        let distance_y = utils::difference(y1, y2);
        return distance_x + distance_y;
    }

    pub fn generate_random_position(&self) -> u16 {
        return rand::thread_rng().gen_range(0, self.get_size_full());
    }

    pub fn render(&self, sappers: &[Sapper]) -> Surface {
        let mut surface = Surface::new(
            usize::from(self.size * 2).saturating_sub(1),
            usize::from(self.size)
        );

        let mut player = None;
        let mut sapper_positions = HashSet::with_capacity(sappers.len());

        for sapper in sappers {
            if sapper.is_alive() {
                if sapper.is_player() {
                    player = Some(sapper);
                } else {
                    sapper_positions.insert(sapper.get_position());
                }
            }
        }

        for (i, cell) in self.cells.iter().enumerate() {
            let cell_position = match u16::try_from(i) {
                Ok(cell_position) => cell_position,
                Err(_) => {
                    // TODO: Log error
                    break;
                }
            };

            let is_player_point = player.map_or(false, |s| s.get_position() == cell_position);
            let mut mark = cell.get_mark(player.map_or(false, |o| o.has_marked(cell_position)));

            if !is_player_point && sapper_positions.contains(&cell_position) {
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

            if (cell_position + 1) % u16::from(self.size) != 0 {
                surface.add_change(Change::Attribute(AttributeChange::Background(
                    ColorAttribute::Default,
                )));

                surface.add_change(Change::Attribute(AttributeChange::Reverse(false)));
                surface.add_change(" ");
            }
        }

        return surface;
    }

    fn is_mined(&self, position: u16) -> bool {
        return self.mines.contains(&position);
    }

    pub fn is_cleaned(&self) -> bool {
        return self.get_cells_undiscovered_count() == 0;
    }

    pub const fn get_size(&self) -> u8 {
        return self.size;
    }

    pub fn get_size_full(&self) -> u16 {
        let size_u16 = u16::from(self.size);
        return size_u16 * size_u16;
    }

    pub const fn get_cells(&self) -> &Vec<Cell> {
        return &self.cells;
    }

    pub fn get_cell(&self, position: u16) -> Option<&Cell> {
        return self.cells.get(usize::from(position));
    }

    pub fn get_cell_mut(&mut self, position: u16) -> Option<&mut Cell> {
        return self.cells.get_mut(usize::from(position));
    }

    pub fn get_mines_count(&self) -> usize {
        return self.mines.len();
    }

    pub fn get_cells_count(&self) -> usize {
        return self.cells.len();
    }

    pub const fn get_cells_discovered_count(&self) -> usize {
        return self.cells_discovered_count;
    }

    pub fn get_cells_undiscovered_count(&self) -> usize {
        return self.get_cells_count() - self.get_mines_count() - self.cells_discovered_count;
    }
}

pub enum DiscoveryResult {
    Success,
    Failure,
    AlreadyDiscovered,
}
