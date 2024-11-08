mod entities;
mod libs;

use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_framepace::{FramepacePlugin, FramepaceSettings};
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;

use entities::*;
use libs::*;

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
                exit_system,
                update_camera,
                player_move,
                constant_velocity_system,
                grab_items,
                release_items,
                engage_button_update,
                minigames::button::update,
                minigames::rune::pixel_update,
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
                minigames::button::levelup,
                minigames::rune::levelup,
                minigames::primordial_ocean::levelup,
                minigames::primordial_ocean::ingest_resource_fixed_update,
                minigames::rune::fixed_update,
                minigames::tree::fixed_update,
                minigames::ball_breaker::hit_block_fixed_update,
                minigames::ball_breaker::ingest_resource_fixed_update,
                item::teleport_distant_loose_items,
                item::combine_loose_items,
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
        .insert_resource(entities::minigame::Engaged { game: None })
        .init_resource::<image_gen::GeneratedImageAssets>()
        .run();
}

fn setup_board(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut random: ResMut<random::Random>,
) {
    entities::minigames::button::spawn(
        &mut commands,
        Transform::from_xyz(0.0, 400.0, 0.0),
        &entities::minigames::button::ButtonMinigame { ..default() },
    );
    entities::minigames::primordial_ocean::spawn(
        &mut commands,
        Transform::from_xyz(400.0, -300.0, 0.0),
        entities::minigames::primordial_ocean::PrimordialOceanMinigame::new(
            0.0,
        ),
    );
    entities::minigames::rune::spawn(
        &mut commands,
        Transform::from_xyz(-400.0, -300.0, 0.0),
        entities::minigames::rune::RuneMinigame::new(0),
    );
    // entities::minigames::tree::spawn(
    //     &mut commands,
    //     &asset_server,
    //     Transform::from_xyz(400.0, 0.0, 0.0),
    //     &entities::minigames::tree::TreeMinigame { ..default() },
    // );
    entities::minigames::ball_breaker::spawn(
        &mut commands,
        &asset_server,
        &mut random,
        Transform::from_xyz(400.0, 400.0, 0.0),
        &entities::minigames::ball_breaker::BallBreakerMinigame { ..default() },
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
