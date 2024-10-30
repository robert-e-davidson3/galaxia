use bevy::prelude::*;
use wyrand::WyRand;

#[derive(Resource)]
pub struct Random {
    rng: WyRand,
}

impl Random {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: WyRand::new(seed),
        }
    }

    pub fn next(&mut self) -> u64 {
        self.rng.rand()
    }
}
