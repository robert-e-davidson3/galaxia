use std::collections::HashMap;
use std::*;

use bevy::ecs::prelude::*;
use bevy::prelude::*;
use bevy::sprite::*;
use bevy_rapier2d::prelude::*;

use crate::area::*;
use crate::collision::*;
use crate::resource::*;

#[derive(Debug, Default, Component)]
pub struct Player {
    pub resources: HashMap<GalaxiaResource, f32>,
}

pub fn setup_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let area = CircularArea { radius: 25.0 };
    commands.spawn((
        Player { ..default() },
        MaterialMesh2dBundle {
            mesh: meshes.add(Circle::from(area)).into(),
            material: materials.add(Color::srgb(0.625, 0.94, 0.91)),
            transform: Transform::from_xyz(-200.0, -400.0, 1.0),
            ..default()
        },
        area,
        Collider::from(area),
        RigidBody::Dynamic,
        ActiveEvents::COLLISION_EVENTS,
        CollisionGroups::new(PLAYER_GROUP, player_filter()),
        ExternalImpulse::default(),
        Damping {
            linear_damping: 4.0,
            angular_damping: 4.0,
        },
        Velocity::default(),
    ));
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

pub fn release_resources(
    mut commands: Commands,
    loose_resource_query: Query<(Entity, &Stuck), With<LooseResource>>,
    player_query: Query<Entity, (With<Player>, Without<Sticky>)>,
) {
    for (stuck_entity, stuck) in loose_resource_query.iter() {
        let player_entity = stuck.player;
        if !player_query.contains(player_entity) {
            continue;
        }
        commands.entity(stuck_entity).remove::<ImpulseJoint>();
        commands.entity(stuck_entity).remove::<Stuck>();
    }
}
