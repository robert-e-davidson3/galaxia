use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Debug, Copy, Clone, Component)]
pub struct ConstantSpeed {
    pub speed: f32,
}

pub fn constant_velocity_system(
    mut query: Query<(&ConstantSpeed, &mut Velocity)>,
) {
    for (speed, mut velocity) in query.iter_mut() {
        if speed.speed == 0.0 {
            velocity.linvel = Vec2::ZERO;
        } else {
            velocity.linvel = velocity.linvel.normalize() * speed.speed;
        }
    }
}
