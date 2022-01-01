use crate::event::EventManager;
use crate::event::Event;
use crate::field::Field;
use crate::sapper::Sapper;
use termwiz::cell::AttributeChange;
use termwiz::color::AnsiColor;
use termwiz::color::ColorAttribute;
use termwiz::input::InputEvent;
use termwiz::surface::Change;
use termwiz::surface::Surface;

const STATISTICS_WIDTH: usize = 16;

pub struct Game {
    pub field: Field,
    pub sappers: Vec<Sapper>,
    pub events: EventManager,
}

impl Game {
    pub fn new(field: Field, sappers: Vec<Sapper>) -> Self {
        return Self {
            field,
            sappers,
            events: EventManager::new(),
        };
    }

    pub fn update(&mut self, input: Option<&InputEvent>) -> Vec<Event> {
        let mut explode_mines = !self.sappers.is_empty();
        let mut local_events = Vec::new();

        if !self.field.is_cleaned() {
            for sapper in &mut self.sappers {
                sapper.update(&mut self.field, input);

                if explode_mines && sapper.is_alive() {
                    explode_mines = false;
                }

                let mut events = sapper.get_events_mut().pull();

                while let Some(event) = events.pop() {
                    local_events.push(event.clone()); // TODO: Optimize

                    self.events.fire(
                        event.data,
                        None,
                        None,
                    );
                }
            }
        }

        if explode_mines {
            self.field.explode_mines();
        }

        return local_events;
    }

    pub fn render(&self) -> Surface {
        let statistics = self.render_statistics();
        let field = self.field.render(&self.sappers);
        let (statistics_width, statistics_height) = statistics.dimensions();
        let (field_width, field_height) = field.dimensions();

        let mut surface = Surface::new(
            statistics_width + field_width,
            std::cmp::max(statistics_height, field_height),
        );

        surface.draw_from_screen(&statistics, 0, 0);
        surface.draw_from_screen(&field, statistics_width, 0);

        return surface;
    }

    pub fn render_statistics(&self) -> Surface {
        let mut surface = Surface::new(STATISTICS_WIDTH, self.sappers.len() + 5);
        let marks = self.get_player().map_or(0, Sapper::get_marks_count);

        surface.add_change(format!(
            "     #GOT #REM  #CLS {:04} {:04}  #MNS {:04} {:04}                  #POS #SPR #SCR  ",
            self.field.get_cells_discovered_count(),
            self.field.get_cells_undiscovered_count(),
            marks,
            self.field.get_mines_count().saturating_sub(marks),
        ));

        for (i, sapper) in self.get_sappers_sorted_by_score().iter().enumerate() {
            if !sapper.is_alive() {
                surface.add_change(Change::Attribute(AttributeChange::Foreground(
                    AnsiColor::Red.into(),
                )));
            }

            if sapper.is_player() {
                surface.add_change(Change::Attribute(AttributeChange::Reverse(true)));

                if sapper.is_alive() && self.field.is_cleaned() {
                    surface.add_change(Change::Attribute(AttributeChange::Foreground(
                        AnsiColor::Green.into(),
                    )));
                }
            }

            surface.add_change(format!(
                "{:04}  {} {:04}",
                i + 1,
                sapper.get_name(),
                sapper.get_score(),
            ));

            surface.add_change(Change::Attribute(AttributeChange::Foreground(
                ColorAttribute::Default,
            )));
            surface.add_change(Change::Attribute(AttributeChange::Reverse(false)));
            surface.add_change("  ");
        }

        return surface;
    }

    pub fn get_sappers_sorted_by_score(&self) -> Vec<&Sapper> {
        let mut sappers = Vec::with_capacity(self.sappers.len());

        for sapper in &self.sappers {
            sappers.push(sapper);
        }

        sappers.sort_by(|sapper_1, sapper_2| {
            let score_1 = sapper_1.get_score();
            let score_2 = sapper_2.get_score();
            return score_1.cmp(&score_2).reverse();
        });

        return sappers;
    }

    pub fn get_sapper_mut(&mut self, id: u8) -> Option<&mut Sapper> {
        for sapper in &mut self.sappers {
            if sapper.get_id() == id {
                return Some(sapper);
            }
        }

        return None;
    }

    // TODO: Try no to use since it is slow
    pub fn get_player(&self) -> Option<&Sapper> {
        for sapper in &self.sappers {
            if sapper.is_player() {
                return Some(sapper);
            }
        }

        return None;
    }
}
