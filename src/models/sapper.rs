use crate::models::field::Field;

pub struct Sapper {
    pub position: usize,
    pub is_alive: bool,
}

impl Sapper {
    pub fn new() -> Self {
        return Sapper {
            position: 0,
            is_alive: true,
        };
    }

    pub fn _move(&mut self, x: i32, y: i32, field: &Field) {
        if let Some(position) = field.move_position(self.position, x, y) {
            self.position = position;
        }
    }
}
