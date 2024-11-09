use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use crate::entities::*;
use crate::libs::*;

pub const NAME: &str = "Tree";
pub const DESCRIPTION: &str = "Pick fruits from the tree!";
const AREA: RectangularArea = RectangularArea {
    width: 300.0,
    height: 300.0,
};

#[derive(Debug, Clone, Component)]
pub struct TreeMinigame {
    pub fruit: PhysicalItemMaterial,
    pub count: u32,
    pub _lushness: f32,
    pub last_fruit_time: f32,
    pub level: u8,
}

impl Default for TreeMinigame {
    fn default() -> Self {
        Self {
            fruit: PhysicalItemMaterial::Apple,
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

    pub fn spawn(&self, parent: &mut ChildBuilder, asset_server: &AssetServer) {
        parent.spawn(SpriteBundle {
            texture: asset_server.load("oak-tree-white-background-300x300.png"),
            sprite: Sprite {
                color: Color::srgba(1.0, 1.0, 1.0, 1.0),
                custom_size: Some(Vec2::new(AREA.width, AREA.height)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        });
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
    pub sprite: SpriteBundle,
}

impl UnpickedFruitBundle {
    pub fn new(
        asset_server: &AssetServer,
        minigame: Entity,
        fruit: PhysicalItemMaterial,
        transform: Transform,
    ) -> Self {
        let area = CircularArea { radius: 8.0 };
        Self {
            unpicked_fruit: UnpickedFruit {
                material: fruit,
                minigame,
            },
            area,
            sprite: SpriteBundle {
                texture: asset_server.load(
                    Item::new_physical(PhysicalItemForm::Object, fruit, 1.0)
                        .asset(),
                ),
                transform: Transform::from_xyz(
                    transform.translation.x,
                    transform.translation.y,
                    1.0,
                ),
                ..default()
            },
        }
    }
}

#[derive(Debug, Clone, Component)]
pub struct UnpickedFruit {
    pub material: PhysicalItemMaterial,
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
    let click_position = match get_click_release_position(
        camera_query,
        window_query,
        mouse_button_input,
    ) {
        Some(world_position) => world_position,
        None => return,
    };

    for (entity, fruit, global_transform, area) in clickable_query.iter() {
        if area.is_within(
            click_position,
            global_transform.translation().truncate(),
        ) {
            commands.entity(entity).despawn();
            let (minigame, minigame_transform, minigame_area) =
                tree_minigames_query.get_mut(fruit.minigame).unwrap();

            if let Minigame::Tree(tree_minigame) = minigame.into_inner() {
                tree_minigame.remove_fruit();

                commands.spawn(ItemBundle::new_from_minigame(
                    &mut images,
                    &mut generated_image_assets,
                    Item::new_physical(
                        PhysicalItemForm::Object,
                        fruit.material,
                        1.0,
                    ),
                    minigame_transform,
                    minigame_area,
                ));
            }
        }
    }
}

// Grow fruits periodically
pub fn fixed_update(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut minigame_query: Query<(Entity, &mut Minigame)>,
    leveling_up_query: Query<&LevelingUp>,
) {
    for (entity, minigame) in minigame_query.iter_mut() {
        // Skip if leveling up
        if leveling_up_query.get(entity).is_ok() {
            continue;
        }
        let tree_minigame =
            if let Minigame::Tree(tree_minigame) = minigame.into_inner() {
                tree_minigame
            } else {
                continue;
            };

        let max_fruit = 1 + (tree_minigame.level / 10) as u32;
        if tree_minigame.count >= max_fruit {
            continue;
        }

        let needed_time_seconds =
            5.0 - (tree_minigame.level as f32 * 0.05).min(4.0);
        let elapsed_seconds = time.elapsed_seconds();

        if elapsed_seconds - tree_minigame.last_fruit_time
            <= needed_time_seconds
        {
            continue;
        }

        tree_minigame.last_fruit_time = elapsed_seconds;
        tree_minigame.add_fruit();

        commands.entity(entity).with_children(|parent| {
            parent.spawn(UnpickedFruitBundle::new(
                &asset_server,
                entity,
                tree_minigame.fruit,
                Transform::from_xyz(0.0, 0.0, 0.0),
            ));
        });
    }
}
