use rand::prelude::*; // TODO: Learn more about imports cause this one looks strange
use std::time::Duration;
use std::time::SystemTime;

pub fn is_chance(chance: f64) -> bool {
    return rand::thread_rng().gen::<f64>() < chance;
}

pub fn difference(n1: usize, n2: usize) -> usize {
    if n1 > n2 {
        return n1 - n2;
    } else {
        return n2 - n1;
    }
}

pub struct Timer {
    duration: Duration,
    target: SystemTime,
}

impl Timer {
    pub fn new(duration: Duration) -> Self {
        return Self {
            duration,
            target: SystemTime::now(),
        };
    }

    pub fn next(&mut self) {
        self.target = SystemTime::now() + self.duration;
    }

    pub fn next_if_is_done(&mut self) -> bool {
        let is_done = self.is_done();

        if is_done {
            self.next();
        }

        return is_done;
    }

    pub fn is_done(&mut self) -> bool {
        return self.target <= SystemTime::now();
    }
}
