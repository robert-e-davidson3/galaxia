use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

#[derive(Resource)]
pub struct CameraController {
    pub dead_zone_squared: f32,
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        camera: Camera { ..default() },
        ..default()
    });
}

const MIN_ZOOM: f32 = 0.2;
const MAX_ZOOM: f32 = 3.0;

pub fn update_camera(
    camera_controller: ResMut<CameraController>,
    time: Res<Time>,
    engaged: Res<crate::minigames::Engaged>,
    mut evr_scroll: EventReader<MouseWheel>,
    mut camera_query: Query<
        (&mut Transform, &mut OrthographicProjection),
        (With<Camera2d>, Without<crate::player::Player>),
    >,
    player_query: Query<
        &Transform,
        (With<crate::player::Player>, Without<Camera2d>),
    >,
    minigame_query: Query<
        &Transform,
        (
            With<crate::minigames::Minigame>,
            Without<crate::player::Player>,
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
