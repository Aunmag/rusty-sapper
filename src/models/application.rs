use crate::models::game::Game;
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
    game: Game,
    is_running: bool,
    is_render_required: bool,
}

impl Application {
    pub fn new() -> Self {
        return Application {
            game: Game::new(),
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
                terminal.draw_from_screen(&self.game.render(), 0, 0);
                terminal.add_change(Change::CursorPosition {
                    x: Position::Absolute(0),
                    y: Position::Absolute(self.game.field.size + 1),
                });

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
                            if self.game.update(&input) {
                                self.require_render();
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
