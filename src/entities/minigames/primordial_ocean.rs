use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use crate::entities::*;
use crate::libs::*;

pub const ID: &str = "primordial_ocean";
pub const POSITION: Vec2 = Vec2::new(250.0, -200.0);

pub const NAME: &str = "Primordial Ocean";
pub const DESCRIPTION: &str = "Infinitely deep, the source of water and mud.";

const BASE_SIZE: f32 = 60.0;
const MAX_SIZE_MULTIPLIER: f32 = 2.0;

#[derive(Debug, Clone, Component)]
pub struct PrimordialOceanMinigame {
    pub radius: f32,
    pub level: u8,
    pub salt_water_collected: f32,
}

impl Default for PrimordialOceanMinigame {
    fn default() -> Self {
        Self {
            radius: BASE_SIZE,
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
            radius: BASE_SIZE * size_multiplier,
            level,
            salt_water_collected,
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
        RectangularArea::new_square(self.radius * 2.0)
    }

    pub fn level(&self) -> u8 {
        self.level
    }

    pub fn levelup(&self) -> Self {
        Self::new(self.salt_water_collected)
    }

    pub fn spawn(&self, parent: &mut ChildBuilder) {
        let radius = self.radius;
        parent.spawn(OceanBundle::new(parent.parent_entity(), radius));
    }

    pub fn ingest_item(
        &mut self,
        commands: &mut Commands,
        minigame_entity: Entity,
        item: &Item,
    ) -> f32 {
        if !Self::item_is_valid(item) {
            return 0.0;
        }

        self.salt_water_collected += item.amount;

        if self.should_level_up() {
            commands.entity(minigame_entity).insert(LevelingUp);
        }

        item.amount
    }

    //
    // SPECIFIC
    //

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
        let physical = match item.r#type {
            ItemType::Physical(data) => data,
            _ => return false,
        };

        matches!(physical.material, PhysicalItemMaterial::SaltWater)
    }
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

pub fn update(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut generated_image_assets: ResMut<image_gen::GeneratedImageAssets>,
    mouse_state: Res<MouseState>,
    minigame_query: Query<(&GlobalTransform, &RectangularArea), With<Minigame>>,
    mut ocean_query: Query<(&Ocean, &GlobalTransform, &CircularArea)>,
    leveling_up_query: Query<&LevelingUp, With<Minigame>>,
) {
    if !mouse_state.just_released {
        return;
    }
    let click_position = mouse_state.current_position;

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
                minigame_query.get(minigame_entity).unwrap();
            let click_type = mouse_state.get_click_type();
            let (form, material) = match click_type {
                ClickType::Short => {
                    (PhysicalItemForm::Liquid, PhysicalItemMaterial::SaltWater)
                }
                ClickType::Long => {
                    (PhysicalItemForm::Lump, PhysicalItemMaterial::Mud)
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
