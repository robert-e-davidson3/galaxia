use std::collections::HashSet;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::entities::*;
use crate::libs::*;

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
    pub level: u64,
    pub _balls: Vec<(Item, f32, f32)>, // (item,x,y) - for (de)serialization
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
    if level == 0 {
        blocks_per_row = 10;
        blocks_per_column = 10;
    } else {
        let r: u64 = random.next();
        blocks_per_row = (10 + (r % level)) as u32;
        blocks_per_column = (10 + (r % level)) as u32;
    }
    let area = RectangularArea {
        width: BLOCK_SIZE * blocks_per_row as f32,
        height: BLOCK_SIZE * blocks_per_column as f32,
    };

    let minigame = BallBreakerMinigame {
        level,
        blocks_per_row,
        blocks_per_column,
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
            parent.spawn(MinigameAuraBundle::new(parent.parent_entity(), area));
            spawn_minigame_container(parent, area, NAME);

            for y in 3..blocks_per_column {
                for x in 0..blocks_per_row {
                    parent.spawn(BlockBundle::new(
                        asset_server,
                        random_resource(level, &mut random),
                        blocks_per_column,
                        blocks_per_row,
                        x,
                        y,
                    ));
                }
            }
            parent.spawn(PaddleBundle::new(
                asset_server,
                parent.parent_entity(),
                blocks_per_column,
            ));
        });
}

#[derive(Debug, Clone, Bundle)]
pub struct BlockBundle {
    pub block: Block,
    pub sprite: SpriteBundle,
    pub area: RectangularArea,
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
}

impl BlockBundle {
    pub fn new(
        asset_server: &AssetServer,
        material: PhysicalItemMaterial,
        blocks_per_column: u32,
        blocks_per_row: u32,
        x: u32,
        y: u32,
    ) -> Self {
        let area = RectangularArea {
            width: BLOCK_SIZE,
            height: BLOCK_SIZE,
        };
        let x = BLOCK_SIZE
            * ((x as f32) - (blocks_per_row as f32 / 2.0) + 1.0 / 2.0);
        let y = BLOCK_SIZE
            * ((y as f32) - (blocks_per_column as f32 / 2.0) + 1.0 / 2.0);
        Self {
            block: Block { resource: material },
            sprite: SpriteBundle {
                texture: asset_server.load(Item {
                    item_type: ItemType::Physical,
                    item_data: ItemData {
                        physical: PhysicalItem {
                            form: PhysicalItemForm::Object,
                            material,
                        },
                    },
                    amount: 1.0,
                }),
                // texture: asset_server.load(resource_to_asset(material)),
                transform: Transform::from_xyz(x, y, 0.0),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
                    ..default()
                },
                ..default()
            },
            area,
            collider: area.into(),
            collision_groups: CollisionGroups::new(
                MINIGAME_CONTENTS_GROUP,
                minigame_contents_filter(),
            ),
        }
    }
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

pub fn resource_toughness(resource: GalaxiaResource) -> u32 {
    match resource {
        GalaxiaResource::Mud => 1,
        GalaxiaResource::Dirt => 2,
        GalaxiaResource::Sandstone => 3,
        GalaxiaResource::Granite => 4,
        GalaxiaResource::Marble => 4,
        GalaxiaResource::Obsidian => 2,
        GalaxiaResource::Copper => 4,
        GalaxiaResource::Tin => 4,
        GalaxiaResource::Iron => 8,
        GalaxiaResource::Silver => 4,
        GalaxiaResource::Gold => 3,
        GalaxiaResource::Diamond => 6,
        GalaxiaResource::Amethyst => 6,
        GalaxiaResource::FreshWater => 0,
        GalaxiaResource::Moss => 1,
        _ => 16,
    }
}

pub fn resource_damage(resource: GalaxiaResource) -> u32 {
    match resource {
        GalaxiaResource::Mud => 2,
        GalaxiaResource::Dirt => 3,
        GalaxiaResource::Sandstone => 4,
        GalaxiaResource::Granite => 4,
        GalaxiaResource::Marble => 4,
        GalaxiaResource::Obsidian => 6,
        GalaxiaResource::Copper => 7,
        GalaxiaResource::Tin => 7,
        GalaxiaResource::Bronze => 8, // must be forged from copper and tin
        GalaxiaResource::Iron => 10,
        GalaxiaResource::Silver => 4,
        GalaxiaResource::Gold => 3,
        GalaxiaResource::Diamond => 11,
        GalaxiaResource::Amethyst => 4,
        GalaxiaResource::FreshWater => 1,
        GalaxiaResource::Moss => 0,
        _ => 16,
    }
}

