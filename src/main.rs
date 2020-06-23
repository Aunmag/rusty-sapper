mod application;
mod cell;
mod field;
mod game;
mod sapper;
mod ui;
mod utils;

use crate::application::Application;

fn main() {
    Application::new().run();
}
