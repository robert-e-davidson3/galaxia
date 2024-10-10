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
        .add_systems(FixedUpdate, (collect_loose_resources,))
        // Gather resources once every five seconds.
        .insert_resource(Time::<Fixed>::from_seconds(5.0))
        .insert_resource(CameraController {
            dead_zone_squared: 1000.0,
        })
        .run();
}

fn setup_board(mut commands: Commands) {
    button_mini_game::spawn(
        &mut commands,
        &mut Transform::from_xyz(0.0, 0.0, 0.0),
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
            Player { ..default() },
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
    mut player_query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
    kb_input: Res<ButtonInput<KeyCode>>,
) {
    let Ok(player) = player_query.get_single_mut() else {
        return;
    };
    let mut transform = player;
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

fn collect_loose_resources(
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

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct Clickable;

#[derive(Debug, Default, Component)]
pub struct Player {
    pub resources: HashMap<String, f32>,
}

#[derive(Debug, Bundle)]
pub struct LooseResourceBundle {
    pub resource: LooseResource,
    pub transform: Transform,
}

#[derive(Debug, Default, Component)]
#[component(storage = "SparseSet")]
pub struct LooseResource {
    pub resource: String,
    pub amount: f32,
}

#[derive(Debug, Default, Component)]
pub struct RectangularArea {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Default, Component)]
pub struct CircularArea {
    pub radius: f32,
}

fn is_click_in_rectangle(
    click_position: Vec2,
    rectangle_center: Vec2,
    rectangle_size: Vec2,
) -> bool {
    let min_x = rectangle_center.x - rectangle_size.x / 2.0;
    let max_x = rectangle_center.x + rectangle_size.x / 2.0;
    let min_y = rectangle_center.y - rectangle_size.y / 2.0;
    let max_y = rectangle_center.y + rectangle_size.y / 2.0;

    click_position.x >= min_x
        && click_position.x <= max_x
        && click_position.y >= min_y
        && click_position.y <= max_y
}

fn is_click_in_circle(
    click_position: Vec2,
    circle_center: Vec2,
    circle_radius: f32,
) -> bool {
    let distance_squared = click_position.distance_squared(circle_center);
    distance_squared <= circle_radius * circle_radius
}

pub mod button_mini_game {
    use super::*;

    #[derive(Debug, Default, Bundle)]
    pub struct ButtonMiniGameBundle {
        pub mini_game: ButtonMiniGame,
        pub area: RectangularArea,
    }

    #[derive(Debug, Default, Clone, Component)]
    pub struct ButtonMiniGame {
        pub count: u64,
    }

    pub fn spawn(
        commands: &mut Commands,
        transform: &Transform,
        frozen: &ButtonMiniGame,
    ) {
        commands
            .spawn((
                ButtonMiniGameBundle {
                    mini_game: frozen.clone(),
                    area: RectangularArea {
                        width: 200.0,
                        height: 220.0,
                    },
                },
                SpatialBundle {
                    transform: Transform::from_xyz(
                        transform.translation.x,
                        transform.translation.y,
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
                    CircularArea { radius: 90.0 },
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
        mut commands: Commands,
        clickable_query: Query<
            (&ClickMeButton, &Transform, &CircularArea),
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
                let button_center = transform.translation.truncate();

                if is_click_in_circle(
                    world_position,
                    button_center,
                    area.radius,
                ) {
                    let mut minigame =
                        button_minigames_query.get_mut(button.game).unwrap();
                    minigame.count += 1;
                    let mut text = text_query.get_mut(button.text).unwrap();
                    text.sections[0].value =
                        format!("Clicks: {}", minigame.count);

                    commands.spawn((
                        LooseResource {
                            resource: "click".to_string(),
                            amount: 1.0,
                        },
                        // TODO spawn on random edge of minigame
                        draw_click(Transform::from_xyz(
                            world_position.x + 100.0,
                            world_position.y,
                            0.0,
                        )),
                    ));
                }
            }
        }
    }

    fn draw_click(transform: Transform) -> impl Bundle {
        let pointer_shape = shapes::Polygon {
            points: vec![
                Vec2::new(0.0, 0.0),   // Tip of the pointer
                Vec2::new(0.0, 18.0),  // Left point
                Vec2::new(6.0, 15.0),  // Left top of shaft
                Vec2::new(10.0, 20.0), // Left bottom of shaft
                Vec2::new(13.0, 20.0), // Right bottom of shaft
                Vec2::new(12.0, 14.0), // Right top of shaft
                Vec2::new(20.0, 18.0), // Right point
            ],
            closed: true,
        };

        (
            ShapeBundle {
                path: GeometryBuilder::build_as(&pointer_shape),
                spatial: SpatialBundle {
                    transform,
                    ..default()
                },
                ..default()
            },
            Fill::color(Color::WHITE),
            Stroke::new(Color::BLACK, 1.0),
        )
    }
}
