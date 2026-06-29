use bevy::prelude::*;

use crate::entities::*;
use crate::libs::*;

pub const ID: &str = "tree";
pub const POSITION: Vec2 = Vec2::new(-350.0, 250.0);

pub const NAME: &str = "Tree";
pub const DESCRIPTION: &str = "Pick fruits from the tree!";
const AREA: RectangularArea = RectangularArea {
    width: 300.0,
    height: 300.0,
};

// Apples grow within the leafy crown (the upper-center of the 300x300 sprite),
// not on the trunk. Coordinates are local to the tree's center.
const CANOPY_MIN: Vec2 = Vec2::new(-95.0, -20.0);
const CANOPY_MAX: Vec2 = Vec2::new(95.0, 105.0);
const FRUIT_RADIUS: f32 = 8.0;
// Centers at least this far apart so the fruit sprites don't overlap.
const FRUIT_SPACING: f32 = FRUIT_RADIUS * 2.0 + 4.0;

#[derive(Debug, Clone, Component)]
pub struct TreeMinigame {
    pub fruit: Species,
    pub count: u32,
    pub _lushness: f32,
    pub last_fruit_time: f32,
    pub level: u8,
}

impl Default for TreeMinigame {
    fn default() -> Self {
        Self {
            fruit: Species::Apple,
            count: 0,
            _lushness: 1.0,
            last_fruit_time: 0.0,
            level: 0,
        }
    }
}

impl TreeMinigame {
    pub fn new(level: u8) -> Self {
        Self { level, ..default() }
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
        AREA
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
        asset_server: &AssetServer,
    ) {
        parent.spawn((
            Sprite {
                image: asset_server.load("oak-tree-white-background-300x300.png"),
                color: Color::srgba(1.0, 1.0, 1.0, 1.0),
                custom_size: Some(Vec2::new(AREA.width, AREA.height)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));
    }

    pub fn ingest_item(&mut self) -> f32 {
        0.0 // does not ingest items
    }

    //
    // SPECIFIC
    //

    pub fn add_fruit(&mut self) {
        self.count += 1;
    }

    pub fn remove_fruit(&mut self) {
        if self.count > 0 {
            self.count -= 1;
        }
    }
}

#[derive(Bundle)]
pub struct UnpickedFruitBundle {
    pub unpicked_fruit: UnpickedFruit,
    pub area: CircularArea,
    pub sprite: Sprite,
    pub transform: Transform,
}

impl UnpickedFruitBundle {
    pub fn new(
        asset_server: &AssetServer,
        minigame: Entity,
        fruit: Species,
        transform: Transform,
    ) -> Self {
        let area = CircularArea {
            radius: FRUIT_RADIUS,
        };
        Self {
            unpicked_fruit: UnpickedFruit {
                form: fruit,
                minigame,
            },
            area,
            sprite: Sprite {
                image: asset_server
                    .load(Item::fruit(fruit, 1.0).asset()),
                ..default()
            },
            transform: Transform::from_xyz(
                transform.translation.x,
                transform.translation.y,
                1.0,
            ),
        }
    }
}

#[derive(Debug, Clone, Component)]
pub struct UnpickedFruit {
    pub form: Species,
    pub minigame: Entity,
}

// When a fruit is clicked, replace it with a fruit resource.
pub fn update(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut generated_image_assets: ResMut<image_gen::GeneratedImageAssets>,
    clickable_query: Query<(
        Entity,
        &UnpickedFruit,
        &GlobalTransform,
        &CircularArea,
    )>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    window_query: Query<&Window>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut tree_minigames_query: Query<(
        &mut Minigame,
        &GlobalTransform,
        &RectangularArea,
    )>,
) {
    let Some(click_position) = get_click_release_position(
        camera_query,
        window_query,
        mouse_button_input,
    ) else {
        return;
    };

    for (entity, fruit, global_transform, area) in clickable_query.iter() {
        if area.is_within(
            click_position,
            global_transform.translation().truncate(),
        ) {
            // despawn_recursive so the fruit detaches from the tree minigame's
            // Children list; a plain despawn leaves a stale child reference that
            // the levelup despawn_recursive later hits (B0003).
            commands.entity(entity).despawn();
            let (minigame, minigame_transform, minigame_area) =
                tree_minigames_query.get_mut(fruit.minigame).unwrap();

            if let Minigame::Tree(tree_minigame) = minigame.into_inner() {
                tree_minigame.remove_fruit();

                commands.spawn(ItemBundle::new_from_minigame(
                    &mut images,
                    &mut generated_image_assets,
                    Item::fruit(fruit.form, 1.0),
                    minigame_transform,
                    minigame_area,
                ));
            }
        }
    }
}

// Pick a spot in the canopy that doesn't overlap existing fruit. Best-effort:
// after a fixed number of tries it returns the last candidate rather than
// looping forever (the canopy can legitimately fill up).
fn random_canopy_position(random: &mut Random, existing: &[Vec2]) -> Vec2 {
    let mut candidate = Vec2::ZERO;
    for _ in 0..24 {
        let fx = (random.next() % 10_000) as f32 / 10_000.0;
        let fy = (random.next() % 10_000) as f32 / 10_000.0;
        candidate = Vec2::new(
            CANOPY_MIN.x + fx * (CANOPY_MAX.x - CANOPY_MIN.x),
            CANOPY_MIN.y + fy * (CANOPY_MAX.y - CANOPY_MIN.y),
        );
        if existing.iter().all(|p| p.distance(candidate) >= FRUIT_SPACING) {
            return candidate;
        }
    }
    candidate
}

// Grow fruits periodically
pub fn fixed_update(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut random: ResMut<Random>,
    mut minigame_query: Query<(Entity, &mut Minigame)>,
    leveling_up_query: Query<&LevelingUp>,
    fruit_query: Query<(&UnpickedFruit, &Transform)>,
) {
    for (entity, minigame) in minigame_query.iter_mut() {
        // Skip if leveling up
        if leveling_up_query.get(entity).is_ok() {
            continue;
        }
        let Minigame::Tree(tree_minigame) = minigame.into_inner() else {
            continue;
        };

        let max_fruit = 1 + (tree_minigame.level / 10) as u32;
        if tree_minigame.count >= max_fruit {
            continue;
        }

        let needed_time_seconds =
            5.0 - (tree_minigame.level as f32 * 0.05).min(4.0);
        let elapsed_seconds = time.elapsed_secs();

        if elapsed_seconds - tree_minigame.last_fruit_time
            <= needed_time_seconds
        {
            continue;
        }

        tree_minigame.last_fruit_time = elapsed_seconds;
        tree_minigame.add_fruit();
        let fruit = tree_minigame.fruit;

        // Scatter the new fruit across the canopy, clear of the others.
        let existing: Vec<Vec2> = fruit_query
            .iter()
            .filter(|(unpicked, _)| unpicked.minigame == entity)
            .map(|(_, transform)| transform.translation.truncate())
            .collect();
        let position = random_canopy_position(&mut random, &existing);

        commands.entity(entity).with_children(|parent| {
            parent.spawn(UnpickedFruitBundle::new(
                &asset_server,
                entity,
                fruit,
                Transform::from_xyz(position.x, position.y, 0.0),
            ));
        });
    }
}
