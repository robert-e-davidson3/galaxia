use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;
use std::collections::HashSet;

use crate::entities::*;
use crate::libs::*;

pub const NAME: &str = "Primordial Ocean";
pub const _DESCRIPTION: &str = "Infinitely deep, the source of water and mud.";

const BASE_SIZE: f32 = 120.0;
const MAX_SIZE_MULTIPLIER: f32 = 2.0;

#[derive(Debug, Clone, Bundle)]
pub struct PrimordialOceanMinigameBundle {
    pub minigame: PrimordialOceanMinigame,
    pub area: RectangularArea,
    pub tag: Minigame,
    pub spatial: SpatialBundle,
}

impl PrimordialOceanMinigameBundle {
    pub fn new(
        minigame: PrimordialOceanMinigame,
        radius: f32,
        transform: Transform,
    ) -> Self {
        let area = RectangularArea::new_square(radius * 2.0);
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

#[derive(Debug, Clone, Component)]
pub struct PrimordialOceanMinigame {
    pub size: f32,
    pub level: u8,
    pub salt_water_collected: f32,
}

impl Default for PrimordialOceanMinigame {
    fn default() -> Self {
        Self {
            size: BASE_SIZE,
            level: 0,
            salt_water_collected: 0.0,
        }
    }
}

impl PrimordialOceanMinigame {
    pub fn new(salt_water_collected: f32) -> Self {
        let level = Self::level_by_salt_water_collected(salt_water_collected);
        let size_multiplier =
            1.0 + (level as f32 / 99.0) * (MAX_SIZE_MULTIPLIER - 1.0);
        Self {
            size: BASE_SIZE * size_multiplier,
            level,
            salt_water_collected,
        }
    }

    pub fn level_by_salt_water_collected(salt_water_collected: f32) -> u8 {
        if salt_water_collected <= 0.0 {
            0
        } else {
            ((salt_water_collected.log2() + 1.0) as u8).min(99)
        }
    }

    pub fn should_level_up(&self) -> bool {
        if self.level == 99 {
            false
        } else {
            Self::level_by_salt_water_collected(self.salt_water_collected)
                > self.level
        }
    }

    pub fn item_is_valid(item: &Item) -> bool {
        let physical = match item.as_physical() {
            Some(data) => data,
            None => return false,
        };

        matches!(physical.material, PhysicalItemMaterial::SaltWater)
    }
}

pub fn spawn(
    commands: &mut Commands,
    transform: Transform,
    minigame: PrimordialOceanMinigame,
) {
    let radius = minigame.size;
    let level = minigame.level;
    let area = RectangularArea::new_square(radius * 2.0);
    commands
        .spawn(PrimordialOceanMinigameBundle::new(
            minigame, radius, transform,
        ))
        .with_children(|parent| {
            parent.spawn(MinigameAuraBundle::new(parent.parent_entity(), area));
            spawn_minigame_container(parent, area, NAME, level);
            parent.spawn(OceanBundle::new(parent.parent_entity(), radius));
        });
}

#[derive(Bundle)]
pub struct OceanBundle {
    pub ocean: Ocean,
    pub area: CircularArea,
    pub shape: ShapeBundle,
    pub fill: Fill,
}

impl OceanBundle {
    pub fn new(minigame: Entity, radius: f32) -> Self {
        let area = CircularArea::new(radius);
        Self {
            ocean: Ocean { minigame },
            area,
            shape: ShapeBundle {
                path: GeometryBuilder::build_as(&shapes::Circle {
                    radius,
                    ..default()
                }),
                ..default()
            },
            fill: Fill::color(Color::srgb(0.0, 0.25, 1.0)),
        }
    }
}

#[derive(Debug, Clone, Component)]
pub struct Ocean {
    pub minigame: Entity,
}

pub fn ingest_resource_fixed_update(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut minigame_query: Query<(Entity, &mut PrimordialOceanMinigame)>,
    aura_query: Query<&MinigameAura>,
    item_query: Query<&Item>,
    leveling_up_query: Query<&LevelingUp, With<PrimordialOceanMinigame>>,
) {
    let mut ingested: HashSet<Entity> = HashSet::new();
    for event in collision_events.read() {
        // only care about collision start
        let (a, b) = match event {
            CollisionEvent::Started(a, b, _flags) => (a, b),
            _ => continue,
        };

        // only care about collisions of resources
        let (&item_entity, aura_entity, item) = match item_query.get(*a) {
            Ok(item) => (a, b, item),
            Err(_) => match item_query.get(*b) {
                Ok(item) => (b, a, item),
                Err(_) => continue,
            },
        };

        // already handled
        if ingested.contains(&item_entity) {
            continue;
        }

        // only certain resources can be ingested
        if !PrimordialOceanMinigame::item_is_valid(item) {
            continue;
        }

        // only care about collisions of resources with minigame auras
        let aura = match aura_query.get(*aura_entity) {
            Ok(x) => x,
            Err(_) => continue,
        };

        // Skip if currently leveling up
        if leveling_up_query.get(aura.minigame).is_ok() {
            continue;
        }

        // Get minigame state
        let (minigame_entity, mut minigame) =
            match minigame_query.get_mut(aura.minigame) {
                Ok(x) => x,
                Err(_) => continue,
            };

        // Mark as ingested and remove the item
        commands.entity(item_entity).despawn_recursive();
        ingested.insert(item_entity);

        // Track the collected water
        minigame.salt_water_collected += item.amount;

        // Check for level up
        if minigame.should_level_up() {
            commands.entity(minigame_entity).insert(LevelingUp {
                minigame: minigame_entity,
            });
        }
    }
}

pub fn update(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut generated_image_assets: ResMut<image_gen::GeneratedImageAssets>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mouse_state: Res<MouseState>,
    time: Res<Time>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    window_query: Query<&Window>,
    primordial_ocean_minigame_query: Query<
        (&GlobalTransform, &RectangularArea),
        With<PrimordialOceanMinigame>,
    >,
    mut ocean_query: Query<(&Ocean, &GlobalTransform, &CircularArea)>,
    leveling_up_query: Query<&LevelingUp, With<PrimordialOceanMinigame>>,
) {
    let click_position = match get_click_release_position(
        camera_query,
        window_query,
        mouse_button_input,
    ) {
        Some(position) => position,
        None => return,
    };

    for (ocean, ocean_transform, ocean_area) in ocean_query.iter_mut() {
        let minigame_entity = ocean.minigame;

        // Skip if currently leveling up
        if leveling_up_query.get(minigame_entity).is_ok() {
            continue;
        }

        if ocean_area
            .is_within(click_position, ocean_transform.translation().truncate())
        {
            let (minigame_transform, minigame_area) =
                primordial_ocean_minigame_query
                    .get(minigame_entity)
                    .unwrap();
            let click_type = mouse_state.get_click_type(time.elapsed_seconds());
            let (form, material) = match click_type {
                ClickType::Short => {
                    (PhysicalItemForm::Liquid, PhysicalItemMaterial::SaltWater)
                }
                ClickType::Long => {
                    (PhysicalItemForm::Block, PhysicalItemMaterial::Mud)
                }
                ClickType::Invalid => {
                    println!("unexpected: invalid click type");
                    continue;
                }
            };
            commands.spawn(ItemBundle::new_from_minigame(
                &mut images,
                &mut generated_image_assets,
                Item::new_physical(form, material, 1.0),
                minigame_transform,
                minigame_area,
            ));
        }
    }
}

pub fn levelup(
    mut commands: Commands,
    primordial_ocean_minigame_query: Query<
        (&PrimordialOceanMinigame, Entity, &Transform),
        With<LevelingUp>,
    >,
) {
    for (minigame, entity, transform) in primordial_ocean_minigame_query.iter()
    {
        println!("Leveling up Primordial Ocean Minigame");
        println!("Salt water collected: {}", minigame.salt_water_collected);
        commands.entity(entity).despawn_recursive();
        spawn(
            &mut commands,
            transform.clone(),
            PrimordialOceanMinigame::new(minigame.salt_water_collected),
        );
    }
}
