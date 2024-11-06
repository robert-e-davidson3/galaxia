use bevy::prelude::*;

use crate::entities::*;
use crate::libs::*;

pub const NAME: &str = "Tree";
pub const _DESCRIPTION: &str = "Pick fruits from the tree!";
const AREA: RectangularArea = RectangularArea {
    width: 300.0,
    height: 300.0,
};

#[derive(Debug, Default, Bundle)]
pub struct TreeMinigameBundle {
    pub minigame: TreeMinigame,
    pub area: RectangularArea,
    pub tag: Minigame,
    pub sprite: SpriteBundle,
}

impl TreeMinigameBundle {
    pub fn new(minigame: TreeMinigame, sprite: SpriteBundle) -> Self {
        Self {
            minigame,
            area: AREA,
            tag: Minigame,
            sprite,
        }
    }
}

#[derive(Debug, Clone, Component)]
pub struct TreeMinigame {
    pub fruit: PhysicalItemMaterial,
    pub count: u32,
    pub _lushness: f32,
    pub last_fruit_time: f32,
}

impl Default for TreeMinigame {
    fn default() -> Self {
        Self {
            fruit: PhysicalItemMaterial::Apple,
            count: 0,
            _lushness: 1.0,
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
    commands
        .spawn(TreeMinigameBundle::new(
            frozen.clone(),
            SpriteBundle {
                texture: asset_server
                    .load("oak-tree-white-background-300x300.png"),
                sprite: Sprite {
                    color: Color::srgba(1.0, 1.0, 1.0, 1.0),
                    custom_size: Some(Vec2::new(AREA.width, AREA.height)),
                    ..default()
                },
                transform: transform.clone(),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn(MinigameAuraBundle::new(parent.parent_entity(), AREA));
            spawn_minigame_container(parent, AREA, NAME);
        });
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
        &mut TreeMinigame,
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
            let (mut minigame, minigame_transform, minigame_area) =
                tree_minigames_query.get_mut(fruit.minigame).unwrap();
            minigame.count -= 1;
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
        if elapsed_seconds - minigame.last_fruit_time <= needed_time_seconds {
            continue;
        }
        minigame.last_fruit_time = elapsed_seconds;
        minigame.count += 1;
        commands.entity(entity).with_children(|parent| {
            parent.spawn(UnpickedFruitBundle::new(
                &asset_server,
                entity,
                minigame.fruit,
                Transform::from_xyz(0.0, 0.0, 0.0),
            ));
        });
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
                // adjust by Z only
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
