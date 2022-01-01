use crate::cell::Cell;
use crate::event::EventData;
use crate::event::EventManager;
use crate::field::Field;
use crate::utils::Timer;
use std::collections::HashSet;
use std::time::Duration;
use std::convert::TryFrom;
use termwiz::input::InputEvent;
use termwiz::input::KeyCode;
use termwiz::input::KeyEvent;

const NAME_PLAYER: &str = "YOU";
const NAME_REMOTE: &str = "NET";
const NAME_BOT: &str = "BOT";

pub enum SapperBehavior {
    Player,
    Remote,
    Bot,
}

pub struct Sapper {
    id: u8,
    pub position: u16,
    pub is_alive: bool,
    pub behavior: SapperBehavior,
    marks: HashSet<u16>,
    timer: Timer,
    pub score: u16,
    events: EventManager,
}

struct BotTask {
    position: u16,
    is_mined: bool,
}

impl Sapper {
    pub fn new(id: u8, behavior: SapperBehavior, position: u16, reaction: f64) -> Self {
        return Self {
            id,
            position,
            is_alive: true,
            behavior,
            marks: HashSet::new(),
            timer: Timer::new(Duration::from_secs_f64(reaction)),
            score: 0,
            events: EventManager::new(),
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
            SapperBehavior::Remote => {}
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
        let mut task_distance = u16::MAX;

        for (i, cell) in field.get_cells().iter().enumerate() {
            let cell_position = match u16::try_from(i) {
                Ok(cell_position) => cell_position,
                Err(_) => {
                    // TODO: Log error
                    break;
                }
            };

            if !cell.is_discovered() {
                continue;
            }

            let mines_around = cell.mines_around.unwrap_or(0);
            let mut undiscovered = Vec::with_capacity(8);
            let mut mines_found = 0;

            for cell_near_position in field.around(cell_position, false) {
                if let Some(cell_near) = field.get_cell(cell_near_position) {
                    if cell_near.is_exploded || self.has_marked(cell_near_position) {
                        mines_found += 1;
                    } else if !cell_near.is_discovered() {
                        undiscovered.push(cell_near_position);
                    }
                }
            }

            let unmarked = mines_around.saturating_sub(mines_found);
            let is_mined = undiscovered.len() == usize::from(unmarked);

            if mines_around == mines_found || is_mined {
                for undiscovered_position in &undiscovered {
                    let distance_test = field.to_distance(self.position, *undiscovered_position);

                    if task_distance > distance_test {
                        task_distance = distance_test;

                        task = Some(BotTask {
                            position: *undiscovered_position,
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

    fn move_to(&mut self, target: u16, field: &Field) {
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
            self.events.fire(
                EventData::SapperMove {
                    id: self.id,
                    position: self.position,
                },
                None,
                None,
            );
        }
    }

    fn toggle_mark(&mut self, field: &Field) {
        let i = self.position;

        if !self.marks.remove(&i)
            && field
                .get_cell(i)
                .map_or(false, Cell::is_markable)
        {
            self.marks.insert(i);
        }
    }

    fn remove_useless_marks(&mut self, field: &Field) {
        self.marks = self.marks.iter().filter_map(|i| {
            let i = *i;

            if field.get_cell(i).map_or(false, Cell::is_markable) {
                return Some(i);
            } else {
                return None;
            }
        }).collect();
    }

    pub fn discover(&mut self, field: &mut Field) {
        let can_discover = field.get_cell(self.position)
            .map_or(false, |c| !c.is_discovered() && !c.is_exploded);

        if can_discover && !self.has_marked(self.position) {
            self.events.fire(
                EventData::SapperDiscover {
                    id: self.id,
                    position: self.position,
                },
                None,
                None,
            );
        }
    }

    pub fn has_marked(&self, position: u16) -> bool {
        return self.marks.contains(&position);
    }

    pub const fn is_player(&self) -> bool {
        if let SapperBehavior::Player = self.behavior {
            return true;
        } else {
            return false;
        }
    }

    pub const fn get_id(&self) -> u8 {
        return self.id;
    }

    pub const fn is_alive(&self) -> bool {
        return self.is_alive;
    }

    pub const fn get_position(&self) -> u16 {
        return self.position;
    }

    pub const fn get_name(&self) -> &'static str {
        return match self.behavior {
            SapperBehavior::Player => NAME_PLAYER,
            SapperBehavior::Remote => NAME_REMOTE,
            SapperBehavior::Bot => NAME_BOT,
        };
    }

    pub fn get_marks_count(&self) -> usize {
        return self.marks.len();
    }

    pub const fn get_score(&self) -> u16 {
        return self.score;
    }

    pub fn get_events_mut(&mut self) -> &mut EventManager {
        return &mut self.events;
    }
}
