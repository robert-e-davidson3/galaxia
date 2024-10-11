use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::sprite::*;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;
use std::collections::*;
use std::*;

fn main() {
    App::new()
        .add_plugins((
            //
            DefaultPlugins,
            ShapePlugin,
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0),
            // RapierDebugRenderPlugin::default(),
        ))
        .add_systems(Startup, (setup_board, setup_player, setup_camera))
        .add_systems(
            Update,
            (
                //
                keyboard_input,
                update_camera,
                player_move,
                button_minigame::update,
            ),
        )
        .add_systems(FixedUpdate, (collect_loose_resources,))
        // Gather resources once every five seconds.
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
        .run();
}

fn setup_board(mut commands: Commands) {
    button_minigame::spawn(
        &mut commands,
        &mut Transform::from_xyz(0.0, 0.0, 0.0),
        &button_minigame::ButtonMiniGame { ..default() },
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
                material: materials.add(Color::srgb(6.25, 9.4, 9.1)),
                transform: Transform::from_xyz(0.0, 250.0, 1.0),
                ..default()
            },
            area,
            Collider::from(area),
            RigidBody::Dynamic,
            AdditionalMassProperties::Mass(10.0),
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
    mut player_query: Query<&mut ExternalImpulse, With<Player>>,
    kb_input: Res<ButtonInput<KeyCode>>,
) {
    for mut external_impulse in player_query.iter_mut() {
        let mut impulse = Vec2::ZERO;
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
        if impulse == Vec2::ZERO {
            return;
        }
        impulse = impulse.normalize() * 45000.0;
        if kb_input.pressed(KeyCode::ShiftLeft) {
            impulse *= 5.0;
        }
        if kb_input.pressed(KeyCode::ControlLeft) {
            impulse *= 0.5;
        }
        external_impulse.impulse = impulse;
    }
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

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct RectangularArea {
    pub width: f32,
    pub height: f32,
}

impl From<RectangularArea> for Collider {
    fn from(area: RectangularArea) -> Self {
        Collider::cuboid(area.width / 2.0, area.height / 2.0)
    }
}

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct CircularArea {
    pub radius: f32,
}

impl From<CircularArea> for Collider {
    fn from(area: CircularArea) -> Self {
        Collider::ball(area.radius)
    }
}

impl From<CircularArea> for Circle {
    fn from(area: CircularArea) -> Self {
        Circle {
            radius: area.radius,
        }
    }
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

fn translate_to_world_position(
    window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Vec2> {
    window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
}

pub mod button_minigame {
    use super::*;

    #[derive(Debug, Default, Bundle)]
    pub struct ButtonMiniGameBundle {
        pub minigame: ButtonMiniGame,
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
        let area = RectangularArea {
            width: 200.0,
            height: 220.0,
        };
        commands
            .spawn((
                ButtonMiniGameBundle {
                    minigame: frozen.clone(),
                    area: area.clone(),
                },
                SpatialBundle {
                    transform: Transform::from_xyz(
                        transform.translation.x,
                        transform.translation.y,
                        0.0,
                    ),
                    ..default()
                },
                RigidBody::Fixed,
                Collider::from(area),
            ))
            .with_children(|parent| {
                let _background = parent.spawn(SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgb(0.9, 0.9, 0.9),
                        custom_size: Some(Vec2::new(area.width, area.height)),
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
        asset_server: Res<AssetServer>,
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

        if let Some(world_position) =
            translate_to_world_position(window, camera, camera_transform)
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
                    spawn_click(
                        &mut commands,
                        &asset_server,
                        Transform::from_xyz(
                            world_position.x + 100.0,
                            world_position.y,
                            0.0,
                        ),
                    );
                }
            }
        }
    }

    fn spawn_click(
        commands: &mut Commands,
        asset_server: &Res<AssetServer>,
        transform: Transform,
    ) {
        let area = CircularArea { radius: 10.0 };
        commands.spawn((
            LooseResource {
                resource: "click".to_string(),
                amount: 1.0,
            },
            area,
            SpriteBundle {
                texture: asset_server.load("slick_arrow-arrow.png"),
                transform,
                ..default()
            },
            RigidBody::Dynamic,
            Collider::from(area),
        ));
    }
}
