use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::area::*;
use crate::collision::*;
use crate::common::*;
use crate::mouse::*;
use crate::random::*;
use crate::resource::*;

// Grid of blocks or empty spaces. The bottom has a paddle that can move left
// and right. The player inserts a ball which bounces off of or breaks the
// blocks, depending on which is harder. The ball also bounces off of the
// paddle - if the ball hits the bottom, it is lost.
// When all blocks are broken, the player wins. This gives them a copy of the
// minigame to use or deploy.

pub const NAME: &str = "ball breaker";
pub const _DESCRIPTION: &str = "Throw balls to break blocks!";

pub const BLOCK_SIZE: f32 = 20.0;

#[derive(Debug, Clone, Bundle)]
pub struct BallBreakerMinigameBundle {
    pub minigame: BallBreakerMinigame,
    pub area: RectangularArea,
    pub tag: Minigame,
    pub aura: Collider,
    pub collision_groups: CollisionGroups,
    pub active_events: ActiveEvents,
    pub spatial: SpatialBundle,
}

impl BallBreakerMinigameBundle {
    pub fn new(
        minigame: BallBreakerMinigame,
        area: RectangularArea,
        transform: Transform,
    ) -> Self {
        Self {
            minigame,
            area,
            tag: Minigame,
            aura: area.grow(1.0, 1.0).into(),
            collision_groups: CollisionGroups::new(
                MINIGAME_AURA_GROUP,
                minigame_aura_filter(),
            ),
            active_events: ActiveEvents::COLLISION_EVENTS,
            spatial: SpatialBundle {
                transform,
                ..default()
            },
        }
    }
}

#[derive(Debug, Clone, Default, Component)]
pub struct BallBreakerMinigame {
    pub blocks_per_row: u32,
    pub blocks_per_column: u32,
    pub _paddle_width: f32,
    pub level: u64,
    pub _balls: Vec<(GalaxiaResource, f32, f32)>, // for (de)serialization
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
        let r: u64 = random.next();
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
        _paddle_width: paddle_width,
        _balls: Vec::new(),
    };
    commands
        .spawn(BallBreakerMinigameBundle::new(minigame, area, transform))
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
            spawn_minigame_container(parent, area, NAME);

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

pub fn resource_is_valid(resource: GalaxiaResource) -> bool {
    match resource {
        GalaxiaResource::Mud
        | GalaxiaResource::Dirt
        | GalaxiaResource::Sandstone
        | GalaxiaResource::Granite
        | GalaxiaResource::Marble
        | GalaxiaResource::Obsidian
        | GalaxiaResource::Copper
        | GalaxiaResource::Tin
        | GalaxiaResource::Iron
        | GalaxiaResource::Silver
        | GalaxiaResource::Gold
        | GalaxiaResource::Diamond
        | GalaxiaResource::Amethyst
        | GalaxiaResource::FreshWater
        | GalaxiaResource::Moss => true,
        _ => false,
    }
}

pub fn random_resource(level: u64, random: &mut Random) -> GalaxiaResource {
    let r: u64;
    if level == 0 {
        r = 0;
    } else {
        r = 1 + random.next() % level;
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
    let x =
        BLOCK_SIZE * ((x as f32) - (blocks_per_row as f32 / 2.0) + 1.0 / 2.0);
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
        CollisionGroups::new(
            MINIGAME_CONTENTS_GROUP,
            minigame_contents_filter(),
        ),
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
    let x = BLOCK_SIZE * ((blocks_per_row / 2) as f32 - 2.0);
    let y = -BLOCK_SIZE * ((blocks_per_column / 2) as f32 - 1.0);
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
        CollisionGroups::new(
            MINIGAME_CONTENTS_GROUP,
            minigame_contents_filter(),
        ),
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
        CollisionGroups::new(
            MINIGAME_CONTENTS_GROUP,
            minigame_contents_filter(),
        ),
    ));
}

pub fn unselected_paddle_update(
    mut commands: Commands,
    mut paddle_query: Query<
        (Entity, &Paddle, &GlobalTransform, &RectangularArea),
        Without<FollowsMouse>,
    >,
    minigame_query: Query<
        (&RectangularArea, &GlobalTransform),
        With<BallBreakerMinigame>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    window_query: Query<&Window>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
) {
    let click_position = match get_click_press_position(
        camera_query,
        window_query,
        mouse_button_input,
    ) {
        Some(world_position) => world_position,
        None => return,
    };

    for (paddle_entity, paddle, paddle_global_transform, paddle_area) in
        paddle_query.iter_mut()
    {
        let paddle_position = paddle_global_transform.translation().truncate();
        if !paddle_area.is_within(click_position, paddle_position) {
            continue;
        }

        let (minigame_area, minigame_global_transform) =
            minigame_query.get(paddle.minigame).unwrap();

        commands.entity(paddle_entity).insert(FollowsMouse::new(
            RectangularArea {
                width: minigame_area.width,
                height: 0.0, // only moves on x-axis
            },
            Vec2::new(
                minigame_global_transform.translation().truncate().x,
                paddle_position.y,
            ),
            *paddle_area,
            click_position - paddle_position,
            true,
        ));
    }
}

#[derive(Debug, Copy, Clone, Component)]
pub struct ConstantSpeed {
    pub speed: f32,
}

pub fn constant_velocity_system(
    mut query: Query<(&ConstantSpeed, &mut Velocity)>,
) {
    for (speed, mut velocity) in query.iter_mut() {
        if speed.speed == 0.0 {
            velocity.linvel = Vec2::ZERO;
        } else {
            velocity.linvel = velocity.linvel.normalize() * speed.speed;
        }
    }
}

pub fn ingest_resource_fixed_update(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut collision_events: EventReader<CollisionEvent>,
    minigame_query: Query<&BallBreakerMinigame>,
    resource_query: Query<&LooseResource>,
) {
    for event in collision_events.read() {
        // only care about collision start
        let (a, b) = match event {
            CollisionEvent::Started(a, b, _flags) => (a, b),
            _ => continue,
        };

        // only care about collisions between minigames and resources
        let resource_entity: Entity;
        let minigame_entity: Entity;
        let resource: &LooseResource;
        match resource_query.get(*a) {
            Ok(x) => {
                resource_entity = *a;
                minigame_entity = *b;
                resource = x;
            }
            Err(_) => match resource_query.get(*b) {
                Ok(x) => {
                    resource_entity = *b;
                    minigame_entity = *a;
                    resource = x;
                }
                Err(_) => continue,
            },
        };

        // only certain resources can be ingested
        if !resource_is_valid(resource.resource) {
            continue;
        }

        let minigame = match minigame_query.get(minigame_entity) {
            Ok(x) => x,
            Err(_) => continue,
        };

        // remove loose resource
        commands.entity(resource_entity).despawn();

        // add ball to minigame
        commands.entity(minigame_entity).with_children(|parent| {
            spawn_ball(
                parent,
                &asset_server,
                CircularArea {
                    radius: BLOCK_SIZE / 2.0,
                    ..default()
                },
                minigame_entity,
                minigame.blocks_per_column,
                minigame.blocks_per_row,
                resource.resource,
            );
        });
    }
}
