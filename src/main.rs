mod models;
mod utils;

use crate::models::cell::CellState;
use crate::models::field::Field;
use crate::models::sapper::Sapper;
use termwiz::caps::Capabilities;
use termwiz::color::ColorAttribute;
use termwiz::input::InputEvent;
use termwiz::input::KeyCode;
use termwiz::input::KeyEvent;
use termwiz::surface::Change;
use termwiz::surface::CursorShape;
use termwiz::surface::Position;
use termwiz::terminal::buffered::BufferedTerminal;
use termwiz::terminal::new_terminal;
use termwiz::terminal::Terminal;

const FIELD_SIZE: usize = 24;
const MINES_DENSITY: f64 = 0.2;

fn main() {
    let mut field = Field::new(FIELD_SIZE, MINES_DENSITY);
    let mut sapper = Sapper::new();
    let mut terminal = BufferedTerminal::new(new_terminal(Capabilities::new_from_env().unwrap()).unwrap()).unwrap();
    let mut update_screen = true;

    terminal.terminal().set_raw_mode().unwrap();
    terminal.add_change(Change::CursorShape(CursorShape::Hidden));

    loop {
        if update_screen {
            terminal.draw_from_screen(&field.render(&sapper), 0, 0);

            terminal.add_change(Change::CursorPosition {
                x: Position::Absolute(0),
                y: Position::Absolute(field.size + 1),
            });

            if !sapper.is_alive {
                terminal.add_change("\r\nSorry, but you've taken the wrong step. Game over, press Esc to exit.");
            }

            if field.is_cleaned {
                terminal.add_change("\r\nWell done! You've found the all mines! Press Esc to exit.");
            }

            terminal.flush().unwrap();
            update_screen = false;
        }

        match terminal.terminal().poll_input(None) {
            Ok(None) => {}
            Ok(Some(input)) => {
                match input {
                    InputEvent::Key(KeyEvent {key: KeyCode::Escape, ..}) => {
                        break
                    }
                    _ => {
                        if sapper.is_alive && !field.is_cleaned {
                            match input {
                                InputEvent::Key(KeyEvent {key: KeyCode::UpArrow, ..}) => {
                                    sapper._move(0, -1, &field);
                                    update_screen = true;
                                }
                                InputEvent::Key(KeyEvent {key: KeyCode::DownArrow, ..}) => {
                                    sapper._move(0, 1, &field);
                                    update_screen = true;
                                }
                                InputEvent::Key(KeyEvent {key: KeyCode::LeftArrow, ..}) => {
                                    sapper._move(-1, 0, &field);
                                    update_screen = true;
                                }
                                InputEvent::Key(KeyEvent {key: KeyCode::RightArrow, ..}) => {
                                    sapper._move(1, 0, &field);
                                    update_screen = true;
                                }
                                InputEvent::Key(KeyEvent {key: KeyCode::Char('m'), ..}) => {
                                    field.cells[sapper.position].toggle_mark();
                                    update_screen = true;
                                    field.update_is_cleaned();
                                }
                                InputEvent::Key(KeyEvent {key: KeyCode::Char(' '), ..}) => {
                                    if field.cells[sapper.position].state != CellState::Marked {
                                        if field.discover(sapper.position) {
                                            field.update_is_cleaned();
                                        } else {
                                            sapper.is_alive = false;
                                        }

                                        update_screen = true;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            Err(error) => {
                println!("{:?}", error);
                break;
            }
        }
    }

    terminal.add_change(Change::ClearScreen(ColorAttribute::Default));
    terminal.flush().unwrap();
}
