use std::*;

use bevy::ecs::prelude::*;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::area::*;
use crate::collision::*;
use crate::resource::*;

#[derive(Bundle)]
pub struct PlayerBundle {
    pub player: Player,
    pub area: CircularArea,
    pub shape: ShapeBundle,
    pub fill: Fill,
    pub stroke: Stroke,
    pub collider: Collider,
    pub rigid_body: RigidBody,
    pub active_events: ActiveEvents,
    pub collision_groups: CollisionGroups,
    pub external_impulse: ExternalImpulse,
    pub damping: Damping,
    pub velocity: Velocity,
}

impl PlayerBundle {
    pub fn new() -> Self {
        let area = CircularArea { radius: 25.0 };
        Self {
            player: Player,
            area,
            shape: ShapeBundle {
                path: GeometryBuilder::build_as(&shapes::Circle {
                    radius: area.radius,
                    ..default()
                }),
                ..default()
            },
            fill: Fill::color(Color::srgb(0.625, 0.94, 0.91)),
            stroke: Stroke::new(Color::BLACK, 1.0),
            collider: area.into(),
            rigid_body: RigidBody::Dynamic,
            active_events: ActiveEvents::COLLISION_EVENTS,
            collision_groups: CollisionGroups::new(
                PLAYER_GROUP,
                player_filter(),
            ),
            external_impulse: default(),
            damping: Damping {
                linear_damping: 4.0,
                angular_damping: 4.0,
            },
            velocity: default(),
        }
    }
}

#[derive(Debug, Component)]
pub struct Player;

pub fn setup_player(mut commands: Commands) {
    commands.spawn(PlayerBundle::new());
}

pub fn player_move(
    mut commands: Commands,
    mut player_query: Query<(Entity, &mut ExternalImpulse), With<Player>>,
    stickiness_query: Query<Entity, (With<Sticky>, With<Player>)>,
    kb_input: Res<ButtonInput<KeyCode>>,
) {
    for (player_entity, mut external_impulse) in player_query.iter_mut() {
        if kb_input.just_released(KeyCode::Space) {
            if stickiness_query.get(player_entity).is_ok() {
                println!("Player is no longer sticky");
                commands.entity(player_entity).remove::<Sticky>();
            } else {
                println!("Player is now sticky");
                commands.entity(player_entity).insert(Sticky);
            }
        }

        let mut impulse = Vec2::ZERO;
        let mut torque = 0.0;
        if kb_input.pressed(KeyCode::KeyW) {
            impulse.y += 1.0;
        }
        if kb_input.pressed(KeyCode::KeyS) {
            impulse.y -= 1.0;
        }
        if kb_input.pressed(KeyCode::KeyA) {
            impulse.x -= 1.0;
        }
        if kb_input.pressed(KeyCode::KeyD) {
            impulse.x += 1.0;
        }
        if kb_input.pressed(KeyCode::KeyQ) {
            torque = 1.0;
        }
        if kb_input.pressed(KeyCode::KeyE) {
            torque = -1.0;
        }
        if impulse != Vec2::ZERO {
            impulse = impulse.normalize() * 45000.0;
            if kb_input.pressed(KeyCode::ShiftLeft) {
                impulse *= 3.0;
            }
            if kb_input.pressed(KeyCode::ControlLeft) {
                impulse *= 0.1;
            }
            external_impulse.impulse = impulse;
        }
        if torque != 0.0 {
            external_impulse.torque_impulse = torque * 200000.0;
        }
    }
}
