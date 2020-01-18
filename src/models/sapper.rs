use crate::models::field::Field;
use std::io::stdin;
use std::io::stdout;
use std::io::Write;

pub struct Sapper {
    pub position: usize,
    pub is_alive: bool,
    pub is_admin: bool,
}

impl Sapper {

    pub fn new() -> Sapper {
        return Sapper {
            position: 0,
            is_alive: true,
            is_admin: false,
        };
    }

    pub fn request_input(&mut self, field: &Field) -> Option<SapperAction> {
        // TODO: Simplify

        print!("Press a key then hit \"Enter\": ");

        let mut s = String::new();
        let _ = stdout().flush();
        stdin().read_line(&mut s).expect("Did not enter a correct string");

        if let Some('\n') = s.chars().next_back() {
            s.pop();
        }

        if let Some('\r') = s.chars().next_back() {
            s.pop();
        }

        let mut result = Option::None;

        match s.as_str() {
            "w" => self._move(0, -1, field),
            "s" => self._move(0, 1, field),
            "a" => self._move(-1, 0, field),
            "d" => self._move(1, 0, field),
            " " => result = Option::Some(SapperAction::Discover),
            "m" => result = Option::Some(SapperAction::Mark),
            "q" => result = Option::Some(SapperAction::Quit),
            _ => (),
        }

        return result;
    }

    pub fn _move(&mut self, x: i32, y: i32, field: &Field) {
        if let Option::Some(position) = field.move_position(self.position, x, y) {
            self.position = position;
        }
    }

}

pub enum SapperAction {
    Discover,
    Mark,
    Quit,
}
