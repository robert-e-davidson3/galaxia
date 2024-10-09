use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::sprite::*;
// use bevy_rapier2d::prelude::*;
use std::collections::*;
use std::*;

fn main() {
    App::new()
        .add_plugins((
            //
            DefaultPlugins,
        ))
        .add_systems(Startup, (setup_board, setup_player, setup_camera))
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
                button_mini_game::system,
            ),
        )
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
pub struct LooseResource {
    pub resource: String,
    pub amount: f32,
}

pub mod button_mini_game {
    use super::*;

    #[derive(Debug, Default, Bundle)]
    pub struct ButtonMiniGameBundle {
        pub mini_game: ButtonMiniGame,
        pub location: EtherLocation,
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
                },
                NodeBundle {
                    style: Style {
                        width: Val::Px(200.0),
                        height: Val::Px(220.0),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    background_color: Color::srgb(0.15, 0.15, 0.15).into(),
                    transform: Transform::from_xyz(
                        location.0.x,
                        location.0.y,
                        0.0,
                    ),
                    ..default()
                },
            ))
            .with_children(|parent| {
                let text = parent
                    .spawn(TextBundle::from_section(
                        format!("Clicks: {}", frozen.count),
                        TextStyle {
                            font_size: 24.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ))
                    .id();
                parent.spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: Color::srgb(0.8, 0.2, 0.2).into(),
                        ..default()
                    },
                    ClickMeButton {
                        game: parent.parent_entity(),
                        text,
                    },
                ));
            });
    }

    #[derive(Debug, Component)]
    pub struct ClickMeButton {
        pub game: Entity,
        pub text: Entity,
    }

    pub fn system(
        mut interaction_query: Query<
            (&Interaction, &ClickMeButton),
            (Changed<Interaction>, With<Button>),
        >,
        mut button_minigames_query: Query<&mut ButtonMiniGame>,
        mut text_query: Query<&mut Text>,
    ) {
        for (interaction, button) in interaction_query.iter_mut() {
            match interaction {
                Interaction::Pressed => {
                    let mut minigame =
                        button_minigames_query.get_mut(button.game).unwrap();
                    minigame.count += 1;
                    let mut text = text_query.get_mut(button.text).unwrap();
                    text.sections[0].value =
                        format!("Clicks: {}", minigame.count);
                }
                _ => {}
            }
        }
    }
}
