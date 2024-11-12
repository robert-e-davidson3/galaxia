use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use wyrand::WyRand;

use crate::entities::*;
use crate::libs::*;

// Grid of blocks or empty spaces. The bottom has a paddle that can move left
// and right. The player inserts a ball which bounces off of or breaks the
// blocks, depending on which is harder. The ball also bounces off of the
// paddle - if the ball hits the bottom, it is lost.
// When all blocks are broken, the player wins. This gives them a copy of the
// minigame to use or deploy.

pub const NAME: &str = "ball breaker";
pub const DESCRIPTION: &str = "Throw balls to break blocks!";

pub const BLOCK_SIZE: f32 = 20.0;

#[derive(Debug, Clone, Default, Component)]
pub struct BallBreakerMinigame {
    pub level: u8,
    pub balls: HashMap<PhysicalItemMaterial, u32>,
}

impl BallBreakerMinigame {
    pub fn new(level: u8) -> Self {
        Self {
            level,
            balls: HashMap::new(),
        }
    }

    //
    // COMMON
    //

    pub fn name(&self) -> &str {
        NAME
    }

    pub fn description(&self) -> &str {
        DESCRIPTION
    }

    pub fn area(&self) -> RectangularArea {
        RectangularArea {
            width: self.blocks_per_row() as f32 * BLOCK_SIZE,
            height: (3 + self.blocks_per_column()) as f32 * BLOCK_SIZE,
        }
    }

    pub fn level(&self) -> u8 {
        self.level
    }

    pub fn levelup(&self) -> Self {
        Self::new(self.level + 1)
    }

