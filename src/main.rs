use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::sprite::*;
use bevy_prototype_lyon::prelude::*;
// use bevy_rapier2d::prelude::*;
use std::collections::*;
use std::*;

fn main() {
    App::new()
        .add_plugins((
            //
            DefaultPlugins,
            ShapePlugin,
        ))
        .add_systems(Startup, (setup_board, setup_player, setup_camera))
        .add_systems(
            Update,
            (
                //
                keyboard_input,
                update_camera,
                player_move,
                button_mini_game::update,
            ),
        )
        // .add_systems(FixedUpdate, ())
        // Run engine code 10x per second, not at render rate.
        .insert_resource(Time::<Fixed>::from_seconds(0.1))
        .insert_resource(CameraController {
            dead_zone_squared: 1000.0,
        })
        .run();
}

fn setup_board(mut commands: Commands) {
    button_mini_game::spawn(
        &mut commands,
        &EtherLocation(Vec2::new(0.0, 0.0)),
        &button_mini_game::ButtonMiniGame { ..default() },
    );
}

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

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct Clickable;

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct EtherLocation(Vec2);

#[derive(Debug, Default, Bundle)]
pub struct PlayerBundle {
    pub player: Player,
    pub location: EtherLocation,
}

#[derive(Debug, Default, Component)]
pub struct Player {
    pub resources: HashMap<String, f32>,
}

pub struct LooseResourceBundle {
    pub resource: LooseResource,
    pub location: EtherLocation,
}

#[derive(Debug, Default, Component)]
#[component(storage = "SparseSet")]
pub struct LooseResource {
    pub resource: String,
    pub amount: f32,
}

#[derive(Debug, Default, Component)]
pub struct Area {
    pub width: f32,
    pub height: f32,
}

pub mod button_mini_game {
    use super::*;

    #[derive(Debug, Default, Bundle)]
    pub struct ButtonMiniGameBundle {
        pub mini_game: ButtonMiniGame,
        pub location: EtherLocation,
        pub area: Area,
    }

    #[derive(Debug, Default, Clone, Component)]
    pub struct ButtonMiniGame {
        pub count: u64,
    }

    pub fn spawn(
        commands: &mut Commands,
        location: &EtherLocation,
        frozen: &ButtonMiniGame,
    ) {
        commands
            .spawn((
                ButtonMiniGameBundle {
                    mini_game: frozen.clone(),
                    location: location.clone(),
                    area: Area {
                        width: 200.0,
                        height: 220.0,
                    },
                },
                SpatialBundle {
                    transform: Transform::from_xyz(
                        location.0.x,
                        location.0.y,
                        0.0,
                    ),
                    ..default()
                },
            ))
            .with_children(|parent| {
                let _background = parent.spawn(SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgb(0.9, 0.9, 0.9),
                        custom_size: Some(Vec2::new(200.0, 220.0)),
                        ..default()
                    },
                    transform: Transform::from_xyz(0.0, 0.0, -1.0),
                    ..default()
                });
                let text = parent
                    .spawn(Text2dBundle {
                        text: Text::from_section(
                            format!("Clicks: {}", frozen.count),
                            TextStyle {
                                font_size: 24.0,
                                color: Color::BLACK,
                                ..default()
                            },
                        ),
                        transform: Transform::from_xyz(0.0, 100.0, 0.0),
                        ..default()
                    })
                    .id();

                let _button = parent.spawn((
                    ClickMeButton {
                        game: parent.parent_entity(),
                        text,
                    },
                    Clickable,
                    Area {
                        width: 180.0,
                        height: 180.0,
                    },
                    ShapeBundle {
                        path: GeometryBuilder::build_as(&shapes::Circle {
                            radius: 90.0,
                            ..default()
                        }),
                        spatial: SpatialBundle {
                            transform: Transform::from_xyz(0.0, -18.0, 0.0),
                            ..default()
                        },
                        ..default()
                    },
                    Fill::color(Color::srgb(0.8, 0.1, 0.1)),
                    Stroke::new(Color::BLACK, 2.0),
                ));
            });
    }

    #[derive(Debug, Component)]
    pub struct ClickMeButton {
        pub game: Entity,
        pub text: Entity,
    }

    pub fn update(
        clickable_query: Query<
            (&ClickMeButton, &Transform, &Area),
            With<Clickable>,
        >,
        camera_query: Query<(&Camera, &GlobalTransform)>,
        windows: Query<&Window>,
        mouse_button_input: Res<ButtonInput<MouseButton>>,
        mut button_minigames_query: Query<&mut ButtonMiniGame>,
        mut text_query: Query<&mut Text>,
    ) {
        // TODO: https://bevy-cheatbook.github.io/programming/run-conditions.html
        if !mouse_button_input.just_pressed(MouseButton::Left) {
            return;
        }
        if button_minigames_query.iter().count() == 0 {
            return;
        }

        let (camera, camera_transform) = camera_query.single();
        let window = windows.single();

        if let Some(world_position) = window
            .cursor_position()
            .and_then(|cursor| {
                camera.viewport_to_world(camera_transform, cursor)
            })
            .map(|ray| ray.origin.truncate())
        {
            for (button, transform, area) in clickable_query.iter() {
                let min_x = transform.translation.x - area.width / 2.0;
                let max_x = transform.translation.x + area.width / 2.0;
                let min_y = transform.translation.y - area.height / 2.0;
                let max_y = transform.translation.y + area.height / 2.0;
                if world_position.x >= min_x
                    && world_position.x <= max_x
                    && world_position.y >= min_y
                    && world_position.y <= max_y
                {
                    let mut minigame =
                        button_minigames_query.get_mut(button.game).unwrap();
                    minigame.count += 1;
                    let mut text = text_query.get_mut(button.text).unwrap();
                    text.sections[0].value =
                        format!("Clicks: {}", minigame.count);
                }
            }
        }
    }
}
