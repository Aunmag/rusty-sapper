mod models;
mod utils;

use crate::models::field::Field;
use crate::models::sapper::Sapper;
use termwiz::caps::Capabilities;
use termwiz::color::ColorAttribute;
use termwiz::input::InputEvent;
use termwiz::input::KeyCode;
use termwiz::input::KeyEvent;
use termwiz::surface::Change;
use termwiz::surface::Position;
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
    let mut update_cursor = true;

    buffer.terminal().set_raw_mode().unwrap();

    loop {
        if render { // TODO: Move to `field.render`
            buffer.add_change(Change::ClearScreen(ColorAttribute::Default));

            for (i, cell) in field.cells.iter().enumerate() {
                let mut mark = cell.get_mark(&field, i);

                if cell.is_mined && (sapper.is_admin || !sapper.is_alive) {
                    mark = '#';
                }

                buffer.add_change(format!("{} ", mark));

                if (i + 1) % field.size == 0 {
                    buffer.add_change("\r\n");
                }
            }
        }

        if render || update_cursor {
            if !sapper.is_alive {
                buffer.add_change("Sorry, but you've taken the wrong step. Game over, press Esc to exit.");
            }

            let (cursor_x, cursor_y) = field.to_position(sapper.position as i32);

            buffer.add_change(Change::CursorPosition {
                x: Position::Absolute(cursor_x as usize * 2),
                y: Position::Absolute(cursor_y as usize),
            });

            buffer.flush().unwrap();
            render = false;
            update_cursor = false;
        }

        match buffer.terminal().poll_input(None) {
            Ok(None) => {},
            Ok(Some(input)) => {
                match input {
                    InputEvent::Key(KeyEvent {key: KeyCode::Escape, ..}) => {
                        break
                    }
                    _ => {
                        if sapper.is_alive {
                            match input {
                                InputEvent::Key(KeyEvent {key: KeyCode::UpArrow, ..}) => {
                                    sapper._move(0, -1, &field);
                                    update_cursor = true;
                                }
                                InputEvent::Key(KeyEvent {key: KeyCode::DownArrow, ..}) => {
                                    sapper._move(0, 1, &field);
                                    update_cursor = true;
                                }
                                InputEvent::Key(KeyEvent {key: KeyCode::LeftArrow, ..}) => {
                                    sapper._move(-1, 0, &field);
                                    update_cursor = true;
                                }
                                InputEvent::Key(KeyEvent {key: KeyCode::RightArrow, ..}) => {
                                    sapper._move(1, 0, &field);
                                    update_cursor = true;
                                }
                                InputEvent::Key(KeyEvent {key: KeyCode::Char('m'), ..}) => {
                                    field.mark(sapper.position);
                                    render = true;
                                }
                                InputEvent::Key(KeyEvent {key: KeyCode::Char(' '), ..}) => {
                                    if !field.discover(sapper.position) {
                                        sapper.is_alive = false;
                                    }

                                    render = true;
                                }
                                _ => {}
                            }
                        }
                    },
                }
            }
            Err(error) => {
                println!("{:?}", error);
                break;
            }
        }
    }

    buffer.add_change(Change::ClearScreen(ColorAttribute::Default));
    buffer.flush().unwrap();
}
