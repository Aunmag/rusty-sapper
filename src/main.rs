mod models;
mod utils;

use crate::models::field::Field;
use crate::models::sapper::Sapper;
use crate::models::sapper::SapperAction;

const FIELD_SIZE: usize = 30;
const MINES_DENSITY: f64 = 0.25;

fn main() {
    let mut field = Field::new(FIELD_SIZE, MINES_DENSITY);
    let mut sapper = Sapper::new();

    loop {
        field.render(&sapper);

        if let Option::Some(action) = sapper.request_input(&field) {
            match action {
                SapperAction::Discover => field.discover(sapper.position),
                SapperAction::Mark => field.mark(sapper.position),
                SapperAction::Quit => break,
            }
        }
    }
}
