use bevy::ecs::component::Component;

#[derive(Debug, Copy, Clone, Component)]
pub struct Ready {
    pub since_time: f32,
}

impl Ready {
    pub fn new(since_time: f32) -> Self {
        Self { since_time }
    }
}
