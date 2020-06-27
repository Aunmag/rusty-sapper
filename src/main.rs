mod application;
mod cell;
mod event;
mod field;
mod game;
mod net;
mod sapper;
mod ui;
mod utils;

use crate::application::Application;

fn main() {
    Application::new().run();
}
