use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::sprite::*;
use bevy_rapier2d::prelude::*;
use std::collections::*;
use std::*;

fn main() {
    App::new()
        .add_plugins((
            //
            DefaultPlugins,
        ))
        .add_systems(
            Startup,
            (
                //
                setup_player,
                setup_camera,
            ),
        )
        .add_systems(
            Update,
            (
                //
                keyboard_input,
                update_camera,
                player_move,
            ),
        )
        .add_systems(
            FixedUpdate,
            (
                //
                foobar,
            ),
        )
        // Run engine code 10x per second, not at render rate.
        .insert_resource(Time::<Fixed>::from_seconds(0.1))
        .insert_resource(CameraController {
            dead_zone_squared: 1000.0,
        })
        .run();
}

fn foobar() {}

fn setup_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let _player = commands
        .spawn((
            PlayerBundle {
                player: Player { ..default() },
                location: EtherLocation { ..default() },
            },
            MaterialMesh2dBundle {
                mesh: meshes.add(Circle::new(25.0)).into(),
                material: materials.add(Color::srgb(6.25, 9.4, 9.1)),
                transform: Transform::from_xyz(0.0, 250.0, 1.0),
                ..default()
            },
            RigidBody::Dynamic,
            Collider::ball(25.0),
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
    camera_controller: ResMut<CameraController>,
    player: Query<&Transform, (With<Player>, Without<Camera2d>)>,
    time: Res<Time>,
) {
    let Ok(mut camera) = camera.get_single_mut() else {
        return;
    };

    let Ok(player) = player.get_single() else {
        return;
    };

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
    mut player: Query<(&mut Transform, &mut EtherLocation), With<Player>>,
    time: Res<Time>,
    kb_input: Res<ButtonInput<KeyCode>>,
) {
    let Ok(player) = player.get_single_mut() else {
        return;
    };
    let (mut transform, mut location) = player;
    let mut direction = Vec2::ZERO;
    if kb_input.pressed(KeyCode::KeyW) {
        direction.y += 1.;
    }

    if kb_input.pressed(KeyCode::KeyS) {
        direction.y -= 1.;
    }

    if kb_input.pressed(KeyCode::KeyA) {
        direction.x -= 1.;
    }

    if kb_input.pressed(KeyCode::KeyD) {
        direction.x += 1.;
    }

    let move_delta = direction.normalize_or_zero()
        * 150.0
        * time.delta_seconds()
        * if kb_input.pressed(KeyCode::ShiftLeft) {
            5.0
        } else {
            1.0
        };
    if move_delta == Vec2::ZERO {
        return;
    }

    transform.translation += move_delta.extend(0.);

    location.0.x = transform.translation.x;
    location.0.y = transform.translation.y;
}

fn keyboard_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    if keys.get_pressed().len() == 0 {
        return;
    }

    // println!("key pressed: {:?}", keys);

    if keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::KeyQ) {
        app_exit_events.send(AppExit::Success);
    }
}

#[derive(Resource)]
struct CameraController {
    pub dead_zone_squared: f32,
    //pub dead_zone_delay: f32,
    //pub dead_zone_last_time: f64,
}

#[derive(Debug, Default, Component)]
struct EtherLocation(Vec2);

#[derive(Debug, Default, Bundle)]
struct PlayerBundle {
    pub player: Player,
    pub location: EtherLocation,
}

#[derive(Debug, Default, Component)]
struct Player {
    pub resources: HashMap<String, f32>,
}
