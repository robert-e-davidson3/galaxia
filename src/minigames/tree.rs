use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::area::*;
use crate::collision::*;
use crate::common::*;
use crate::mouse::*;
use crate::resource::*;

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
    pub aura: Collider,
    pub collision_groups: CollisionGroups,
    pub active_events: ActiveEvents,
    pub sprite: SpriteBundle,
}

impl TreeMinigameBundle {
    pub fn new(minigame: TreeMinigame, sprite: SpriteBundle) -> Self {
        Self {
            minigame,
            area: AREA,
            tag: Minigame,
            aura: AREA.grow(1.0, 1.0).into(),
            collision_groups: CollisionGroups::new(
                MINIGAME_AURA_GROUP,
                minigame_aura_filter(),
            ),
            active_events: ActiveEvents::COLLISION_EVENTS,
            sprite,
        }
    }
}

#[derive(Debug, Clone, Component)]
pub struct TreeMinigame {
    pub fruit: GalaxiaResource,
    pub count: u32,
    pub _lushness: f32,
    pub last_fruit_time: f32,
}

impl Default for TreeMinigame {
    fn default() -> Self {
        Self {
            fruit: GalaxiaResource::Apple,
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
            spawn_minigame_container(parent, AREA, NAME);
        });
}

// When a fruit is clicked, replace it with a fruit resource.
pub fn update(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
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
            commands.spawn(LooseResourceBundle::new_from_minigame(
                &asset_server,
                fruit.resource,
                1.0,
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
        ))
        .set_parent(parent);
}

#[derive(Debug, Clone, Component)]
pub struct UnpickedFruit {
    pub resource: GalaxiaResource,
    pub minigame: Entity,
}
