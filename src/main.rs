use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::sprite::*;
use bevy_framepace::*;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;
use std::collections::*;
use std::*;
use wyrand::WyRand;

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
                keyboard_input,
                update_camera,
                player_move,
                constant_velocity_system,
                button_minigame::update,
                tree_minigame::update,
            ),
        )
        .add_systems(
            FixedUpdate,
            (collect_loose_resources, tree_minigame::fixed_update),
        )
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
        .insert_resource(FramepaceSettings {
            // limiter: Limiter::from_framerate(10.0),
            ..default()
        })
        .insert_resource(Random {
            rng: WyRand::new(42),
        })
        .run();
}

#[derive(Resource)]
pub struct Random {
    rng: WyRand,
}

fn setup_board(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut random: ResMut<Random>,
) {
    button_minigame::spawn(
        &mut commands,
        Transform::from_xyz(0.0, 0.0, 0.0),
        &button_minigame::ButtonMinigame { ..default() },
    );
    tree_minigame::spawn(
        &mut commands,
        &asset_server,
        Transform::from_xyz(400.0, 0.0, 0.0),
        &tree_minigame::TreeMinigame { ..default() },
    );
    ball_breaker_minigame::spawn(
        &mut commands,
        &asset_server,
        &mut random,
        Transform::from_xyz(-400.0, -400.0, 0.0),
        &ball_breaker_minigame::BallBreakerMinigame { ..default() },
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
                material: materials.add(Color::srgb(0.625, 0.94, 0.91)),
                transform: Transform::from_xyz(-200.0, -400.0, 1.0),
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
        impulse = impulse.normalize() * 40000.0;
        if kb_input.pressed(KeyCode::ShiftLeft) {
            impulse *= 3.0;
        }
        if kb_input.pressed(KeyCode::ControlLeft) {
            impulse *= 0.1;
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
    pub resources: HashMap<GalaxiaResource, f32>,
}

#[derive(Debug, Bundle)]
pub struct LooseResourceBundle {
    pub resource: LooseResource,
    pub transform: Transform,
}

#[derive(Debug, Component)]
#[component(storage = "SparseSet")]
pub struct LooseResource {
    pub resource: GalaxiaResource,
    pub amount: f32,
}

pub enum ResourceKind {
    Abstract,
    Solid,
    Liquid,
    Gas,
    Mana,
    Energy,
    Heat,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum GalaxiaResource {
    // abstract
    ShortLeftClick,
    LongLeftClick,

    // solid
    Apple,
    Lemon,
    Lime,
    Mud,
    Dirt,
    Sandstone,
    Granite,
    Marble,
    Obsidian,
    Copper,
    Tin,
    Iron,
    Silver,
    Gold,
    Diamond,
    Amethyst,
    Moss,
    Unobtainium,

    // liquid
    SaltWater,
    FreshWater,
    // gas
    // mana
    // energy
    // heat
}

pub fn resource_to_kind(resource: GalaxiaResource) -> ResourceKind {
    match resource {
        // abstract
        GalaxiaResource::ShortLeftClick => ResourceKind::Abstract,
        GalaxiaResource::LongLeftClick => ResourceKind::Abstract,
        // solid
        GalaxiaResource::Apple => ResourceKind::Solid,
        GalaxiaResource::Lemon => ResourceKind::Solid,
        GalaxiaResource::Lime => ResourceKind::Solid,
        GalaxiaResource::Mud => ResourceKind::Solid,
        GalaxiaResource::Dirt => ResourceKind::Solid,
        GalaxiaResource::Sandstone => ResourceKind::Solid,
        GalaxiaResource::Granite => ResourceKind::Solid,
        GalaxiaResource::Marble => ResourceKind::Solid,
        GalaxiaResource::Obsidian => ResourceKind::Solid,
        GalaxiaResource::Copper => ResourceKind::Solid,
        GalaxiaResource::Tin => ResourceKind::Solid,
        GalaxiaResource::Iron => ResourceKind::Solid,
        GalaxiaResource::Silver => ResourceKind::Solid,
        GalaxiaResource::Gold => ResourceKind::Solid,
        GalaxiaResource::Diamond => ResourceKind::Solid,
        GalaxiaResource::Amethyst => ResourceKind::Solid,
        GalaxiaResource::Moss => ResourceKind::Solid,
        GalaxiaResource::Unobtainium => ResourceKind::Solid,
        // liquid
        GalaxiaResource::SaltWater => ResourceKind::Liquid,
        GalaxiaResource::FreshWater => ResourceKind::Liquid,
        // gas
        // mana
        // energy
        // heat
    }
}

pub fn resource_to_asset(resource: GalaxiaResource) -> String {
    match resource {
        // abstract
        GalaxiaResource::ShortLeftClick => {
            "abstract/short_left_click.png".to_string()
        }
        GalaxiaResource::LongLeftClick => {
            "abstract/long_left_click.png".to_string()
        }
        // solid
        GalaxiaResource::Apple => "solid/apple.png".to_string(),
        GalaxiaResource::Lemon => "solid/lemon.png".to_string(),
        GalaxiaResource::Lime => "solid/lime.png".to_string(),
        GalaxiaResource::Mud => "solid/mud.png".to_string(),
        GalaxiaResource::Dirt => "solid/dirt.png".to_string(),
        GalaxiaResource::Sandstone => "solid/sandstone.png".to_string(),
        GalaxiaResource::Granite => "solid/granite.png".to_string(),
        GalaxiaResource::Marble => "solid/marble.png".to_string(),
        GalaxiaResource::Obsidian => "solid/obsidian.png".to_string(),
        GalaxiaResource::Copper => "solid/copper.png".to_string(),
        GalaxiaResource::Tin => "solid/tin.png".to_string(),
        GalaxiaResource::Iron => "solid/iron.png".to_string(),
        GalaxiaResource::Silver => "solid/silver.png".to_string(),
        GalaxiaResource::Gold => "solid/gold.png".to_string(),
        GalaxiaResource::Diamond => "solid/diamond.png".to_string(),
        GalaxiaResource::Amethyst => "solid/amethyst.png".to_string(),
        GalaxiaResource::Moss => "solid/moss.png".to_string(),
        GalaxiaResource::Unobtainium => "solid/unobtainium.png".to_string(),
        // liquid
        GalaxiaResource::SaltWater => "liquid/salt_water.png".to_string(),
        GalaxiaResource::FreshWater => "liquid/fresh_water.png".to_string(),
        // gas
        // mana
        // energy
        // heat
    }
}

pub fn resource_to_name(resource: GalaxiaResource, full: bool) -> String {
    if full {
        match resource {
            // abstract
            GalaxiaResource::ShortLeftClick => "Short Left Click".to_string(),
            GalaxiaResource::LongLeftClick => "Long Left Click".to_string(),
            // solid
            GalaxiaResource::Apple => "Apple".to_string(),
            GalaxiaResource::Lemon => "Lemon".to_string(),
            GalaxiaResource::Lime => "Lime".to_string(),
            GalaxiaResource::Mud => "Mud".to_string(),
            GalaxiaResource::Dirt => "Dirt".to_string(),
            GalaxiaResource::Sandstone => "Sandstone".to_string(),
            GalaxiaResource::Granite => "Granite".to_string(),
            GalaxiaResource::Marble => "Marble".to_string(),
            GalaxiaResource::Obsidian => "Obsidian".to_string(),
            GalaxiaResource::Copper => "Copper".to_string(),
            GalaxiaResource::Tin => "Tin".to_string(),
            GalaxiaResource::Iron => "Iron".to_string(),
            GalaxiaResource::Silver => "Silver".to_string(),
            GalaxiaResource::Gold => "Gold".to_string(),
            GalaxiaResource::Diamond => "Diamond".to_string(),
            GalaxiaResource::Amethyst => "Amethyst".to_string(),
            GalaxiaResource::Moss => "Moss".to_string(),
            GalaxiaResource::Unobtainium => "Unobtainium".to_string(),
            // liquid
            GalaxiaResource::SaltWater => "Salt Water".to_string(),
            GalaxiaResource::FreshWater => "Fresh Water".to_string(),
            // gas
            // mana
            // energy
            // heat
        }
    } else {
        match resource {
            // abstract
            GalaxiaResource::ShortLeftClick => "Click".to_string(),
            GalaxiaResource::LongLeftClick => "Click".to_string(),
            // solid
            GalaxiaResource::Apple => "Fruit".to_string(),
            GalaxiaResource::Lemon => "Fruit".to_string(),
            GalaxiaResource::Lime => "Fruit".to_string(),
            GalaxiaResource::Mud => "Dirt".to_string(),
            GalaxiaResource::Dirt => "Dirt".to_string(),
            GalaxiaResource::Sandstone => "Stone".to_string(),
            GalaxiaResource::Granite => "Stone".to_string(),
            GalaxiaResource::Marble => "Stone".to_string(),
            GalaxiaResource::Obsidian => "Stone".to_string(),
            GalaxiaResource::Copper => "Metal".to_string(),
            GalaxiaResource::Tin => "Metal".to_string(),
            GalaxiaResource::Iron => "Metal".to_string(),
            GalaxiaResource::Silver => "Metal".to_string(),
            GalaxiaResource::Gold => "Metal".to_string(),
            GalaxiaResource::Diamond => "Gem".to_string(),
            GalaxiaResource::Amethyst => "Gem".to_string(),
            GalaxiaResource::Moss => "Plant".to_string(),
            GalaxiaResource::Unobtainium => "Metal".to_string(),
            // liquid
            GalaxiaResource::SaltWater => "Water".to_string(),
            GalaxiaResource::FreshWater => "Water".to_string(),
            // gas
            // mana
            // energy
            // heat
        }
    }
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

fn _is_click_in_rectangle(
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

pub fn spawn_bounding_rectangle(
    parent: &mut ChildBuilder,
    area: RectangularArea,
) {
    const WALL_THICKNESS: f32 = 1.0;
    const HALF_WALL_THICKNESS: f32 = WALL_THICKNESS / 2.0;
    let half_width: f32 = area.width / 2.0;
    let half_height: f32 = area.height / 2.0;
    parent
        .spawn((
            ShapeBundle {
                path: GeometryBuilder::build_as(&shapes::Rectangle {
                    extents: Vec2::new(area.width, area.height),
                    origin: RectangleOrigin::Center,
                }),
                ..Default::default()
            },
            Fill::color(Color::NONE),
            Stroke::new(Color::BLACK, WALL_THICKNESS),
        ))
        .with_children(|parent| {
            // top wall
            parent.spawn((
                TransformBundle::from(Transform::from_xyz(
                    0.0,
                    half_height,
                    0.0,
                )),
                Collider::cuboid(half_width, HALF_WALL_THICKNESS),
                RigidBody::Fixed,
            ));
            // bottom wall
            parent.spawn((
                TransformBundle::from(Transform::from_xyz(
                    0.0,
                    -half_height,
                    0.0,
                )),
                Collider::cuboid(half_width, HALF_WALL_THICKNESS),
                RigidBody::Fixed,
            ));
            // left wall
            parent.spawn((
                TransformBundle::from(Transform::from_xyz(
                    -half_width,
                    0.0,
                    0.0,
                )),
                Collider::cuboid(HALF_WALL_THICKNESS, half_height),
                RigidBody::Fixed,
            ));
            // right wall
            parent.spawn((
                TransformBundle::from(Transform::from_xyz(
                    half_width, 0.0, 0.0,
                )),
                Collider::cuboid(HALF_WALL_THICKNESS, half_height),
                RigidBody::Fixed,
            ));
        });
}

#[derive(Debug, Copy, Clone, Component)]
pub struct ConstantSpeed {
    pub speed: f32,
}

pub fn constant_velocity_system(
    mut query: Query<(&ConstantSpeed, &mut Velocity)>,
) {
    for (speed, mut velocity) in query.iter_mut() {
        velocity.linvel = velocity.linvel.normalize() * speed.speed;
    }
}

pub mod button_minigame {
    use super::*;

    pub const DESCRIPTION: &str = "Click the button, get clicks!";

    #[derive(Debug, Default, Bundle)]
    pub struct ButtonMinigameBundle {
        pub minigame: ButtonMinigame,
        pub area: RectangularArea,
    }

    #[derive(Debug, Default, Clone, Component)]
    pub struct ButtonMinigame {
        pub count: u64,
    }

    pub fn spawn(
        commands: &mut Commands,
        transform: Transform,
        frozen: &ButtonMinigame,
    ) {
        let area = RectangularArea {
            width: 200.0,
            height: 220.0,
        };
        commands
            .spawn((
                ButtonMinigameBundle {
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
            ))
            .with_children(|parent| {
                spawn_bounding_rectangle(parent, area);
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
            (&ClickMeButton, &GlobalTransform, &CircularArea),
            With<Clickable>,
        >,
        camera_query: Query<(&Camera, &GlobalTransform)>,
        windows: Query<&Window>,
        mouse_button_input: Res<ButtonInput<MouseButton>>,
        mut button_minigames_query: Query<&mut ButtonMinigame>,
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
            for (button, global_transform, area) in clickable_query.iter() {
                let button_center = global_transform.translation().truncate();

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
                resource: GalaxiaResource::ShortLeftClick,
                amount: 1.0,
            },
            area,
            SpriteBundle {
                texture: asset_server
                    .load(resource_to_asset(GalaxiaResource::ShortLeftClick)),
                transform,
                ..default()
            },
            RigidBody::Dynamic,
            Collider::from(area),
        ));
    }
}

pub mod tree_minigame {
    use super::*;

    pub const DESCRIPTION: &str = "Pick fruits from the tree!";

    #[derive(Debug, Default, Bundle)]
    pub struct TreeMinigameBundle {
        pub minigame: TreeMinigame,
        pub area: RectangularArea,
    }

    #[derive(Debug, Clone, Component)]
    pub struct TreeMinigame {
        pub fruit: GalaxiaResource,
        pub count: u32,
        pub lushness: f32,
        pub last_fruit_time: f32,
    }

    impl Default for TreeMinigame {
        fn default() -> Self {
            Self {
                fruit: GalaxiaResource::Apple,
                count: 0,
                lushness: 1.0,
                last_fruit_time: 0.0,
            }
        }
    }

    pub fn spawn(
        commands: &mut Commands,
        asset_server: &Res<AssetServer>,
        transform: Transform,
        frozen: &TreeMinigame,
    ) {
        let area = RectangularArea {
            width: 300.0,
            height: 300.0,
        };
        commands
            .spawn((
                TreeMinigameBundle {
                    minigame: frozen.clone(),
                    area: area.clone(),
                },
                SpriteBundle {
                    texture: asset_server
                        .load("oak-tree-white-background-300x300.png"),
                    sprite: Sprite {
                        color: Color::srgba(1.0, 1.0, 1.0, 1.0),
                        custom_size: Some(Vec2::new(area.width, area.height)),
                        ..default()
                    },
                    transform: transform.clone(),
                    ..default()
                },
            ))
            .with_children(|parent| {
                spawn_bounding_rectangle(parent, area);
            });
    }

    // When a fruit is clicked, replace it with a fruit resource.
    pub fn update(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        clickable_query: Query<
            (Entity, &UnpickedFruit, &GlobalTransform, &CircularArea),
            With<Clickable>,
        >,
        camera_query: Query<(&Camera, &GlobalTransform)>,
        windows: Query<&Window>,
        mouse_button_input: Res<ButtonInput<MouseButton>>,
        mut tree_minigames_query: Query<&mut TreeMinigame>,
    ) {
        if !mouse_button_input.just_pressed(MouseButton::Left) {
            return;
        }

        let (camera, camera_transform) = camera_query.single();
        let window = windows.single();

        if let Some(world_position) =
            translate_to_world_position(window, camera, camera_transform)
        {
            for (entity, fruit, transform, area) in clickable_query.iter() {
                let fruit_center = transform.translation().truncate();
                if is_click_in_circle(world_position, fruit_center, area.radius)
                {
                    commands.entity(entity).despawn();
                    let mut minigame =
                        tree_minigames_query.get_mut(fruit.minigame).unwrap();
                    minigame.count -= 1;
                    spawn_fruit(
                        &mut commands,
                        &asset_server,
                        Transform::from_xyz(
                            fruit_center.x + 200.0,
                            fruit_center.y,
                            0.0,
                        ),
                        fruit.resource,
                    );
                }
            }
        }
    }

    // Grow fruits.
    pub fn fixed_update(
        mut commands: Commands,
        time: Res<Time>,
        asset_server: Res<AssetServer>,
        mut tree_minigames_query: Query<(&mut TreeMinigame, Entity)>,
    ) {
        for (mut minigame, entity) in tree_minigames_query.iter_mut() {
            // let max_fruit = minigame.lushness * 4.0;
            let max_fruit = 1; // TODO
            if minigame.count >= max_fruit as u32 {
                continue;
            }
            // let needed_time_seconds = 100.0 / minigame.lushness;
            let needed_time_seconds = 0.0; // TODO
            let elapsed_seconds = time.elapsed_seconds();
            if elapsed_seconds - minigame.last_fruit_time <= needed_time_seconds
            {
                continue;
            }
            minigame.last_fruit_time = elapsed_seconds;
            minigame.count += 1;
            spawn_unpicked_fruit(
                &mut commands,
                &asset_server,
                Transform::from_xyz(0.0, 0.0, 0.0),
                entity,
                minigame.fruit,
            );
        }
    }

    fn spawn_unpicked_fruit(
        commands: &mut Commands,
        asset_server: &Res<AssetServer>,
        transform: Transform,
        parent: Entity,
        fruit: GalaxiaResource,
    ) {
        let area = CircularArea { radius: 8.0 };
        commands
            .spawn((
                UnpickedFruit {
                    resource: fruit,
                    minigame: parent,
                },
                area,
                SpriteBundle {
                    texture: asset_server.load(resource_to_asset(fruit)),
                    // adjust by Z only
                    transform: Transform::from_xyz(
                        transform.translation.x,
                        transform.translation.y,
                        1.0,
                    ),
                    ..default()
                },
                Clickable,
            ))
            .set_parent(parent);
    }

    fn spawn_fruit(
        commands: &mut Commands,
        asset_server: &Res<AssetServer>,
        transform: Transform,
        fruit: GalaxiaResource,
    ) {
        let area = CircularArea { radius: 8.0 };
        commands.spawn((
            LooseResource {
                resource: fruit,
                amount: 1.0,
            },
            area,
            SpriteBundle {
                texture: asset_server.load(resource_to_asset(fruit)),
                transform,
                ..default()
            },
            RigidBody::Dynamic,
            Collider::from(area),
        ));
    }

    #[derive(Debug, Clone, Component)]
    pub struct UnpickedFruit {
        pub resource: GalaxiaResource,
        pub minigame: Entity,
    }
}

// Grid of blocks or empty spaces. The bottom has a paddle that can move left
// and right. The player inserts a ball which bounces off of or breaks the
// blocks, depending on which is harder. The ball also bounces off of the
// paddle - if the ball hits the bottom, it is lost.
// When all blocks are broken, the player wins. This gives them a copy of the
// minigame to use or deploy.
pub mod ball_breaker_minigame {
    use super::*;

    pub const DESCRIPTION: &str = "Throw balls to break blocks!";

    pub const BLOCK_SIZE: f32 = 20.0;

    #[derive(Debug, Clone, Default, Bundle)]
    pub struct BallBreakderMinigameBundle {
        pub minigame: BallBreakerMinigame,
        pub area: RectangularArea,
    }

    #[derive(Debug, Clone, Default, Component)]
    pub struct BallBreakerMinigame {
        pub blocks_per_row: u32,
        pub blocks_per_column: u32,
        pub paddle_width: f32,
        pub level: u64,
        pub balls: Vec<(Entity, f32, f32)>, // entity, x, y
    }

    pub fn spawn(
        commands: &mut Commands,
        asset_server: &AssetServer,
        mut random: &mut Random,
        transform: Transform,
        frozen: &BallBreakerMinigame,
    ) {
        let level = frozen.level;
        let blocks_per_row: u32;
        let blocks_per_column: u32;
        let paddle_width: f32;
        if level == 0 {
            blocks_per_row = 10;
            blocks_per_column = 10;
            paddle_width = BLOCK_SIZE * 3.0;
        } else {
            let r: u64 = random.rng.rand();
            blocks_per_row = (10 + (r % level)) as u32;
            blocks_per_column = (10 + (r % level)) as u32;
            paddle_width = BLOCK_SIZE * 3.0 + (r % level) as f32;
        }
        let area = RectangularArea {
            width: BLOCK_SIZE * blocks_per_row as f32,
            height: BLOCK_SIZE * blocks_per_column as f32,
        };

        let minigame = BallBreakerMinigame {
            level,
            blocks_per_row,
            blocks_per_column,
            paddle_width,
            balls: Vec::new(),
        };
        commands
            .spawn((
                BallBreakderMinigameBundle { minigame, area },
                SpatialBundle {
                    transform,
                    ..default()
                },
            ))
            .with_children(|parent| {
                let _background = parent.spawn(SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgb(1.0, 1.0, 1.0),
                        custom_size: Some(Vec2::new(area.width, area.height)),
                        ..default()
                    },
                    transform: Transform::from_xyz(0.0, 0.0, -1.0),
                    ..default()
                });
                spawn_bounding_rectangle(
                    parent,
                    RectangularArea {
                        width: area.width,
                        height: area.height,
                    },
                );

                for y in 3..blocks_per_column {
                    for x in 0..blocks_per_row {
                        let resource = random_resource(level, &mut random);
                        spawn_block(
                            parent,
                            &asset_server,
                            resource,
                            blocks_per_column,
                            blocks_per_row,
                            x,
                            y,
                        );
                    }
                }
                spawn_paddle(
                    parent,
                    &asset_server,
                    RectangularArea {
                        width: paddle_width,
                        height: BLOCK_SIZE,
                    },
                    parent.parent_entity(),
                    blocks_per_column,
                    paddle_width,
                );

                // TODO do not do this here - need an input ball
                spawn_ball(
                    parent,
                    &asset_server,
                    CircularArea {
                        radius: BLOCK_SIZE / 2.0,
                    },
                    parent.parent_entity(),
                    blocks_per_column,
                    blocks_per_row,
                    GalaxiaResource::Dirt, // TODO use actual resource
                );
            });
    }

    #[derive(Debug, Clone, Component)]
    pub struct Block {
        pub resource: GalaxiaResource,
    }

    pub fn random_resource(level: u64, random: &mut Random) -> GalaxiaResource {
        let r: u64;
        if level == 0 {
            r = 0;
        } else {
            r = 1 + random.rng.rand() % level;
        }

        match r {
            0 => GalaxiaResource::Mud,
            1 => GalaxiaResource::Dirt,
            2 => GalaxiaResource::Sandstone,
            3 => GalaxiaResource::Granite,
            4 => GalaxiaResource::Marble,
            5 => GalaxiaResource::Obsidian,
            6 => GalaxiaResource::Copper,
            7 => GalaxiaResource::Tin,
            8 => GalaxiaResource::Iron,
            9 => GalaxiaResource::Silver,
            10 => GalaxiaResource::Gold,
            11 => GalaxiaResource::Diamond,
            12 => GalaxiaResource::Amethyst,
            13 => GalaxiaResource::FreshWater,
            14 => GalaxiaResource::Moss,
            _ => GalaxiaResource::Unobtainium,
        }
    }

    pub fn resource_toughness(resource: &str) -> u32 {
        match resource {
            "mud" => 1,
            "dirt" => 2,
            "sandstone" => 3,
            "granite" => 4,
            "marble" => 4,
            "obsidian" => 2,
            "copper" => 4,
            "tin" => 4,
            "iron" => 8,
            "silver" => 4,
            "gold" => 3,
            "diamond" => 6,
            "amethyst" => 6,
            "fresh water" => 0,
            "moss" => 1,
            _ => 16,
        }
    }

    pub fn resource_damage(resource: &str) -> u32 {
        match resource {
            "mud" => 2,
            "dirt" => 3,
            "sandstone" => 4,
            "granite" => 4,
            "marble" => 4,
            "obsidian" => 6,
            "copper" => 7,
            "tin" => 7,
            "bronze" => 8, // must be forged from copper and tin
            "iron" => 10,
            "silver" => 4,
            "gold" => 3,
            "diamond" => 11,
            "amethyst" => 4,
            "fresh water" => 1,
            "moss" => 0,
            _ => 16,
        }
    }

    pub fn spawn_block(
        commands: &mut ChildBuilder,
        asset_server: &AssetServer,
        resource: GalaxiaResource,
        blocks_per_column: u32,
        blocks_per_row: u32,
        x: u32,
        y: u32,
    ) {
        let area = RectangularArea {
            width: BLOCK_SIZE,
            height: BLOCK_SIZE,
        };
        let x = BLOCK_SIZE
            * ((x as f32) - (blocks_per_row as f32 / 2.0) + 1.0 / 2.0);
        let y = BLOCK_SIZE
            * ((y as f32) - (blocks_per_column as f32 / 2.0) + 1.0 / 2.0);

        commands.spawn((
            Block { resource },
            SpriteBundle {
                texture: asset_server.load(resource_to_asset(resource)),
                transform: Transform::from_xyz(x, y, 0.0),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
                    ..default()
                },
                ..default()
            },
            area,
            Collider::from(area),
        ));
    }

    #[derive(Debug, Clone, Component)]
    pub struct Ball {
        pub resource: GalaxiaResource,
        pub minigame: Entity,
    }

    pub fn spawn_ball(
        commands: &mut ChildBuilder,
        asset_server: &AssetServer,
        area: CircularArea,
        minigame: Entity,
        blocks_per_column: u32,
        blocks_per_row: u32,
        resource: GalaxiaResource,
    ) {
        let x = -30.0 + BLOCK_SIZE * ((blocks_per_row / 2) as f32 - 0.5);
        let y = -BLOCK_SIZE * ((blocks_per_column / 2) as f32 - 1.5);
        let square = area.radius * 2.0;
        commands.spawn((
            Ball { resource, minigame },
            SpriteBundle {
                texture: asset_server.load("block_breaker/ball.png"),
                transform: Transform::from_xyz(x, y, 0.0),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(square, square)),
                    ..default()
                },
                ..default()
            },
            area,
            Collider::from(area),
            RigidBody::Dynamic {},
            Velocity::linear(Vec2::new(-1.0, 1.0)),
            LockedAxes::ROTATION_LOCKED,
            ConstantSpeed { speed: 200.0 },
            Friction {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Min,
            },
            Restitution {
                coefficient: 1.0,
                combine_rule: CoefficientCombineRule::Max,
            },
            Damping {
                linear_damping: 0.0,
                angular_damping: 0.0,
            },
        ));
    }

    #[derive(Debug, Clone, Component)]
    pub struct Paddle {
        pub minigame: Entity,
    }

    pub fn spawn_paddle(
        commands: &mut ChildBuilder,
        asset_server: &AssetServer,
        area: RectangularArea,
        minigame: Entity,
        blocks_per_column: u32,
        paddle_width: f32,
    ) {
        let x = 0.0;
        let y = -BLOCK_SIZE * ((blocks_per_column / 2) as f32 - 0.5);
        commands.spawn((
            Paddle { minigame },
            SpriteBundle {
                texture: asset_server.load("block_breaker/paddle.png"),
                transform: Transform::from_xyz(x, y, 0.0),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(paddle_width, BLOCK_SIZE)),
                    ..default()
                },
                ..default()
            },
            area,
            Collider::from(area),
        ));
    }
}

// Chest acts as an inventory. Only certain resources can be stored.
// The resource must be a solid (not a fluid, mana, abstraction, etc).
// It
pub mod chest_minigame {
    use super::*;

    pub const DESCRIPTION: &str = "Store items!";
}

pub mod board_minigame {
    use super::*;

    pub const DESCRIPTION: &str = "TODO";
}
