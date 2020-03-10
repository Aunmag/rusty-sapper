use crate::models::field::Field;
use crate::models::sapper::Sapper;
use crate::models::sapper::SapperType;
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
    pub sappers: Vec<Sapper>,
    observer_id: u8,
}

impl Game {
    pub fn new(field_size: usize, mines_density: f64, bots: u8, bots_reaction: f64) -> Self {
        let field = Field::new(field_size, mines_density);

        let mut sappers = Vec::with_capacity(bots as usize + 1);

        sappers.push(Sapper::new(
            0,
            SapperType::Player,
            field.generate_random_position(),
            0.0,
        ));

        for i in 0..bots {
            sappers.push(Sapper::new(
                i + 1,
                SapperType::Bot,
                field.generate_random_position(),
                bots_reaction,
            ));
        }

        return Game {
            field,
            sappers,
            observer_id: 0,
        };
    }

    pub fn update(&mut self, input: &Option<InputEvent>) {
        match input {
            Some(InputEvent::Key(KeyEvent { key: KeyCode::PageUp, .. })) => {
                self.observer_id = self.observer_id.saturating_sub(1);
            }
            Some(InputEvent::Key(KeyEvent { key: KeyCode::PageDown, .. })) => {
                self.observer_id = self.observer_id.saturating_add(1);
            }
            _ => {}
        }

        if !self.field.is_cleaned() {
            for sapper in self.sappers.iter_mut() {
                sapper.update(&mut self.field, &input);
            }
        }
    }

    pub fn render(&self) -> Surface {
        let is_observer_alive = self.get_observer().map(|s| s.is_alive).unwrap_or(false);
        let mut surface = Surface::new(
            self.field.get_size() * 2 + STATISTICS_WIDTH + 1,
            self.field.get_size() + 4
        );

        surface.draw_from_screen(
            &self.field.render(&self.sappers, self.observer_id),
            2 + STATISTICS_WIDTH,
            0,
        );

        surface.draw_from_screen(
            &self.render_statistics(),
            0,
            0,
        );

        if !is_observer_alive || self.field.is_cleaned() {
            let message;
            let color;

            surface.add_change(Change::CursorPosition {
                x: Position::Absolute(0),
                y: Position::Absolute(1 + self.field.get_size()),
            });

            if is_observer_alive {
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
        let sappers = self.get_sappers_sorted_by_score();
        let marks = self.get_observer().map(|p| p.get_marks_count()).unwrap_or(0);

        surface.add_change(format!(
            "     #GOT #REM#CLS {:04} {:04}#MNS {:04} {:04}              #POS #SPR #SCR",
            self.field.get_cells_discovered_count(),
            self.field.get_cells_undiscovered_count(),
            marks,
            self.field.get_mines_count() as i32 - marks as i32,
        ));

        for i in 0..(self.field.get_size() - 5) {
            let sapper = sappers.get(i);

            if let Some(sapper) = sappers.get(i) {
                if !sapper.is_alive {
                    surface.add_change(Change::Attribute(AttributeChange::Foreground(AnsiColor::Red.into())));
                }

                if sapper.get_id() == self.observer_id {
                    surface.add_change(Change::Attribute(AttributeChange::Reverse(true)));

                    if sapper.is_alive && self.field.is_cleaned() {
                        surface.add_change(Change::Attribute(AttributeChange::Foreground(AnsiColor::Green.into())));
                    }
                }
            }

            surface.add_change(format!(
                "{:04}  {} {}",
                i + 1,
                sapper.map(|s| s.get_name()).unwrap_or(&"---".to_string()),
                sapper.map(|s| format!("{:04}", s.get_score())).unwrap_or("----".to_string()),
            ));

            surface.add_change(Change::Attribute(AttributeChange::Foreground(ColorAttribute::Default)));
            surface.add_change(Change::Attribute(AttributeChange::Reverse(false)));
        }

        return surface;
    }

    pub fn get_sappers_sorted_by_score(&self) -> Vec<&Sapper> {
        let mut sappers = Vec::with_capacity(self.sappers.len());

        for sapper in self.sappers.iter() {
            sappers.push(sapper);
        }

        sappers.sort_by(|sapper_1, sapper_2| {
            let score_1 = sapper_1.get_score();
            let score_2 = sapper_2.get_score();
            return score_1.cmp(&score_2).reverse();
        });

        return sappers;
    }

    // TODO: Try no to use since it is slow
    pub fn get_observer(&self) -> Option<&Sapper> {
        for sapper in self.sappers.iter() {
            if sapper.get_id() == self.observer_id {
                return Some(sapper);
            }
        }

        return None;
    }
}
