use crate::models::cell::Cell;
use crate::models::field::Field;
use crate::utils::Timer;
use std::collections::HashSet;
use std::time::Duration;
use termwiz::input::InputEvent;
use termwiz::input::KeyCode;
use termwiz::input::KeyEvent;

const NAME_PLAYER: &'static str = "YOU";
const NAME_BOT: &'static str = "BOT";

pub enum SapperBehavior {
    Player,
    Bot,
}

pub struct Sapper {
    position: usize,
    is_alive: bool,
    behavior: SapperBehavior,
    marks: HashSet<usize>,
    timer: Timer,
    score: u16,
}

struct BotTask {
    position: usize,
    is_mined: bool,
}

impl Sapper {
    pub fn new(behavior: SapperBehavior, position: usize, reaction: f64) -> Sapper {
        return Sapper {
            position,
            is_alive: true,
            behavior,
            marks: HashSet::new(),
            timer: Timer::new(Duration::from_secs_f64(reaction)),
            score: 0,
        };
    }

    pub fn update(&mut self, field: &mut Field, input: Option<&InputEvent>) {
        self.remove_useless_marks(field); // TODO: Try to optimize

        if !self.is_alive {
            return;
        }

        match self.behavior {
            SapperBehavior::Player => {
                if let Some(input) = input {
                    self.update_as_player(field, input);
                }
            }
            SapperBehavior::Bot => {
                self.update_as_bot(field);
            }
        }
    }

    fn update_as_player(&mut self, field: &mut Field, input: &InputEvent) {
        match input {
            InputEvent::Key(KeyEvent {
                key: KeyCode::UpArrow,
                ..
            }) => {
                self.shift_position(0, -1, field);
            }
            InputEvent::Key(KeyEvent {
                key: KeyCode::DownArrow,
                ..
            }) => {
                self.shift_position(0, 1, field);
            }
            InputEvent::Key(KeyEvent {
                key: KeyCode::LeftArrow,
                ..
            }) => {
                self.shift_position(-1, 0, field);
            }
            InputEvent::Key(KeyEvent {
                key: KeyCode::RightArrow,
                ..
            }) => {
                self.shift_position(1, 0, field);
            }
            InputEvent::Key(KeyEvent {
                key: KeyCode::Char('m'),
                ..
            }) => {
                self.toggle_mark(field);
            }
            InputEvent::Key(KeyEvent {
                key: KeyCode::Char(' '),
                ..
            }) => {
                self.discover(field);
            }
            _ => {}
        }
    }

    fn update_as_bot(&mut self, field: &mut Field) {
        if self.timer.next_if_is_done() {
            if let Some(task) = self.find_task(field) {
                self.perform_task(&task, field);
            }
        }
    }

    fn find_task(&mut self, field: &mut Field) -> Option<BotTask> {
        let mut task = None;
        let mut task_distance = std::usize::MAX;

        for (cell_i, cell) in field.get_cells().iter().enumerate() {
            if !cell.is_discovered() {
                continue;
            }

            let mines_around = cell.mines_around.unwrap_or(0);
            let mut undiscovered = Vec::with_capacity(8);
            let mut mines_found = 0;

            for cell_near_i in field.around(cell_i, false) {
                let cell_near = &field.get_cells()[cell_near_i];

                if cell_near.is_exploded || self.has_marked(cell_near_i) {
                    mines_found += 1;
                } else if !cell_near.is_discovered() {
                    undiscovered.push(cell_near_i);
                }
            }

            let unmarked = mines_around.saturating_sub(mines_found);
            let is_mined = undiscovered.len() == unmarked as usize;

            if mines_around == mines_found || is_mined {
                for i in undiscovered.iter() {
                    let i = *i;
                    let distance_test = field.to_distance(self.position, i);

                    if task_distance > distance_test {
                        task_distance = distance_test;

                        task = Some(BotTask {
                            position: i,
                            is_mined,
                        });
                    }
                }
            }
        }

        return task;
    }

    fn perform_task(&mut self, task: &BotTask, field: &mut Field) {
        if self.position == task.position {
            if task.is_mined {
                self.toggle_mark(field);
            } else {
                self.discover(field);
            }
        } else {
            self.move_to(task.position, field);
        }
    }

    fn move_to(&mut self, target: usize, field: &Field) {
        let (x, y) = field.to_coordinate(self.position);
        let (target_x, target_y) = field.to_coordinate(target);
        let mut shift_x = 0;
        let mut shift_y = 0;

        if x < target_x {
            shift_x += 1;
        } else if x > target_x {
            shift_x -= 1;
        } else if y < target_y {
            shift_y += 1;
        } else if y > target_y {
            shift_y -= 1;
        }

        if shift_x != 0 || shift_y != 0 {
            self.shift_position(shift_x, shift_y, field);
        }
    }

    fn shift_position(&mut self, x: i32, y: i32, field: &Field) {
        if let Some(position) = field.move_position(self.position, x, y) {
            self.position = position;
        }
    }

    fn toggle_mark(&mut self, field: &Field) {
        let i = self.position;

        if !self.marks.remove(&i) {
            if field.get_cells().get(i).map(Cell::is_markable).unwrap_or(false) {
                self.marks.insert(i);
            }
        }
    }

    fn remove_useless_marks(&mut self, field: &Field) {
        self.marks = self.marks.iter().filter_map(|i| {
            let i = *i;

            if field.get_cells().get(i).map(Cell::is_markable).unwrap_or(false) {
                return Some(i);
            } else {
                return None;
            }
        }).collect();
    }

    fn discover(&mut self, field: &mut Field) {
        if !self.has_marked(self.position) {
            if field.discover(self.position) {
                self.score += 1;
            } else {
                self.is_alive = false;
            }
        }
    }

    pub fn has_marked(&self, position: usize) -> bool {
        return self.marks.contains(&position);
    }

    pub fn is_player(&self) -> bool {
        if let SapperBehavior::Player = self.behavior {
            return true;
        } else {
            return false;
        }
    }

    pub fn is_alive(&self) -> bool {
        return self.is_alive;
    }

    pub fn get_position(&self) -> usize {
        return self.position;
    }

    pub fn get_name(&self) -> &'static str {
        return match self.behavior {
            SapperBehavior::Player => NAME_PLAYER,
            SapperBehavior::Bot => NAME_BOT,
        };
    }

    pub fn get_marks_count(&self) -> usize {
        return self.marks.len();
    }

    pub fn get_score(&self) -> u16 {
        return self.score;
    }
}
