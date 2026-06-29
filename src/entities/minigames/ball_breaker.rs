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

pub const ID: &str = "ball_breaker";
pub const POSITION: Vec2 = Vec2::new(0.0, 900.0);

pub const NAME: &str = "ball breaker";
pub const DESCRIPTION: &str = "Throw balls to break blocks!";

pub const BLOCK_SIZE: f32 = 20.0;

#[derive(Debug, Clone, Default, Component)]
pub struct BallBreakerMinigame {
    pub level: u8,
    pub balls: HashMap<Substance, u32>,
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
        parent: &mut ChildSpawnerCommands,
        random: &mut Random,
        asset_server: &AssetServer,
    ) {
        let (area, blocks_per_column, blocks_per_row, level) = (
            self.area(),
            self.blocks_per_column(),
            self.blocks_per_row(),
            self.level,
        );
        let _background = parent.spawn((
            Sprite {
                color: Color::srgb(1.0, 1.0, 1.0),
                custom_size: Some(area.into()),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, -1.0),
        ));

        for y in 3..(blocks_per_column + 3) {
            for x in 0..blocks_per_row {
                parent.spawn(BlockBundle::new(
                    asset_server,
                    BallBreakerMinigame::random_substance(level, random),
                    blocks_per_column,
                    blocks_per_row,
                    x,
                    y,
                ));
            }
        }
        parent.spawn(PaddleBundle::new(
            asset_server,
            parent.target_entity(),
            blocks_per_column,
        ));

        // TODO empty out balls as loose items
    }

    pub fn ingest_item(
        &mut self,
        commands: &mut Commands,
        images: &mut Assets<Image>,
        generated_image_assets: &mut image_gen::GeneratedImageAssets,
        minigame_entity: Entity,
        item: &Item,
    ) -> f32 {
        // Need at least 1.0 to form a ball
        if item.amount < 1.0 {
            return 0.0;
        }

        let Some(substance) = Self::item_is_valid(item) else {
            return 0.0;
        };

        self.add_ball(substance);
        // TODO verify this works since its parent is minigame instead of aura
        commands.entity(minigame_entity).with_children(|parent| {
            parent.spawn(BallBundle::new(
                images,
                generated_image_assets,
                substance,
                minigame_entity,
                self.blocks_per_column(),
                self.blocks_per_row(),
            ));
        });

        1.0 // Ball uses 1.0 of the item
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

    pub fn item_is_valid(item: &Item) -> Option<Substance> {
        let ItemType::Physical(PhysicalItem::Bulk(bulk)) = item.r#type else {
            return None;
        };

        let valid = matches!(
            bulk.substance,
            Substance::Mud
                | Substance::Dirt
                | Substance::Sandstone
                | Substance::Granite
                | Substance::Marble
                | Substance::Obsidian
                | Substance::Copper
                | Substance::Tin
                | Substance::Iron
                | Substance::Silver
                | Substance::Gold
                | Substance::Diamond
                | Substance::Amethyst
                | Substance::FreshWater
                | Substance::Moss
        );
        valid.then_some(bulk.substance)
    }

    pub fn random_substance(level: u8, random: &mut Random) -> Substance {
        let r: u64 = if level == 0 {
            0
        } else {
            1 + random.next() % (level as u64)
        };

        match r {
            0 => Substance::Mud,
            1 => Substance::Dirt,
            2 => Substance::Sandstone,
            3 => Substance::Granite,
            4 => Substance::Marble,
            5 => Substance::Obsidian,
            6 => Substance::Copper,
            7 => Substance::Tin,
            8 => Substance::Iron,
            9 => Substance::Silver,
            10 => Substance::Gold,
            11 => Substance::Diamond,
            12 => Substance::Amethyst,
            13 => Substance::FreshWater,
            14 => Substance::Moss,
            _ => Substance::Unobtainium,
        }
    }

    pub fn material_toughness(substance: Substance) -> u32 {
        match substance {
            Substance::Mud => 1,
            Substance::Dirt => 2,
            Substance::Sandstone => 3,
            Substance::Granite => 4,
            Substance::Marble => 4,
            Substance::Obsidian => 2,
            Substance::Copper => 4,
            Substance::Tin => 4,
            Substance::Iron => 8,
            Substance::Silver => 4,
            Substance::Gold => 3,
            Substance::Diamond => 6,
            Substance::Amethyst => 6,
            Substance::FreshWater => 0,
            Substance::Moss => 1,
            _ => 16,
        }
    }

    pub fn material_damage(substance: Substance) -> u32 {
        match substance {
            Substance::Mud => 2,
            Substance::Dirt => 3,
            Substance::Sandstone => 4,
            Substance::Granite => 4,
            Substance::Marble => 4,
            Substance::Obsidian => 6,
            Substance::Copper => 7,
            Substance::Tin => 7,
            Substance::Bronze => 8, // must be forged from copper and tin
            Substance::Iron => 10,
            Substance::Silver => 4,
            Substance::Gold => 3,
            Substance::Diamond => 11,
            Substance::Amethyst => 4,
            Substance::FreshWater => 1,
            Substance::Moss => 0,
            _ => 16,
        }
    }

    // counts ball substance
    pub fn add_ball(&mut self, substance: Substance) {
        *self.balls.entry(substance).or_insert(0) += 1;
    }

    // decrements ball substance
    pub fn remove_ball(&mut self, substance: Substance) {
        if let Entry::Occupied(mut entry) = self.balls.entry(substance) {
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
    pub sprite: Sprite,
    pub transform: Transform,
    pub area: RectangularArea,
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
}

impl BlockBundle {
    pub fn new(
        asset_server: &AssetServer,
        substance: Substance,
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
            block: Block { substance },
            sprite: Sprite {
                image: asset_server.load(
                    Item::solid(substance, BulkShape::Block, 1.0).asset(),
                ),
                custom_size: Some(Vec2::new(BLOCK_SIZE, BLOCK_SIZE)),
                ..default()
            },
            transform: Transform::from_xyz(x, y, 0.0),
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
    pub substance: Substance,
}

#[derive(Debug, Clone, Bundle)]
pub struct BallBundle {
    pub ball: Ball,
    pub sprite: Sprite,
    pub transform: Transform,
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
        images: &mut Assets<Image>,
        generated_image_assets: &mut image_gen::GeneratedImageAssets,
        substance: Substance,
        minigame: Entity,
        blocks_per_column: u32,
        blocks_per_row: u32,
    ) -> Self {
        let x = BLOCK_SIZE * ((blocks_per_row / 2) as f32 - 2.0);
        let y = -BLOCK_SIZE * (((blocks_per_column + 3) / 2) as f32 - 1.0);
        let area = CircularArea {
            radius: BLOCK_SIZE / 2.0,
        };
        let item = Item::solid(substance, BulkShape::Ball, 1.0);
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
            ball: Ball {
                substance,
                minigame,
            },
            sprite: Sprite {
                image: texture,
                custom_size: Some(area.into()),
                ..default()
            },
            transform: Transform::from_xyz(x, y, 0.0),
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
    pub substance: Substance,
    pub minigame: Entity,
}

#[derive(Debug, Clone, Bundle)]
pub struct PaddleBundle {
    pub paddle: Paddle,
    pub sprite: Sprite,
    pub transform: Transform,
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
            sprite: Sprite {
                image: asset_server.load("block_breaker/paddle.png"),
                custom_size: Some(area.into()),
                ..default()
            },
            transform: Transform::from_xyz(x, y, 0.0),
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
    let Some(click_position) = get_click_press_position(
        camera_query,
        window_query,
        mouse_button_input,
    ) else {
        return;
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
    mut images: ResMut<Assets<Image>>,
    mut generated_image_assets: ResMut<image_gen::GeneratedImageAssets>,
    mut collision_events: MessageReader<CollisionEvent>,
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
        let CollisionEvent::Started(a, b, _flags) = event else {
            continue;
        };

        // only care about collisions between balls and blocks
        let (ball_entity, block_entity, ball) =
            if let Ok(ball) = ball_query.get(*a) {
                (*a, *b, ball)
            } else if let Ok(ball) = ball_query.get(*b) {
                (*b, *a, ball)
            } else {
                continue;
            };
        let ball_substance = ball.substance;
        let minigame_entity = ball.minigame;

        let Ok(block) = block_query.get(block_entity) else {
            continue;
        };
        let block_substance = block.substance;

        if broken.contains(&block_entity) || broken.contains(&ball_entity) {
            continue;
        }

        // get minigame
        let Ok((minigame, minigame_global_transform, minigame_area)) =
            minigame_query.get_mut(minigame_entity)
        else {
            continue;
        };
        let Minigame::BallBreaker(minigame) = minigame.into_inner() else {
            continue;
        };

        // break stuff! and spit out resources!
        if BallBreakerMinigame::material_damage(ball_substance)
            >= BallBreakerMinigame::material_toughness(block_substance)
        {
            // despawn_recursive (not despawn) so the block detaches from the
            // minigame's Children list; a plain despawn leaves a stale child
            // reference that the levelup despawn_recursive later hits (B0003).
            commands.entity(block_entity).despawn();
            broken.insert(block_entity);
            commands.spawn(ItemBundle::new_from_minigame(
                &mut images,
                &mut generated_image_assets,
                Item::powder(block_substance, 1.0),
                minigame_global_transform,
                minigame_area,
            ));

            // this was the last block, so reset and level up!
            if block_query.iter().count() == 1 {
                commands.entity(minigame_entity).insert(LevelingUp);
            }
        }
        if BallBreakerMinigame::material_damage(block_substance)
            >= BallBreakerMinigame::material_toughness(ball_substance)
        {
            // despawn_recursive so the ball detaches from the minigame's
            // Children (see the block despawn above) — avoids a stale child
            // reference on levelup (B0003).
            commands.entity(ball_entity).despawn();
            broken.insert(ball_entity);
            minigame.remove_ball(ball_substance);
            commands.spawn(ItemBundle::new_from_minigame(
                &mut images,
                &mut generated_image_assets,
                Item::powder(ball_substance, 1.0),
                minigame_global_transform,
                minigame_area,
            ));
        }
    }
}
