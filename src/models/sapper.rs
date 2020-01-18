use crate::models::field::Field;

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

    pub fn _move(&mut self, x: i32, y: i32, field: &Field) {
        if let Option::Some(position) = field.move_position(self.position, x, y) {
            self.position = position;
        }
    }

}
