use crate::models::game::Game;
use crate::models::ui::button::Button;
use crate::models::ui::element::ElementEvent;
use crate::models::ui::input_number::InputNumber;
use crate::models::ui::spacer::Spacer;
use crate::models::ui::menu::Menu;
use crate::models::ui::page::Page;
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

const MAIN: &'static str = "Main menu";
const CONTINUE: &'static str = "Continue";
const NEW_GAME: &'static str = "New game";
const START: &'static str = "Start";
const BACK: &'static str = "Back";
const QUIT: &'static str = "Quit";
const FIELD_SIZE: &'static str = "Field size   ";
const MINES_DENSITY: &'static str = "Mines density";

const DEFAULT_FILED_SIZE: usize = 24;
const DEFAULT_MINES_DENSITY: f64 = 0.2;

#[derive(PartialEq)]
pub enum ScreenUpdate {
    None,
    Partial,
    Full,
}

pub struct Application {
    menu: Menu,
    game: Option<Game>,
    is_running: bool,
    is_menu: bool,
    screen_update: ScreenUpdate,
}

impl Application {
    pub fn new() -> Self {
        return Application {
            menu: Self::init_menu(),
            game: None,
            is_running: false,
            is_menu: true,
            screen_update: ScreenUpdate::Full,
        }
    }

    fn init_menu() -> Menu {
        let mut menu = Menu::new();

        let mut page_main = Page::new(MAIN);
        page_main.elements.push(Box::new(Button::new(CONTINUE, false)));
        page_main.elements.push(Box::new(Button::new(NEW_GAME, true)));
        page_main.elements.push(Box::new(Button::new(QUIT, true)));
        page_main.reset_cursor();
        menu.add(page_main);

        let mut new_game = Page::new(NEW_GAME);
        new_game.elements.push(Box::new(InputNumber::new(FIELD_SIZE, DEFAULT_FILED_SIZE as f64, 8.0, 32.0, 1.0)));
        new_game.elements.push(Box::new(InputNumber::new(MINES_DENSITY, DEFAULT_MINES_DENSITY, 0.0, 1.0, 0.01)));
        new_game.elements.push(Box::new(Spacer::new()));
        new_game.elements.push(Box::new(Button::new(START, true)));
        new_game.elements.push(Box::new(Button::new(BACK, true)));
        new_game.reset_cursor();
        menu.add(new_game);

        return menu;
    }

    pub fn run(&mut self) {
        let mut terminal = BufferedTerminal::new(new_terminal(Capabilities::new_from_env().unwrap()).unwrap()).unwrap();

        terminal.terminal().set_raw_mode().unwrap();
        terminal.add_change(Change::CursorShape(CursorShape::Hidden));

        self.is_running = true;

        while self.is_running {
            if self.screen_update == ScreenUpdate::Full {
                terminal.add_change(Change::ClearScreen(ColorAttribute::Default));
            }

            if self.screen_update == ScreenUpdate::Partial || self.screen_update == ScreenUpdate::Full {
                if self.is_menu {
                    terminal.draw_from_screen(&self.menu.render(), 0, 0);
                } else {
                    if let Some(game) = &self.game {
                        terminal.draw_from_screen(&game.render(), 0, 0);
                    }
                }

                terminal.add_change(Change::CursorPosition {
                    x: Position::Absolute(0),
                    y: Position::Absolute(0),
                });

                terminal.flush().unwrap();
            }

            self.screen_update = ScreenUpdate::None;

            match terminal.terminal().poll_input(None) {
                Ok(None) => {}
                Ok(Some(input)) => {
                    let mut is_menu_toggling = false;

                    if let InputEvent::Key(KeyEvent {key: KeyCode::Escape, ..}) = input {
                        is_menu_toggling = !self.is_menu || (self.menu.is_on_base_page() && self.game.is_some());
                    }

                    if is_menu_toggling {
                        self.toggle_menu();
                    } else {
                        if self.is_menu {
                            self.menu.update(&input);

                            let mut ui_events = self.menu.pull_events();

                            while !ui_events.is_empty() {
                                for ui_event in &ui_events {
                                    match ui_event {
                                        // TODO: Optimize str comparison
                                        ElementEvent::ButtonPressed(CONTINUE) => {
                                            self.toggle_menu();
                                        }
                                        ElementEvent::ButtonPressed(NEW_GAME) => {
                                            self.menu.open(NEW_GAME);
                                        }
                                        ElementEvent::ButtonPressed(START) => {
                                            self.start_new_game();
                                        }
                                        ElementEvent::ButtonPressed(BACK) => {
                                            self.menu.back();
                                        }
                                        ElementEvent::ButtonPressed(QUIT) => {
                                            self.stop();
                                        }
                                        ElementEvent::PageChanged => {
                                            self.set_screen_update(ScreenUpdate::Partial);
                                        }
                                        ElementEvent::MenuChanged => {
                                            self.set_screen_update(ScreenUpdate::Full);
                                        }
                                        _ => {}
                                    }
                                }

                                ui_events = self.menu.pull_events();
                            }
                        } else {
                            if self.game.as_mut().unwrap().update(&input) {
                                self.set_screen_update(ScreenUpdate::Partial);
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

    fn start_new_game(&mut self) {
        let mut field_size = DEFAULT_FILED_SIZE;
        let mut mines_density = DEFAULT_MINES_DENSITY;

        if let Some(page) = self.menu.fetch_page_mut(NEW_GAME) {
            if let Some(v) = page.fetch_input_number_mut(FIELD_SIZE) {
                field_size = v.value as usize;
            }

            if let Some(v) = page.fetch_input_number_mut(MINES_DENSITY) {
                mines_density = v.value;
            }
        }

        if let Some(page) = self.menu.fetch_page_mut(MAIN) {
            if let Some(button) = page.fetch_button_mut(CONTINUE) {
                button.is_active = true;
            }
        }

        self.game = Some(Game::new(field_size, mines_density));
        self.menu.back();
        self.toggle_menu();
    }

    fn toggle_menu(&mut self) {
        if !self.is_menu || self.game.is_some() {
            self.is_menu = !self.is_menu;

            if self.is_menu {
                if let Some(page) = self.menu.get_page_current() {
                    page.reset_cursor();
                }
            }

            self.set_screen_update(ScreenUpdate::Full);
        }
    }

    fn stop(&mut self) {
        self.is_running = false;
    }

    fn set_screen_update(&mut self, screen_update: ScreenUpdate) {
        if self.screen_update != ScreenUpdate::Full {
            self.screen_update = screen_update;
        }
    }
}
