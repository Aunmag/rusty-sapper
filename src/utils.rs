use rand::prelude::*; // TODO: Learn more about imports cause this one looks strange

pub fn is_chance(chance: f64) -> bool {
    return rand::thread_rng().gen::<f64>() < chance;
}