    pub fn spawn(
        &self,
        parent: &mut ChildBuilder,
        mut random: &mut Random,
        asset_server: &AssetServer,
    ) {
        let (area, blocks_per_column, blocks_per_row, level) = (
            self.area(),
            self.blocks_per_column(),
            self.blocks_per_row(),
            self.level,
        );
        let _background = parent.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(1.0, 1.0, 1.0),
                custom_size: Some(area.into()),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, -1.0),
            ..default()
        });

        for y in 3..(blocks_per_column + 3) {
            for x in 0..blocks_per_row {
                parent.spawn(BlockBundle::new(
                    asset_server,
                    BallBreakerMinigame::random_material(level, &mut random),
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

        // TODO empty out balls as loose items
    }

    //
    // SPECIFIC
    //

    pub fn blocks_per_row(&self) -> u32 {
        Self::calculate_blocks_per_row(self.level)
    }

    pub fn blocks_per_column(&self) -> u32 {
        Self::calculate_blocks_per_column(self.level)
    }

    pub fn calculate_blocks_per_row(level: u8) -> u32 {
        10 + (level as u32 / 10)
    }

    pub fn calculate_blocks_per_column(level: u8) -> u32 {
        7 + (level as u32 / 10)
    }

    pub fn item_is_valid(item: &Item) -> bool {
        let physical = match item.r#type {
            ItemType::Physical(data) => data,
            _ => return false,
        };

        match physical.material {
            PhysicalItemMaterial::Mud
            | PhysicalItemMaterial::Dirt
            | PhysicalItemMaterial::Sandstone
            | PhysicalItemMaterial::Granite
            | PhysicalItemMaterial::Marble
            | PhysicalItemMaterial::Obsidian
            | PhysicalItemMaterial::Copper
            | PhysicalItemMaterial::Tin
            | PhysicalItemMaterial::Iron
            | PhysicalItemMaterial::Silver
            | PhysicalItemMaterial::Gold
            | PhysicalItemMaterial::Diamond
            | PhysicalItemMaterial::Amethyst
            | PhysicalItemMaterial::FreshWater
            | PhysicalItemMaterial::Moss => true,
            _ => false,
        }
    }

    pub fn random_material(
        level: u8,
        random: &mut Random,
    ) -> PhysicalItemMaterial {
        let r: u64;
        if level == 0 {
            r = 0;
        } else {
            r = 1 + random.next() % (level as u64);
        }

        match r {
            0 => PhysicalItemMaterial::Mud,
            1 => PhysicalItemMaterial::Dirt,
            2 => PhysicalItemMaterial::Sandstone,
            3 => PhysicalItemMaterial::Granite,
            4 => PhysicalItemMaterial::Marble,
            5 => PhysicalItemMaterial::Obsidian,
            6 => PhysicalItemMaterial::Copper,
            7 => PhysicalItemMaterial::Tin,
            8 => PhysicalItemMaterial::Iron,
            9 => PhysicalItemMaterial::Silver,
            10 => PhysicalItemMaterial::Gold,
            11 => PhysicalItemMaterial::Diamond,
            12 => PhysicalItemMaterial::Amethyst,
            13 => PhysicalItemMaterial::FreshWater,
            14 => PhysicalItemMaterial::Moss,
            _ => PhysicalItemMaterial::Unobtainium,
        }
    }

    pub fn material_toughness(resource: PhysicalItemMaterial) -> u32 {
        match resource {
            PhysicalItemMaterial::Mud => 1,
            PhysicalItemMaterial::Dirt => 2,
            PhysicalItemMaterial::Sandstone => 3,
            PhysicalItemMaterial::Granite => 4,
            PhysicalItemMaterial::Marble => 4,
            PhysicalItemMaterial::Obsidian => 2,
            PhysicalItemMaterial::Copper => 4,
            PhysicalItemMaterial::Tin => 4,
            PhysicalItemMaterial::Iron => 8,
            PhysicalItemMaterial::Silver => 4,
            PhysicalItemMaterial::Gold => 3,
            PhysicalItemMaterial::Diamond => 6,
            PhysicalItemMaterial::Amethyst => 6,
            PhysicalItemMaterial::FreshWater => 0,
            PhysicalItemMaterial::Moss => 1,
            _ => 16,
        }
    }

    pub fn material_damage(resource: PhysicalItemMaterial) -> u32 {
        match resource {
            PhysicalItemMaterial::Mud => 2,
            PhysicalItemMaterial::Dirt => 3,
            PhysicalItemMaterial::Sandstone => 4,
            PhysicalItemMaterial::Granite => 4,
            PhysicalItemMaterial::Marble => 4,
            PhysicalItemMaterial::Obsidian => 6,
            PhysicalItemMaterial::Copper => 7,
            PhysicalItemMaterial::Tin => 7,
            PhysicalItemMaterial::Bronze => 8, // must be forged from copper and tin
            PhysicalItemMaterial::Iron => 10,
            PhysicalItemMaterial::Silver => 4,
            PhysicalItemMaterial::Gold => 3,
            PhysicalItemMaterial::Diamond => 11,
            PhysicalItemMaterial::Amethyst => 4,
            PhysicalItemMaterial::FreshWater => 1,
            PhysicalItemMaterial::Moss => 0,
            _ => 16,
        }
    }

    // counts ball material
    pub fn add_ball(&mut self, material: PhysicalItemMaterial) {
        *self.balls.entry(material).or_insert(0) += 1;
    }

    // decrements ball material
    pub fn remove_ball(&mut self, material: PhysicalItemMaterial) {
        if let Entry::Occupied(mut entry) = self.balls.entry(material) {
            let count = entry.get_mut();
            if *count > 0 {
                *count -= 1;
            }
        }
    }
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
            * ((y as f32) - ((blocks_per_column + 3) as f32 / 2.0) + 1.0 / 2.0);
        Self {
            block: Block { material },
            sprite: SpriteBundle {
                texture: asset_server.load(
                    Item::new_physical(PhysicalItemForm::Block, material, 1.0)
                        .asset(),
                ),
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
    pub material: PhysicalItemMaterial,
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
        images: &mut ResMut<Assets<Image>>,
        generated_image_assets: &mut image_gen::GeneratedImageAssets,
        material: PhysicalItemMaterial,
        minigame: Entity,
        blocks_per_column: u32,
        blocks_per_row: u32,
    ) -> Self {
        let x = BLOCK_SIZE * ((blocks_per_row / 2) as f32 - 2.0);
        let y = -BLOCK_SIZE * (((blocks_per_column + 3) / 2) as f32 - 1.0);
        let area = CircularArea {
            radius: BLOCK_SIZE / 2.0,
        };
        let item = Item::new_physical(PhysicalItemForm::Ball, material, 1.0);
        let texture: Handle<Image> =
            match generated_image_assets.get(&item.uid()) {
                Some(image) => image,
                None => {
                    let image = item.draw(&mut WyRand::new(SEED));
                    let handle = images.add(image.clone());
                    generated_image_assets.insert(item.uid(), &handle);
                    handle
                }
            };
        Self {
            ball: Ball { material, minigame },
            sprite: SpriteBundle {
                texture,
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
    pub material: PhysicalItemMaterial,
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
        let y = -BLOCK_SIZE * (((blocks_per_column + 3) as f32 / 2.0) - 0.5);
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
    minigame_query: Query<(&RectangularArea, &GlobalTransform), With<Minigame>>,
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
    mut images: ResMut<Assets<Image>>,
    mut generated_image_assets: ResMut<image_gen::GeneratedImageAssets>,
    mut collision_events: EventReader<CollisionEvent>,
    mut minigame_query: Query<(
        &mut Minigame,
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
        let ball_material: PhysicalItemMaterial;
        let minigame_entity: Entity;
        match ball_query.get(*a) {
            Ok(ball) => {
                ball_entity = *a;
                block_entity = *b;
                ball_material = ball.material;
                minigame_entity = ball.minigame;
            }
            Err(_) => match ball_query.get(*b) {
                Ok(ball) => {
                    ball_entity = *b;
                    block_entity = *a;
                    ball_material = ball.material;
                    minigame_entity = ball.minigame;
                }
                Err(_) => continue,
            },
        };

        let block_material: PhysicalItemMaterial =
            match block_query.get(block_entity) {
                Ok(x) => x.material,
                Err(_) => continue,
            };

        if broken.contains(&block_entity) || broken.contains(&ball_entity) {
            continue;
        }

        // get minigame
        let (minigame, minigame_global_transform, minigame_area) =
            match minigame_query.get_mut(minigame_entity) {
                Ok(x) => x,
                Err(_) => continue,
            };
        let minigame = match minigame.into_inner() {
            Minigame::BallBreaker(x) => x,
            _ => continue,
        };

        // break stuff! and spit out resources!
        if BallBreakerMinigame::material_damage(ball_material)
            >= BallBreakerMinigame::material_toughness(block_material)
        {
            commands.entity(block_entity).despawn();
            broken.insert(block_entity);
            commands.spawn(ItemBundle::new_from_minigame(
                &mut images,
                &mut generated_image_assets,
                Item::new_physical(
                    PhysicalItemForm::Powder,
                    block_material,
                    1.0,
                ),
                minigame_global_transform,
                minigame_area,
            ));

            // this was the last block, so reset and level up!
            if block_query.iter().count() == 1 {
                commands.entity(minigame_entity).insert(LevelingUp);
            }
        }
        if BallBreakerMinigame::material_damage(block_material)
            >= BallBreakerMinigame::material_toughness(ball_material)
        {
            commands.entity(ball_entity).despawn();
            broken.insert(ball_entity);
            minigame.remove_ball(ball_material);
            commands.spawn(ItemBundle::new_from_minigame(
                &mut images,
                &mut generated_image_assets,
                Item::new_physical(
                    PhysicalItemForm::Powder,
                    ball_material,
                    1.0,
                ),
                minigame_global_transform,
                minigame_area,
            ));
        }
    }
}

pub fn ingest_resource_fixed_update(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut generated_image_assets: ResMut<image_gen::GeneratedImageAssets>,
    mut collision_events: EventReader<CollisionEvent>,
    mut minigame_query: Query<&mut Minigame>,
    aura_query: Query<&MinigameAura>,
    mut item_query: Query<&mut Item>,
) {
    let mut ingested: HashSet<Entity> = HashSet::new();
    for event in collision_events.read() {
        // only care about collision start
        let (a, b) = match event {
            CollisionEvent::Started(a, b, _flags) => (a, b),
            _ => continue,
        };

        // only care about collisions of resources
        let (item_entity, aura_entity, mut item) = match item_query.get_mut(*a)
        {
            Ok(item) => (a, b, item),
            Err(_) => match item_query.get_mut(*b) {
                Ok(item) => (b, a, item),
                Err(_) => continue,
            },
        };

        // already handled
        if ingested.contains(&item_entity) {
            continue;
        }

        // only certain resources can be ingested
        if !BallBreakerMinigame::item_is_valid(&item) {
            continue;
        }

        // need enough resource to form ball
        if item.amount < 1.0 {
            continue;
        }

        // only care about collisions of resources with minigame auras
        let aura = match aura_query.get(*aura_entity) {
            Ok(x) => x,
            Err(_) => continue,
        };
        // only applies to ball breaker minigame
        let minigame = match minigame_query.get_mut(aura.minigame) {
            Ok(x) => x,
            Err(_) => continue,
        };
        let minigame = match minigame.into_inner() {
            Minigame::BallBreaker(m) => m,
            _ => continue,
        };

        // too small to form ball
        if item.amount < 1.0 {
            continue;
        }

        ingested.insert(*item_entity);

        // deplete item
        item.amount -= 1.0;

        // item is destroyed
        if item.amount == 0.0 {
            commands.entity(*item_entity).despawn_recursive();
        }

        // add ball to minigame
        let material = match item.r#type {
            ItemType::Physical(x) => x.material,
            _ => continue,
        };
        commands.entity(*aura_entity).with_children(|parent| {
            parent.spawn(BallBundle::new(
                &mut images,
                &mut generated_image_assets,
                material,
                aura.minigame,
                minigame.blocks_per_column(),
                minigame.blocks_per_row(),
            ));
        });
        minigame.add_ball(material);
    }
}
