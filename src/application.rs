use crate::game::Game;
use crate::ui::button::Button;
use crate::ui::input_number::InputNumber;
use crate::ui::menu::Menu;
use crate::ui::page::Page;
use crate::ui::spacer::Spacer;
use crate::ui::text::Text;
use crate::ui::Event;
use std::time::Duration;
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
const RESET: &'static str = "Reset";
const HELP: &'static str = "Help";
const BACK: &'static str = "Back";
const QUIT: &'static str = "Quit";
const FIELD_SIZE: &'static str = "Field size   ";
const MINES_DENSITY: &'static str = "Mines density";
const BOTS: &'static str = "Bots         ";
const BOTS_REACTION: &'static str = "Bots reaction";

const DEFAULT_FILED_SIZE: usize = 24;
const DEFAULT_MINES_DENSITY: f64 = 0.2;
const DEFAULT_BOTS: u8 = 1;
const DEFAULT_BOTS_REACTION: f64 = 1.0;

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
        };
    }

    fn init_menu() -> Menu {
        let mut menu = Menu::new();

        let mut page_main = Page::new(MAIN);
        page_main.elements.push(Box::new(Button::new(CONTINUE, false)));
        page_main.elements.push(Box::new(Button::new(NEW_GAME, true)));
        page_main.elements.push(Box::new(Button::new(HELP, true)));
        page_main.elements.push(Box::new(Button::new(QUIT, true)));
        page_main.reset_cursor();
        menu.add(page_main);

        let mut new_game = Page::new(NEW_GAME);

        new_game.elements.push(Box::new(InputNumber::new(
            FIELD_SIZE,
            DEFAULT_FILED_SIZE as f64,
            1.0,
            32.0,
            1.0,
            None,
        )));

        new_game.elements.push(Box::new(InputNumber::new(
            MINES_DENSITY,
            DEFAULT_MINES_DENSITY,
            0.0,
            1.0,
            0.01,
            Some(&"The probability that a cell will have a mine. 0 - no mines, 1 - every cell will be mined."),
        )));

        new_game.elements.push(Box::new(InputNumber::new(
            BOTS,
            DEFAULT_BOTS as f64,
            0.0,
            254.0,
            1.0,
            Some(&"The number of rival bots who will try to sweep mines as you too."),
        )));

        new_game.elements.push(Box::new(InputNumber::new(
            BOTS_REACTION,
            DEFAULT_BOTS_REACTION,
            0.1,
            5.0,
            0.1,
            Some(&"The time in seconds for a bot to make a move."),
        )));

        new_game.elements.push(Box::new(Spacer::new()));
        new_game.elements.push(Box::new(Button::new(START, true)));
        new_game.elements.push(Box::new(Button::new(RESET, true)));
        new_game.elements.push(Box::new(Button::new(BACK, true)));
        new_game.reset_cursor();
        menu.add(new_game);

        let help_text = "\
            - Use the arrow keys to move around the field\r\n\
            - Press `M` to mark a cell\r\n\
            - Press `Space` to discover a cell\r\n\
            - Press `Escape` to switch between the game and menu\
            ".to_string();

        let mut help = Page::new(HELP);
        help.elements.push(Box::new(Text::new(help_text)));
        help.elements.push(Box::new(Spacer::new()));
        help.elements.push(Box::new(Button::new(BACK, true)));
        help.reset_cursor();
        menu.add(help);

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

            match terminal.terminal().poll_input(Some(Duration::from_secs_f64(0.1))) {
                Ok(input) => {
                    let mut do_break = false;

                    if let Some(InputEvent::Key(KeyEvent {
                        key: KeyCode::Escape,
                        ..
                    })) = input {
                        if !self.is_menu || (self.menu.is_on_base_page() && self.game.is_some()) {
                            self.toggle_menu();
                            do_break = true;
                        }
                    }

                    if let Some(InputEvent::Resized { .. }) = input {
                        self.screen_update = ScreenUpdate::Full;
                        do_break = true;
                    }

                    if !do_break {
                        if self.is_menu {
                            if let Some(input) = input {
                                self.menu.update(&input);

                                while let Some(ui_event) = self.menu.pop_event() {
                                    match ui_event {
                                        // TODO: Optimize str comparison
                                        Event::ButtonPressed(CONTINUE) => {
                                            self.toggle_menu();
                                        }
                                        Event::ButtonPressed(NEW_GAME) => {
                                            self.menu.open(NEW_GAME);
                                        }
                                        Event::ButtonPressed(START) => {
                                            self.start_new_game();
                                        }
                                        Event::ButtonPressed(RESET) => {
                                            self.reset_settings();
                                        }
                                        Event::ButtonPressed(BACK) => {
                                            self.menu.back();
                                        }
                                        Event::ButtonPressed(HELP) => {
                                            self.menu.open(HELP);
                                        }
                                        Event::ButtonPressed(QUIT) => {
                                            self.stop();
                                        }
                                        Event::PageChanged => {
                                            self.set_screen_update(ScreenUpdate::Partial);
                                        }
                                        Event::MenuChanged => {
                                            self.set_screen_update(ScreenUpdate::Full);
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        } else {
                            self.game.as_mut().unwrap().update(input.as_ref());
                            self.set_screen_update(ScreenUpdate::Partial);
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
        let mut bots = DEFAULT_BOTS;
        let mut bots_reaction = DEFAULT_BOTS_REACTION;

        if let Some(page) = self.menu.fetch_page_mut(NEW_GAME) {
            if let Some(v) = page.fetch_input_number_mut(FIELD_SIZE) {
                field_size = v.value as usize;
            }

            if let Some(v) = page.fetch_input_number_mut(MINES_DENSITY) {
                mines_density = v.value;
            }

            if let Some(v) = page.fetch_input_number_mut(BOTS) {
                bots = v.value as u8;
            }

            if let Some(v) = page.fetch_input_number_mut(BOTS_REACTION) {
                bots_reaction = v.value;
            }
        }

        if let Some(page) = self.menu.fetch_page_mut(MAIN) {
            if let Some(button) = page.fetch_button_mut(CONTINUE) {
                button.is_active = true;
            }
        }

        self.game = Some(Game::new(field_size, mines_density, bots, bots_reaction));
        self.menu.back();
        self.toggle_menu();
    }

    fn reset_settings(&mut self) {
        if let Some(page) = self.menu.fetch_page_mut(NEW_GAME) {
            if let Some(v) = page.fetch_input_number_mut(FIELD_SIZE) {
                v.value = DEFAULT_FILED_SIZE as f64;
            }

            if let Some(v) = page.fetch_input_number_mut(MINES_DENSITY) {
                v.value = DEFAULT_MINES_DENSITY as f64;
            }

            if let Some(v) = page.fetch_input_number_mut(BOTS) {
                v.value = DEFAULT_BOTS as f64;
            }

            if let Some(v) = page.fetch_input_number_mut(BOTS_REACTION) {
                v.value = DEFAULT_BOTS_REACTION as f64;
            }

            self.set_screen_update(ScreenUpdate::Partial);
        }
    }

    fn toggle_menu(&mut self) {
        if !self.is_menu || self.game.is_some() {
            self.is_menu = !self.is_menu;

            if self.is_menu {
                if let Some(page) = self.menu.get_page_current_mut() {
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
