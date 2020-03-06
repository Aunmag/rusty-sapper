use crate::models::field::Field;
use crate::models::sapper::Sapper;
use termwiz::cell::AttributeChange;
use termwiz::color::AnsiColor;
use termwiz::color::ColorAttribute;
use termwiz::input::InputEvent;
use termwiz::input::KeyCode;
use termwiz::input::KeyEvent;
use termwiz::surface::Change;
use termwiz::surface::Position;
use termwiz::surface::Surface;

const STATISTICS_WIDTH: usize = 14;

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

        if self.sapper.is_alive && !self.field.is_cleaned() {
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
                    self.sapper.toggle_mark();
                    update_screen = true;
                }
                InputEvent::Key(KeyEvent {key: KeyCode::Char(' '), ..}) => {
                    if !self.sapper.has_marked(self.sapper.position) {
                        if self.field.discover(self.sapper.position) {
                            self.sapper.increase_score();
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
        let mut surface = Surface::new(
            self.field.get_size() * 2 + STATISTICS_WIDTH + 1,
            self.field.get_size() + 4
        );

        surface.draw_from_screen(
            &self.field.render(&self.sapper),
            2 + STATISTICS_WIDTH,
            0,
        );

        surface.draw_from_screen(
            &self.render_statistics(),
            0,
            0,
        );

        if !self.sapper.is_alive || self.field.is_cleaned() {
            let message;
            let color;

            surface.add_change(Change::CursorPosition {
                x: Position::Absolute(0),
                y: Position::Absolute(1 + self.field.get_size()),
            });

            if self.sapper.is_alive {
                message = "Well done! You've found the all mines! Press Esc to go back to the main menu.";
                color = AnsiColor::Green.into();
            } else {
                message = "Sorry, but you've taken the wrong step. Game over. Press Esc to go back to the main menu.";
                color = AnsiColor::Red.into();
            }

            surface.add_change(Change::Attribute(AttributeChange::Foreground(color)));
            surface.add_change(message);
        }

        return surface;
    }

    pub fn render_statistics(&self) -> Surface {
        let mut surface = Surface::new(STATISTICS_WIDTH, self.field.get_size());

        surface.add_change("     #GOT #REM");

        surface.add_change(format!(
            "#CLS {:04} {:04}",
            self.field.get_cells_discovered_count(),
            self.field.get_cells_undiscovered_count(),
        ));

        surface.add_change(format!(
            "#MNS {:04} {:04}",
            self.sapper.get_marks_count(),
            self.field.get_mines_count() as i32 - self.sapper.get_marks_count() as i32,
        ));

        surface.add_change("              ");
        surface.add_change("#POS #SPR #SCR");

        for i in 0..(self.field.get_size() - 5) {
            let name;
            let score;

            if i == 0 {
                if !self.sapper.is_alive {
                    surface.add_change(Change::Attribute(AttributeChange::Foreground(AnsiColor::Red.into())));
                } else if self.field.is_cleaned() {
                    surface.add_change(Change::Attribute(AttributeChange::Foreground(AnsiColor::Green.into())));
                }

                surface.add_change(Change::Attribute(AttributeChange::Reverse(true)));
                name = " YOU";
                score = format!("{:04}", self.sapper.get_score());
            } else {
                name = " ---";
                score = "----".to_string();
            }

            surface.add_change(format!("{:04} {} {}", i + 1, name, score));
            surface.add_change(Change::Attribute(AttributeChange::Foreground(ColorAttribute::Default)));
            surface.add_change(Change::Attribute(AttributeChange::Reverse(false)));
        }

        return surface;
    }
}
