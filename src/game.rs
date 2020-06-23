use crate::field::Field;
use crate::sapper::Sapper;
use crate::sapper::SapperBehavior;
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
}

impl Game {
    pub fn new(field_size: usize, mines_density: f64, bots: u8, bots_reaction: f64) -> Self {
        let field = Field::new(field_size, mines_density);

        let mut sappers = Vec::with_capacity(bots as usize + 1);

        sappers.push(Sapper::new(
            SapperBehavior::Player,
            field.generate_random_position(),
            0.0,
        ));

        for _ in 0..bots {
            sappers.push(Sapper::new(
                SapperBehavior::Bot,
                field.generate_random_position(),
                bots_reaction,
            ));
        }

        return Game { field, sappers };
    }

    pub fn update(&mut self, input: Option<&InputEvent>) {
        let mut explode_mines = !self.sappers.is_empty();

        if !self.field.is_cleaned() {
            for sapper in self.sappers.iter_mut() {
                sapper.update(&mut self.field, input);

                if explode_mines && sapper.is_alive() {
                    explode_mines = false;
                }
            }
        }

        if explode_mines {
            self.field.explode_mines();
        }
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
        let marks = self.get_player().map(|p| p.get_marks_count()).unwrap_or(0);

        surface.add_change(format!(
            "     #GOT #REM  #CLS {:04} {:04}  #MNS {:04} {:04}                  #POS #SPR #SCR  ",
            self.field.get_cells_discovered_count(),
            self.field.get_cells_undiscovered_count(),
            marks,
            self.field.get_mines_count() as i32 - marks as i32,
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
                "{:04}  {} {}",
                i + 1,
                sapper.get_name(),
                format!("{:04}", sapper.get_score()),
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
    pub fn get_player(&self) -> Option<&Sapper> {
        for sapper in self.sappers.iter() {
            if sapper.is_player() {
                return Some(sapper);
            }
        }

        return None;
    }
}
