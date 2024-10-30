mod area;
mod camera;
mod collision;
mod constant_velocity;
mod minigames;
mod mouse;
mod player;
mod random;
mod resource;
mod toggleable;

use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_framepace::*;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            ShapePlugin,
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0),
            // RapierDebugRenderPlugin::default(),
            FramepacePlugin {},
        ))
        .add_systems(
            Startup,
            (setup_board, player::setup_player, camera::setup_camera),
        )
        .add_systems(
            Update,
            (
                exit_system,
                camera::update_camera,
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
        .insert_resource(camera::CameraController {
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

fn exit_system(
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
