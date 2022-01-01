use crate::game::Game;
use crate::ui::button::Button;
use crate::ui::input_number::InputNumber;
use crate::ui::input_text::InputText;
use crate::ui::menu::Menu;
use crate::ui::page::Page;
use crate::ui::spacer::Spacer;
use crate::ui::text::Text;
use crate::ui::Event;
use crate::net::NetHandler;
use crate::net::client::Client;
use crate::net::server::Server;
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
use crate::field::Field;
use crate::sapper::Sapper;
use crate::sapper::SapperBehavior;
use futures::executor::block_on;
use crate::utils;

const MAIN: &str = "Main menu";
const CONTINUE: &str = "Continue";
const NEW_GAME: &str = "New game";
const START: &str = "Start";
const RESET: &str = "Reset";
const JOIN_GAME: &str = "Join game";
const JOIN: &str = "Join";
const HELP: &str = "Help";
const BACK: &str = "Back";
const QUIT: &str = "Quit";
const FIELD_SIZE: &str = "Field size   ";
const MINES_DENSITY: &str = "Mines density";
const BOTS: &str = "Bots         ";
const BOTS_REACTION: &str = "Bots reaction";
const SERVER_IP: &str = "Server IP    ";
const SERVER_PORT: &str = "Server port  ";
const ERROR: &str = "Error";
const DISCONNECTED: &str = "Disconnected";

const DEFAULT_FILED_SIZE: u8 = 8;
const DEFAULT_MINES_DENSITY: f64 = 0.2;
const DEFAULT_BOTS: u8 = 0;
const DEFAULT_BOTS_REACTION: f64 = 1.0;
const DEFAULT_SERVER_IP: &str = "127.0.0.1";
const DEFAULT_SERVER_PORT: &str = "6000";

#[derive(PartialEq)]
pub enum ScreenUpdate {
    None,
    Partial,
    Full,
}

pub struct Application {
    menu: Menu,
    server: Option<Server>,
    client: Option<Client>,
    is_running: bool,
    is_menu: bool,
    screen_update: ScreenUpdate,
}

impl Application {
    pub fn new() -> Self {
        return Self {
            menu: Self::init_menu(),
            server: None,
            client: None,
            is_running: false,
            is_menu: true,
            screen_update: ScreenUpdate::Full,
        };
    }

    fn init_menu() -> Menu {
        let mut menu = Menu::new();

        {
            let mut page_main = Page::new(MAIN);
            page_main.elements.push(Box::new(Button::new(CONTINUE, false)));
            page_main.elements.push(Box::new(Button::new(NEW_GAME, true)));
            page_main.elements.push(Box::new(Button::new(JOIN_GAME, true)));
            page_main.elements.push(Box::new(Button::new(HELP, true)));
            page_main.elements.push(Box::new(Button::new(QUIT, true)));
            page_main.reset_cursor();
            menu.add(page_main);
        }

        {
            let mut new_game = Page::new(NEW_GAME);

            new_game.elements.push(Box::new(InputNumber::new(
                FIELD_SIZE,
                f64::from(DEFAULT_FILED_SIZE),
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
                Some("The probability that a cell will have a mine. 0 - no mines, 1 - every cell will be mined."),
            )));

            new_game.elements.push(Box::new(InputNumber::new(
                BOTS,
                f64::from(DEFAULT_BOTS),
                0.0,
                254.0,
                1.0,
                Some("The number of rival bots who will try to sweep mines as you too."),
            )));

            new_game.elements.push(Box::new(InputNumber::new(
                BOTS_REACTION,
                DEFAULT_BOTS_REACTION,
                0.1,
                5.0,
                0.1,
                Some("The time in seconds for a bot to make a move."),
            )));

            let mut server_ip = InputText::new(
                SERVER_IP,
                Some("IPv4 or IPv6 address."), // TODO: Verify
            );
            server_ip.value = DEFAULT_SERVER_IP.to_owned();

            let mut server_port = InputText::new(SERVER_PORT, None);
            server_port.value = DEFAULT_SERVER_PORT.to_owned();

            new_game.elements.push(Box::new(server_ip));
            new_game.elements.push(Box::new(server_port));
            new_game.elements.push(Box::new(Spacer::new()));
            new_game.elements.push(Box::new(Button::new(START, true)));
            new_game.elements.push(Box::new(Button::new(RESET, true)));
            new_game.elements.push(Box::new(Button::new(BACK, true)));
            new_game.reset_cursor();
            menu.add(new_game);
        }

        {
            let help_text = "\
                - Use the arrow keys to move around the field\r\n\
                - Press `M` to mark a cell\r\n\
                - Press `Space` to discover a cell\r\n\
                - Press `Escape` to switch between the game and menu\
                ".to_owned();

            let mut help = Page::new(HELP);
            help.elements.push(Box::new(Text::new(help_text)));
            help.elements.push(Box::new(Spacer::new()));
            help.elements.push(Box::new(Button::new(BACK, true)));
            help.reset_cursor();
            menu.add(help);
        }

        {
            let mut join = Page::new(JOIN_GAME);

            let mut server_ip = InputText::new(
                SERVER_IP,
                Some("IPv4 or IPv6 address."), // TODO: Verify
            );
            server_ip.value = DEFAULT_SERVER_IP.to_owned();

            let mut server_port = InputText::new(SERVER_PORT, None);
            server_port.value = DEFAULT_SERVER_PORT.to_owned();

            join.elements.push(Box::new(server_ip));
            join.elements.push(Box::new(server_port));
            join.elements.push(Box::new(Spacer::new()));
            join.elements.push(Box::new(Button::new(JOIN, true)));
            join.elements.push(Box::new(Button::new(BACK, true)));
            join.reset_cursor();
            menu.add(join);
        }

        return menu;
    }

