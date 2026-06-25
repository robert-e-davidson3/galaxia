use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

use crate::entities::*;

#[derive(Resource)]
pub struct CameraController {
    pub dead_zone_squared: f32,
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

const MIN_ZOOM: f32 = 0.2;
const MAX_ZOOM: f32 = 3.0;

pub fn update_camera(
    camera_controller: ResMut<CameraController>,
    time: Res<Time>,
    engaged: Res<Engaged>,
    minigames: Res<MinigamesResource>,
    mut evr_scroll: MessageReader<MouseWheel>,
    mut camera_query: Query<
        (&mut Transform, &mut Projection),
        (With<Camera2d>, Without<player::Player>),
    >,
    player_query: Query<&Transform, (With<player::Player>, Without<Camera2d>)>,
    minigame_query: Query<
        &Transform,
        (With<Minigame>, Without<player::Player>, Without<Camera2d>),
    >,
) {
    let Ok(camera) = camera_query.single_mut() else {
        return;
    };
    let (mut camera_transform, mut projection) = camera;
    let Projection::Orthographic(camera_projection) = projection.as_mut() else {
        return;
    };

    let Ok(player) = player_query.single() else {
        return;
    };

    // focused on minigame
    if let Some(id) = engaged.game {
        // Resolve the engaged minigame's id to its live entity (it may be
        // mid-respawn from a levelup, or gone). If it resolves, follow it;
        // otherwise fall through to following the player.
        let focus = minigames
            .entity(id)
            .and_then(|e| minigame_query.get(e).ok());
        if let Some(minigame_transform) = focus {
            let direction = minigame_transform
                .translation
                .with_z(camera_transform.translation.z);
            camera_transform.translation = camera_transform
                .translation
                .lerp(direction, time.delta_secs() * 2.0);
            camera_projection.scale = 1.0;
            return;
        }
    }

    // focused on player

    let direction =
        player.translation.with_z(camera_transform.translation.z);

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
            .lerp(direction, time.delta_secs() * 2.0);
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
