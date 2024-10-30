mod area;
mod collision;
mod constant_velocity;
mod minigames;
mod mouse;
mod player;
mod random;
mod resource;
mod toggleable;

use bevy::app::AppExit;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy_framepace::*;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;

use std::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            ShapePlugin,
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0),
            // RapierDebugRenderPlugin::default(),
            FramepacePlugin {},
        ))
        .add_systems(Startup, (setup_board, player::setup_player, setup_camera))
        .add_systems(
            Update,
            (
                keyboard_input,
                update_camera,
                player::player_move,
                constant_velocity::constant_velocity_system,
                resource::grab_resources,
                resource::release_resources,
                minigames::common::engage_button_update,
                minigames::button::update,
                minigames::tree::update,
                minigames::ball_breaker::unselected_paddle_update,
                minigames::primordial_ocean::update,
                mouse::update_mouse_state,
                mouse::follow_mouse_update,
            )
                .chain(),
        )
        .add_systems(
            FixedUpdate,
            (
                minigames::tree::fixed_update,
                minigames::ball_breaker::hit_block_fixed_update,
                minigames::ball_breaker::ingest_resource_fixed_update,
                resource::teleport_distant_loose_resources,
                resource::combine_loose_resources,
            ),
        )
        .insert_resource(mouse::MouseState::new(1.0))
        .insert_resource(Time::<Fixed>::from_hz(20.0))
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
        .insert_resource(random::Random::new(42))
        .insert_resource(minigames::Engaged { game: None })
        .run();
}

fn setup_board(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut random: ResMut<random::Random>,
) {
    minigames::button::spawn(
        &mut commands,
        Transform::from_xyz(400.0, 400.0, 0.0),
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

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        camera: Camera { ..default() },
        ..default()
    });
}

const MIN_ZOOM: f32 = 0.2;
const MAX_ZOOM: f32 = 3.0;

fn update_camera(
    camera_controller: ResMut<CameraController>,
    time: Res<Time>,
    engaged: Res<minigames::Engaged>,
    mut evr_scroll: EventReader<MouseWheel>,
    mut camera_query: Query<
        (&mut Transform, &mut OrthographicProjection),
        (With<Camera2d>, Without<player::Player>),
    >,
    player_query: Query<&Transform, (With<player::Player>, Without<Camera2d>)>,
    minigame_query: Query<
        &Transform,
        (
            With<minigames::Minigame>,
            Without<player::Player>,
            Without<Camera2d>,
        ),
    >,
) {
    let Ok(camera) = camera_query.get_single_mut() else {
        return;
    };
    let (mut camera_transform, mut camera_projection) = camera;

    let Ok(player) = player_query.get_single() else {
        return;
    };

    // focused on minigame
    if let Some(minigame) = engaged.game {
        let minigame_transform = minigame_query.get(minigame).unwrap();
        let Vec3 { x, y, .. } = minigame_transform.translation;
        let direction = Vec3::new(x, y, camera_transform.translation.z);
        camera_transform.translation = camera_transform
            .translation
            .lerp(direction, time.delta_seconds() * 2.0);
        camera_projection.scale = 1.0;
        return;
    }

    // focused on player

    let Vec3 { x, y, .. } = player.translation;
    let direction = Vec3::new(x, y, camera_transform.translation.z);

    // Applies a smooth effect to camera movement using interpolation between
    // the camera position and the player position on the x and y axes.
    // Here we use the in-game time, to get the elapsed time (in seconds)
    // since the previous update. This avoids jittery movement when tracking
    // the player.
    if (player.translation - camera_transform.translation).length_squared()
        > camera_controller.dead_zone_squared
    {
        camera_transform.translation = camera_transform
            .translation
            .lerp(direction, time.delta_seconds() * 2.0);
    }

    // adjust zoom
    for ev in evr_scroll.read() {
        if camera_projection.scale <= MIN_ZOOM && ev.y > 0.0 {
            continue;
        }
        if camera_projection.scale >= MAX_ZOOM && ev.y < 0.0 {
            continue;
        }
        camera_projection.scale -= ev.y * 0.1;
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

#[derive(Resource)]
struct CameraController {
    pub dead_zone_squared: f32,
}
