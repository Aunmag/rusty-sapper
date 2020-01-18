mod models;
mod utils;

use crate::models::field::Field;
use crate::models::sapper::Sapper;
use termwiz::caps::Capabilities;
use termwiz::input::InputEvent;
use termwiz::input::KeyCode;
use termwiz::input::KeyEvent;
use termwiz::terminal::buffered::BufferedTerminal;
use termwiz::terminal::new_terminal;
use termwiz::terminal::Terminal;

const FIELD_SIZE: usize = 30;
const MINES_DENSITY: f64 = 0.25;

fn main() {
    let mut field = Field::new(FIELD_SIZE, MINES_DENSITY);
    let mut sapper = Sapper::new();
    let mut buffer = BufferedTerminal::new(new_terminal(Capabilities::new_from_env().unwrap()).unwrap()).unwrap();
    let mut render = true;

    buffer.terminal().set_raw_mode().unwrap();

    loop {
        if render {
            buffer.flush().unwrap();
            field.render(&sapper);
            render = false;
        }

        match buffer.terminal().poll_input(None) {
            Ok(None) => {},
            Ok(Some(input)) => {
                match input {
                    InputEvent::Key(KeyEvent {key: KeyCode::UpArrow, ..}) => {
                        sapper._move(0, -1, &field);
                        render = true;
                    }
                    InputEvent::Key(KeyEvent {key: KeyCode::DownArrow, ..}) => {
                        sapper._move(0, 1, &field);
                        render = true;
                    }
                    InputEvent::Key(KeyEvent {key: KeyCode::LeftArrow, ..}) => {
                        sapper._move(-1, 0, &field);
                        render = true;
                    }
                    InputEvent::Key(KeyEvent {key: KeyCode::RightArrow, ..}) => {
                        sapper._move(1, 0, &field);
                        render = true;
                    }
                    InputEvent::Key(KeyEvent {key: KeyCode::Char('m'), ..}) => {
                        field.mark(sapper.position);
                        render = true;
                    }
                    InputEvent::Key(KeyEvent {key: KeyCode::Char(' '), ..}) => {
                        field.discover(sapper.position);
                        render = true;
                    }
                    InputEvent::Key(KeyEvent {key: KeyCode::Escape, ..}) => {
                        break;
                    }
                    _ => {}
                }
            }
            Err(error) => {
                println!("{:?}", error);
                break;
            }
        }
    }
}
