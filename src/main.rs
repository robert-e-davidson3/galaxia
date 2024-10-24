use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::sprite::*;
use bevy_framepace::*;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;
use std::collections::*;
use std::*;

mod area;
use area::*;
mod mouse;
use mouse::*;
mod toggleable;
use toggleable::*;
mod resource;
use resource::*;
mod random;
use random::*;

mod minigames;
use minigames::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            ShapePlugin,
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0),
            // RapierDebugRenderPlugin::default(),
            FramepacePlugin {},
        ))
        .add_systems(Startup, (setup_board, setup_player, setup_camera))
        .add_systems(
            Update,
            (
                keyboard_input,
                update_camera,
                player_move,
                constant_velocity_system,
                grab_resources,
                release_resources,
                engage_button_update,
                minigames::button::update,
                minigames::tree::update,
                minigames::ball_breaker::unselected_paddle_update,
                minigames::primordial_ocean::update,
                update_mouse_state,
                follow_mouse_update,
            )
                .chain(),
        )
        .add_systems(FixedUpdate, minigames::tree::fixed_update)
        .insert_resource(MouseState::new(1.0))
        .insert_resource(Time::<Fixed>::from_seconds(5.0))
        .insert_resource(CameraController {
            dead_zone_squared: 1000.0,
        })
        .insert_resource(RapierConfiguration {
            gravity: Vec2::ZERO,
            physics_pipeline_active: true,
            query_pipeline_active: true,
            timestep_mode: TimestepMode::Variable {
                max_dt: 1.0 / 60.0,
                time_scale: 1.0,
                substeps: 1,
            },
            scaled_shape_subdivision: 10,
            force_update_from_transform_changes: false,
        })
        .insert_resource(FramepaceSettings {
            // limiter: Limiter::from_framerate(10.0),
            ..default()
        })
        .insert_resource(Random::new(42))
        .insert_resource(Engaged { game: None })
        .run();
}

fn setup_board(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut random: ResMut<Random>,
) {
    minigames::button::spawn(
        &mut commands,
        Transform::from_xyz(0.0, 0.0, 0.0),
        &minigames::button::ButtonMinigame { ..default() },
    );
    minigames::tree::spawn(
        &mut commands,
        &asset_server,
        Transform::from_xyz(400.0, 0.0, 0.0),
        &minigames::tree::TreeMinigame { ..default() },
    );
    minigames::ball_breaker::spawn(
        &mut commands,
        &asset_server,
        &mut random,
        Transform::from_xyz(-400.0, -400.0, 0.0),
        &minigames::ball_breaker::BallBreakerMinigame { ..default() },
    );
    minigames::primordial_ocean::spawn(
        &mut commands,
        Transform::from_xyz(0.0, 400.0, 0.0),
        &minigames::primordial_ocean::PrimordialOceanMinigame { ..default() },
    );
}

fn setup_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let area = CircularArea { radius: 25.0 };
    let _player = commands
        .spawn((
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
            ExternalImpulse::default(),
            Damping {
                linear_damping: 4.0,
                angular_damping: 4.0,
            },
            Velocity::default(),
        ))
        .id();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        camera: Camera { ..default() },
        ..default()
    });
}

fn update_camera(
    mut camera: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
    player: Query<&Transform, (With<Player>, Without<Camera2d>)>,
    camera_controller: ResMut<CameraController>,
    time: Res<Time>,
    engaged: Res<Engaged>,
    minigame_query: Query<
        &Transform,
        (With<Minigame>, Without<Player>, Without<Camera2d>),
    >,
) {
    let Ok(mut camera) = camera.get_single_mut() else {
        return;
    };

    let Ok(player) = player.get_single() else {
        return;
    };

    if let Some(minigame) = engaged.game {
        let minigame_transform = minigame_query.get(minigame).unwrap();
        let Vec3 { x, y, .. } = minigame_transform.translation;
        let direction = Vec3::new(x, y, camera.translation.z);
        camera.translation = camera
            .translation
            .lerp(direction, time.delta_seconds() * 2.0);
        return;
    }

    let Vec3 { x, y, .. } = player.translation;
    let direction = Vec3::new(x, y, camera.translation.z);

    // Applies a smooth effect to camera movement using interpolation between
    // the camera position and the player position on the x and y axes.
    // Here we use the in-game time, to get the elapsed time (in seconds)
    // since the previous update. This avoids jittery movement when tracking
    // the player.
    if (player.translation - camera.translation).length_squared()
        > camera_controller.dead_zone_squared
    {
        camera.translation = camera
            .translation
            .lerp(direction, time.delta_seconds() * 2.0);
    }
}

fn player_move(
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

fn keyboard_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    if keys.get_pressed().len() == 0 {
        return;
    }

    if keys.just_pressed(KeyCode::Escape) {
        app_exit_events.send(AppExit::Success);
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

pub fn grab_resources(
    mut commands: Commands,
    rapier_context: Res<RapierContext>,
    player_query: Query<Entity, (With<Player>, With<Sticky>)>,
    loose_resources: Query<&LooseResource, Without<Stuck>>,
    mut collision_events: EventReader<CollisionEvent>,
) {
    let Ok(player) = player_query.get_single() else {
        return;
    };

    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(entity1, entity2, _) => {
                let other: Entity;
                if *entity1 == player {
                    other = *entity2;
                } else if *entity2 == player {
                    other = *entity1;
                } else {
                    continue;
                }
                let Ok(resource) = loose_resources.get(other) else {
                    continue;
                };
                let Some(contact_pair) =
                    rapier_context.contact_pair(player, other)
                else {
                    continue;
                };
                let Some(manifold) = contact_pair.manifold(0) else {
                    continue;
                };
                let contact_point = manifold.local_n1();
                let direction = contact_point.normalize();
                let attachment_position = direction * (25.0 + 10.0); // TODO player and resource radii

                // TODO stick resource to player on touched side
                println!("Player grabbed resource: {:?}", resource);
                let joint = FixedJointBuilder::new()
                    .local_anchor1(attachment_position)
                    .local_anchor2(Vec2::ZERO);
                commands
                    .entity(other)
                    .insert(ImpulseJoint::new(player, joint))
                    .insert(Stuck { player });
            }
            _ => {}
        }
    }
}

fn _collect_loose_resources(
    mut commands: Commands,
    mut player: Query<&mut Player>,
    loose_resources: Query<(Entity, &LooseResource)>,
) {
    for (entity, resource) in loose_resources.iter() {
        let Ok(mut player) = player.get_single_mut() else {
            return;
        };

        if let Some(amount) = player.resources.get_mut(&resource.resource) {
            *amount += resource.amount;
        } else {
            player
                .resources
                .insert(resource.resource.clone(), resource.amount);
        }

        commands.entity(entity).despawn();
    }
}

#[derive(Resource)]
struct CameraController {
    pub dead_zone_squared: f32,
    //pub dead_zone_delay: f32,
    //pub dead_zone_last_time: f64,
}

#[derive(Debug, Default, Component)]
pub struct Player {
    pub resources: HashMap<GalaxiaResource, f32>,
}

#[derive(Debug, Copy, Clone, Component)]
pub struct Stuck {
    pub player: Entity,
}

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct Sticky;

pub fn constant_velocity_system(
    mut query: Query<(&ConstantSpeed, &mut Velocity)>,
) {
    for (speed, mut velocity) in query.iter_mut() {
        velocity.linvel = velocity.linvel.normalize() * speed.speed;
    }
}
