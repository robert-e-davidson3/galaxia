use bevy::prelude::*;

#[derive(Debug, Copy, Clone, Component)]
pub struct Toggleable {
    pub active: bool,
}

impl Toggleable {
    pub fn new() -> Self {
        Self { active: false }
    }

    pub fn toggle(&mut self) {
        self.active = !self.active;
    }
}
