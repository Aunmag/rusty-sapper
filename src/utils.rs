use rand::prelude::*;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::time::Duration;
use std::time::SystemTime;
const IS_LOG_ENABLED: bool = true;

pub fn is_chance(chance: f64) -> bool {
    return rand::thread_rng().gen::<f64>() < chance;
}

pub const fn difference(n1: u16, n2: u16) -> u16 {
    if n1 > n2 {
        return n1 - n2;
    } else {
        return n2 - n1;
    }
}

#[allow(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub const fn f64_to_u8_saturating_floor(value: f64) -> u8 {
    return value as u8;
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

    pub fn is_done(&self) -> bool {
        return self.target <= SystemTime::now();
    }
}

pub fn log(data: &str) {
    if IS_LOG_ENABLED {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open("log.txt")
            .expect("Failed to initialize log file")
            .write_all(format!("{}\r\n", data).as_bytes())
            .expect("Failed to write into log");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f64_to_u8_saturating_floor() {
        assert_eq!(f64_to_u8_saturating_floor(100.0), 100, "Basic");
        assert_eq!(f64_to_u8_saturating_floor(100.9), 100, "Rounding");

        assert_eq!(f64_to_u8_saturating_floor(0.0), 0, "Min");
        assert_eq!(f64_to_u8_saturating_floor(255.0), 255, "Max");

        assert_eq!(f64_to_u8_saturating_floor(-1.0), 0, "Less than min");
        assert_eq!(f64_to_u8_saturating_floor(256.0), 255, "Greater than max");

        assert_eq!(
            f64_to_u8_saturating_floor(f64::NEG_INFINITY),
            0,
            "Negative infinity"
        );

        assert_eq!(
            f64_to_u8_saturating_floor(f64::INFINITY),
            255,
            "Positive infinity"
        );

        assert_eq!(f64_to_u8_saturating_floor(f64::NAN), 0, "Not a number");
    }
}
