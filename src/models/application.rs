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

pub struct Application {
    field: Field,
    sapper: Sapper,
    is_running: bool,
    is_render_required: bool,
}

impl Application {
    pub fn new() -> Self {
        return Application {
            field: Field::new(),
            sapper: Sapper::new(),
            is_running: false,
            is_render_required: false,
        }
    }

    pub fn run(&mut self) {
        let mut terminal = BufferedTerminal::new(new_terminal(Capabilities::new_from_env().unwrap()).unwrap()).unwrap();

        terminal.terminal().set_raw_mode().unwrap();
        terminal.add_change(Change::CursorShape(CursorShape::Hidden));

        self.is_running = true;
        self.require_render();

        while self.is_running {
            if self.is_render_required {
                terminal.draw_from_screen(&self.field.render(&self.sapper), 0, 0);

                terminal.add_change(Change::CursorPosition {
                    x: Position::Absolute(0),
                    y: Position::Absolute(self.field.size + 1),
                });

                if !self.sapper.is_alive {
                    terminal.add_change("\r\nSorry, but you've taken the wrong step. Game over, press Esc to exit.");
                }

                if self.field.is_cleaned {
                    terminal.add_change("\r\nWell done! You've found the all mines! Press Esc to exit.");
                }

                terminal.flush().unwrap();
                self.is_render_required = false;
            }

            match terminal.terminal().poll_input(None) {
                Ok(None) => {}
                Ok(Some(input)) => {
                    match input {
                        InputEvent::Key(KeyEvent {key: KeyCode::Escape, ..}) => {
                            self.stop();
                        }
                        _ => {
                            if self.sapper.is_alive && !self.field.is_cleaned {
                                match input {
                                    InputEvent::Key(KeyEvent {key: KeyCode::UpArrow, ..}) => {
                                        self.sapper._move(0, -1, &self.field);
                                        self.require_render();
                                    }
                                    InputEvent::Key(KeyEvent {key: KeyCode::DownArrow, ..}) => {
                                        self.sapper._move(0, 1, &self.field);
                                        self.require_render();
                                    }
                                    InputEvent::Key(KeyEvent {key: KeyCode::LeftArrow, ..}) => {
                                        self.sapper._move(-1, 0, &self.field);
                                        self.require_render();
                                    }
                                    InputEvent::Key(KeyEvent {key: KeyCode::RightArrow, ..}) => {
                                        self.sapper._move(1, 0, &self.field);
                                        self.require_render();
                                    }
                                    InputEvent::Key(KeyEvent {key: KeyCode::Char('m'), ..}) => {
                                        self.field.cells[self.sapper.position].toggle_mark();
                                        self.field.update_is_cleaned();
                                        self.require_render();
                                    }
                                    InputEvent::Key(KeyEvent {key: KeyCode::Char(' '), ..}) => {
                                        if self.field.cells[self.sapper.position].state != CellState::Marked {
                                            if self.field.discover(self.sapper.position) {
                                                self.field.update_is_cleaned();
                                            } else {
                                                self.sapper.is_alive = false;
                                            }

                                            self.require_render();
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
                    self.stop();
                }
            }
        }

        terminal.add_change(Change::ClearScreen(ColorAttribute::Default));
        terminal.flush().unwrap();
    }

    pub fn stop(&mut self) {
        self.is_running = false;
    }

    pub fn require_render(&mut self) {
        self.is_render_required = true;
    }
}
