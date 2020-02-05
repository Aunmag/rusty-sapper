use crate::models::cell::CellState;
use crate::models::field::Field;
use crate::models::sapper::Sapper;
use termwiz::input::InputEvent;
use termwiz::input::KeyCode;
use termwiz::input::KeyEvent;
use termwiz::surface::Surface;

pub struct Game {
    pub field: Field,
    pub sapper: Sapper,
}

impl Game {
    pub fn new(field_size: usize, mines_density: f64) -> Self {
        return Game {
            field: Field::new(field_size, mines_density),
            sapper: Sapper::new(),
        }
    }

    pub fn update(&mut self, input: &InputEvent) -> bool {
        let mut update_screen = false;

        if self.sapper.is_alive && !self.field.is_cleaned {
            match input {
                InputEvent::Key(KeyEvent {key: KeyCode::UpArrow, ..}) => {
                    self.sapper._move(0, -1, &self.field);
                    update_screen = true;
                }
                InputEvent::Key(KeyEvent {key: KeyCode::DownArrow, ..}) => {
                    self.sapper._move(0, 1, &self.field);
                    update_screen = true;
                }
                InputEvent::Key(KeyEvent {key: KeyCode::LeftArrow, ..}) => {
                    self.sapper._move(-1, 0, &self.field);
                    update_screen = true;
                }
                InputEvent::Key(KeyEvent {key: KeyCode::RightArrow, ..}) => {
                    self.sapper._move(1, 0, &self.field);
                    update_screen = true;
                }
                InputEvent::Key(KeyEvent {key: KeyCode::Char('m'), ..}) => {
                    self.field.cells[self.sapper.position].toggle_mark();
                    self.field.update_is_cleaned();
                    update_screen = true;
                }
                InputEvent::Key(KeyEvent {key: KeyCode::Char(' '), ..}) => {
                    if self.field.cells[self.sapper.position].state != CellState::Marked {
                        if self.field.discover(self.sapper.position) {
                            self.field.update_is_cleaned();
                        } else {
                            self.sapper.is_alive = false;
                        }

                        update_screen = true;
                    }
                }
                _ => {}
            }
        }

        return update_screen;
    }

    pub fn render(&self) -> Surface {
        let mut surface = self.field.render(&self.sapper);
        let (size_x, size_y) = surface.dimensions();

        if self.field.is_cleaned || !self.sapper.is_alive {
            surface.resize(size_x, size_y + 8);

            if !self.sapper.is_alive {
                surface.add_change("\r\n\r\nSorry, but you've taken the wrong step. Game over, press Esc to go back to the main menu.");
            }

            if self.field.is_cleaned {
                surface.add_change("\r\n\r\nWell done! You've found the all mines! Press Esc to go back to the main menu.");
            }
        }

        return surface;
    }
}
