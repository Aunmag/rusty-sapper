mod models;
mod utils;

use crate::models::application::Application;

fn main() {
    Application::new().run();
}