#[derive(Debug, Clone, Bundle)]
pub struct BallBundle {
    pub ball: Ball,
    pub sprite: SpriteBundle,
    pub area: CircularArea,
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
    pub rigid_body: RigidBody,
    pub velocity: Velocity,
    pub locked_axes: LockedAxes,
    pub constant_speed: ConstantSpeed,
    pub friction: Friction,
    pub restitution: Restitution,
    pub damping: Damping,
    pub active_events: ActiveEvents,
}

impl BallBundle {
    pub fn new(
        asset_server: &AssetServer,
        resource: GalaxiaResource,
        minigame: Entity,
        blocks_per_column: u32,
        blocks_per_row: u32,
    ) -> Self {
        let x = BLOCK_SIZE * ((blocks_per_row / 2) as f32 - 2.0);
        let y = -BLOCK_SIZE * ((blocks_per_column / 2) as f32 - 1.0);
        let area = CircularArea {
            radius: BLOCK_SIZE / 2.0,
        };
        Self {
            ball: Ball { resource, minigame },
            sprite: SpriteBundle {
                texture: asset_server.load("block_breaker/ball.png"),
                transform: Transform::from_xyz(x, y, 0.0),
                sprite: Sprite {
                    custom_size: Some(area.into()),
                    ..default()
                },
                ..default()
            },
            area,
            collider: Collider::from(area),
            collision_groups: CollisionGroups::new(
                MINIGAME_CONTENTS_GROUP,
                minigame_contents_filter(),
            ),
            rigid_body: RigidBody::Dynamic {},
            velocity: Velocity::linear(Vec2::new(-1.0, 1.0)),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            constant_speed: ConstantSpeed { speed: 200.0 },
            friction: Friction {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Min,
            },
            restitution: Restitution {
                coefficient: 1.0,
                combine_rule: CoefficientCombineRule::Max,
            },
            damping: Damping {
                linear_damping: 0.0,
                angular_damping: 0.0,
            },
            active_events: ActiveEvents::COLLISION_EVENTS,
        }
    }
}

#[derive(Debug, Clone, Component)]
pub struct Ball {
    pub resource: GalaxiaResource,
    pub minigame: Entity,
}

#[derive(Debug, Clone, Bundle)]
pub struct PaddleBundle {
    pub paddle: Paddle,
    pub sprite: SpriteBundle,
    pub area: RectangularArea,
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
}

impl PaddleBundle {
    pub fn new(
        asset_server: &AssetServer,
        minigame: Entity,
        blocks_per_column: u32,
    ) -> Self {
        let x = 0.0;
        let y = -BLOCK_SIZE * ((blocks_per_column as f32 / 2.0) - 0.5);
        let area = RectangularArea {
            width: BLOCK_SIZE * 3.0,
            height: BLOCK_SIZE,
        };
        Self {
            paddle: Paddle { minigame },
            sprite: SpriteBundle {
                texture: asset_server.load("block_breaker/paddle.png"),
                transform: Transform::from_xyz(x, y, 0.0),
                sprite: Sprite {
                    custom_size: Some(area.into()),
                    ..default()
                },
                ..default()
            },
            area,
            collider: Collider::from(area),
            collision_groups: CollisionGroups::new(
                MINIGAME_CONTENTS_GROUP,
                minigame_contents_filter(),
            ),
        }
    }
}