    #[allow(clippy::too_many_lines)] // TODO: Resolve later
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
                } else if let Some(client) = self.client.as_ref() {
                    terminal.draw_from_screen(
                        &client.game.render(),
                        0,
                        0,
                    );
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
                        if !self.is_menu || (self.menu.is_on_base_page() && self.client.is_some()) {
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
                                            self.start_new_game_safely(true);
                                        }
                                        Event::ButtonPressed(JOIN) => {
                                            self.start_new_game_safely(false);
                                        }
                                        Event::ButtonPressed(RESET) => {
                                            self.reset_settings();
                                        }
                                        Event::ButtonPressed(BACK) => {
                                            self.menu.back();
                                        }
                                        Event::ButtonPressed(JOIN_GAME) => {
                                            self.menu.open(JOIN_GAME);
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
                            let mut do_stop = false;

                            if let Some(server) = self.server.as_mut() {
                                server.update(None);

                                if let Some(error) = server.error.take() {
                                    self.menu.show_message(error, ERROR, BACK);
                                    do_stop = true;
                                }
                            }

                            if let Some(client) = self.client.as_mut() {
                                client.update(input.as_ref());

                                if let Some(error) = client.error.take() {
                                    self.menu.show_message(error, DISCONNECTED, BACK);
                                    do_stop = true;
                                }
                            }

                            if do_stop {
                                self.stop_game();
                            }

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

    fn start_new_game(&mut self, is_host: bool) -> Result<(), String> {
        self.stop_game();

        let mut field_size = DEFAULT_FILED_SIZE;
        let mut mines_density = DEFAULT_MINES_DENSITY;
        let mut bots = DEFAULT_BOTS;
        let mut bots_reaction = DEFAULT_BOTS_REACTION;
        let mut address = "".to_owned();

        if let Some(page) = self.menu.fetch_page_mut(NEW_GAME) {
            if let Some(v) = page.fetch_input_number_mut(FIELD_SIZE) {
                field_size = utils::f64_to_u8_saturating_floor(v.value);
            }

            if let Some(v) = page.fetch_input_number_mut(MINES_DENSITY) {
                mines_density = v.value;
            }

            if let Some(v) = page.fetch_input_number_mut(BOTS) {
                bots = utils::f64_to_u8_saturating_floor(v.value);
            }

            if let Some(v) = page.fetch_input_number_mut(BOTS_REACTION) {
                bots_reaction = v.value;
            }
        }

        if let Some(page) = self.menu.get_page_current_mut() {
            if let Some(v) = page.fetch_input_text_mut(SERVER_IP) {
                address = v.value.clone();
            }
        }

        if let Some(page) = self.menu.get_page_current_mut() {
            if let Some(v) = page.fetch_input_text_mut(SERVER_PORT) {
                address = format!("{}:{}", address, v.value.clone());
            }
        }

        let address = address.parse()
            .map_err(|e| format!("{}", e))
            ?;

        if is_host {
            let field = Field::new(field_size, mines_density);

            let mut sappers = Vec::with_capacity(usize::from(bots) + 1);

            for i in 0..bots {
                sappers.push(Sapper::new(
                    i,
                    SapperBehavior::Bot,
                    field.generate_random_position(),
                    bots_reaction,
                ));
            }

            self.server = Some(block_on(Server::new(
                address,
                Game::new(field, sappers),
            ))?);
        }

        self.client = None;
        self.client = Some(block_on(Client::new(address))?);

        if let Some(page) = self.menu.fetch_page_mut(MAIN) {
            if let Some(button) = page.fetch_button_mut(CONTINUE) {
                button.is_active = true;
            }
        }

        return Ok(());
    }

    fn start_new_game_safely(&mut self, is_host: bool) {
        match self.start_new_game(is_host) {
            Ok(()) => {
                self.menu.back();
                self.toggle_menu();
            }
            Err(error) => {
                self.stop_game();
                self.menu.show_message(error, ERROR, BACK);
            }
        }
    }

    fn reset_settings(&mut self) {
        if let Some(page) = self.menu.fetch_page_mut(NEW_GAME) {
            if let Some(v) = page.fetch_input_number_mut(FIELD_SIZE) {
                v.value = f64::from(DEFAULT_FILED_SIZE);
            }

            if let Some(v) = page.fetch_input_number_mut(MINES_DENSITY) {
                v.value = DEFAULT_MINES_DENSITY;
            }

            if let Some(v) = page.fetch_input_number_mut(BOTS) {
                v.value = f64::from(DEFAULT_BOTS);
            }

            if let Some(v) = page.fetch_input_number_mut(BOTS_REACTION) {
                v.value = DEFAULT_BOTS_REACTION;
            }

            if let Some(v) = page.fetch_input_text_mut(SERVER_IP) {
                v.value = DEFAULT_SERVER_IP.to_owned();
            }

            if let Some(v) = page.fetch_input_text_mut(SERVER_PORT) {
                v.value = DEFAULT_SERVER_PORT.to_owned();
            }

            self.set_screen_update(ScreenUpdate::Partial);
        }
    }

    fn toggle_menu(&mut self) {
        if !self.is_menu || self.client.is_some() {
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

    fn stop_game(&mut self) {
        self.client = None;
        self.server = None;
        self.toggle_menu();
    }

    fn set_screen_update(&mut self, screen_update: ScreenUpdate) {
        if self.screen_update != ScreenUpdate::Full {
            self.screen_update = screen_update;
        }
    }
}