#[derive(Debug, Clone, Component)]
pub struct Paddle {
    pub minigame: Entity,
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

pub fn hit_block_fixed_update(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut random: ResMut<Random>,
    mut collision_events: EventReader<CollisionEvent>,
    mut minigame_query: Query<(
        &mut BallBreakerMinigame,
        &GlobalTransform,
        &RectangularArea,
    )>,
    ball_query: Query<&Ball>,
    block_query: Query<&Block>,
) {
    let mut broken: HashSet<Entity> = HashSet::new();

    for event in collision_events.read() {
        // only care about collision start
        let (a, b) = match event {
            CollisionEvent::Started(a, b, _flags) => (a, b),
            _ => continue,
        };

        // only care about collisions between balls and blocks
        let ball_entity: Entity;
        let block_entity: Entity;
        let ball_resource: GalaxiaResource;
        let minigame_entity: Entity;
        match ball_query.get(*a) {
            Ok(ball) => {
                ball_entity = *a;
                block_entity = *b;
                ball_resource = ball.resource;
                minigame_entity = ball.minigame;
            }
            Err(_) => match ball_query.get(*b) {
                Ok(ball) => {
                    ball_entity = *b;
                    block_entity = *a;
                    ball_resource = ball.resource;
                    minigame_entity = ball.minigame;
                }
                Err(_) => continue,
            },
        };

        let block_resource: GalaxiaResource =
            match block_query.get(block_entity) {
                Ok(x) => x.resource,
                Err(_) => continue,
            };

        if broken.contains(&block_entity) || broken.contains(&ball_entity) {
            continue;
        }

        // get minigame
        let (mut minigame, minigame_global_transform, minigame_area) =
            match minigame_query.get_mut(minigame_entity) {
                Ok(x) => x,
                Err(_) => continue,
            };

        // break stuff! and spit out resources!
        if resource_damage(ball_resource) >= resource_toughness(block_resource)
        {
            commands.entity(block_entity).despawn();
            broken.insert(block_entity);
            commands.spawn(ItemBundle::new_from_minigame(
                &asset_server,
                block_resource,
                1.0,
                minigame_global_transform,
                minigame_area,
            ));

            // TODO move leveling up to another phase because it throws tons of
            //      warnings here by trying to despawn what's already been
            //      despawned yet despawn_recursive is needed anyway

            // this was the last block, so reset and level up!
            if block_query.iter().count() == 1 {
                if minigame.level < 99 {
                    minigame.level += 1;
                }
                for ball in ball_query.iter() {
                    if ball.minigame != minigame_entity {
                        continue;
                    }
                    if broken.contains(&ball_entity) {
                        continue;
                    }
                    // TODO spawn in ball form, if appropriate
                    // TODO check if ball broke here and so should spawn as pulverized, if appropriate
                    commands.spawn(ItemBundle::new_from_minigame(
                        &asset_server,
                        ball.resource,
                        1.0,
                        minigame_global_transform,
                        minigame_area,
                    ));
                }

                // Despawn and recreate
                commands.entity(minigame_entity).despawn_recursive();
                spawn(
                    &mut commands,
                    &asset_server,
                    &mut random,
                    Transform::from_translation(Vec3::new(
                        minigame_global_transform.translation().x,
                        minigame_global_transform.translation().y,
                        0.0,
                    )),
                    &minigame,
                );
            }
        }
        if resource_damage(block_resource) >= resource_toughness(ball_resource)
        {
            commands.entity(ball_entity).despawn();
            broken.insert(ball_entity);
            commands.spawn(ItemBundle::new_from_minigame(
                &asset_server,
                ball_resource,
                1.0,
                minigame_global_transform,
                minigame_area,
            ));
        }
    }
}

pub fn ingest_resource_fixed_update(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut collision_events: EventReader<CollisionEvent>,
    minigame_query: Query<(&BallBreakerMinigame, &Transform)>,
    aura_query: Query<&MinigameAura>,
    resource_query: Query<(&Item, &Transform)>,
) {
    let mut ingested: HashSet<Entity> = HashSet::new();
    for event in collision_events.read() {
        // only care about collision start
        let (a, b) = match event {
            CollisionEvent::Started(a, b, _flags) => (a, b),
            _ => continue,
        };

        // only care about collisions of resources
        let (resource_entity, aura_entity, resource, resource_transform) =
            match resource_query.get(*a) {
                Ok(x) => (a, b, x.0, x.1),
                Err(_) => match resource_query.get(*b) {
                    Ok(x) => (b, a, x.0, x.1),
                    Err(_) => continue,
                },
            };

        // already handled
        if ingested.contains(&resource_entity) {
            continue;
        }

        // only certain resources can be ingested
        if !resource_is_valid(resource.resource) {
            continue;
        }

        // need enough resource to form ball
        if resource.amount < 1.0 {
            continue;
        }

        // only care about collisions of resources with minigame auras
        let aura = match aura_query.get(*aura_entity) {
            Ok(x) => x,
            Err(_) => continue,
        };
        // only applies to ball breaker minigame
        let (minigame, minigame_transform) =
            match minigame_query.get(aura.minigame) {
                Ok(x) => x,
                Err(_) => continue,
            };

        // deplete or remove resource
        commands.entity(*resource_entity).despawn_recursive();
        ingested.insert(*resource_entity);

        let amount = resource.amount - 1.0;
        if amount > 0.0 {
            let velocity = (resource_transform.translation
                - minigame_transform.translation)
                .truncate();
            commands.spawn(ItemBundle::new(
                &asset_server,
                resource.resource,
                amount,
                *resource_transform,
                Velocity::linear(velocity.normalize() * 70.0),
            ));
        }

        // add ball to minigame
        commands.entity(*aura_entity).with_children(|parent| {
            parent.spawn(BallBundle::new(
                &asset_server,
                resource.resource,
                aura.minigame,
                minigame.blocks_per_column,
                minigame.blocks_per_row,
            ));
        });
    }
}
