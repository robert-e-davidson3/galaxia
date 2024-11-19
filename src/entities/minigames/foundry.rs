use std::collections::VecDeque;

use bevy::prelude::*;

use crate::entities::*;
use crate::libs::*;

// Changes items under the vague notion of transmutation through heating.
// This works for physical items like metals but also abstract items.
// Collects Heat Energy for physical transmutation but creates Heat Energy
// when fed Clicks.
// Levels up as more items are transmuted.

pub const ID: &str = "foundry";
pub const POSITION: Vec2 = Vec2::new(0.0, 500.0);

pub const NAME: &str = "Foundry";
pub const DESCRIPTION: &str = "Transmute items through heat.";
const AREA: RectangularArea = RectangularArea {
    width: 150.0,
    height: 150.0,
};

#[derive(Debug, Clone, Default, Component)]
pub struct FoundryMinigame {
    pub level: u8,
    pub heat: f32,
    pub cooking: VecDeque<Item>,
    pub special_cooking: VecDeque<Item>, // clicks
    pub last_cook: f32,
    pub total_cooked: f32,
}

impl FoundryMinigame {
    pub fn new(
        total_cooked: f32,
        heat: f32,
        cooking: VecDeque<Item>,
        special_cooking: VecDeque<Item>,
    ) -> Self {
        Self {
            level: Self::level_by_total_cooked(total_cooked),
            heat,
            cooking,
            special_cooking,
            last_cook: 0.0,
            total_cooked,
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
        AREA
    }

    pub fn level(&self) -> u8 {
        self.level
    }

    pub fn levelup(&self) -> Self {
        Self::new(
            self.total_cooked,
            self.heat,
            self.cooking.clone(),
            self.special_cooking.clone(),
        )
    }

    pub fn spawn(&self, _parent: &mut ChildBuilder) {
        // TODO background
        // TODO heat meter
        // TODO transmutation timer
    }

    pub fn ingest_item(&mut self, item: &Item) -> f32 {
        match item.r#type {
            // Keep heat
            ItemType::Energy(energy) => match energy.kind {
                EnergyKind::Thermal => {
                    self.heat += item.amount;
                    item.amount
                }
                _ => 0.0,
            },
            // Special cooking (priority)
            ItemType::Abstract(abstraction) => match abstraction.kind {
                AbstractKind::Click => {
                    self.special_cooking.push_back(item.clone());
                    item.amount
                }
                _ => 0.0,
            },
            // Regular cooking
            ItemType::Physical(physical) => match physical.form {
                PhysicalForm::Ore => {
                    self.cooking.push_back(item.clone());
                    item.amount
                }
                _ => 0.0,
            },
            _ => 0.0,
        }
    }

    //
    // SPECIFIC
    //

    pub fn level_by_total_cooked(total_cooked: f32) -> u8 {
        if total_cooked <= 0.0 {
            0
        } else {
            ((total_cooked.log2() + 1.0) as u8).min(99)
        }
    }

    pub fn transmute(item_type: ItemType) -> ItemType {
        match item_type {
            ItemType::Abstract(abstraction) => match abstraction.kind {
                AbstractKind::Click => {
                    let kind = match abstraction.variant {
                        0 => EnergyKind::Thermal,
                        1 => EnergyKind::Kinetic,
                        _ => panic!("Unknown Click variant"),
                    };
                    ItemType::Energy(EnergyItem { kind })
                }
                _ => item_type,
            },
            ItemType::Physical(physical) => match physical.form {
                PhysicalForm::Ore => ItemType::Physical(PhysicalItem {
                    form: PhysicalForm::Liquid,
                    material: physical.material,
                }),
                _ => item_type,
            },
            _ => item_type,
        }
    }
}

const COOK_PERIOD_SECONDS: f32 = 1.0;

pub fn cook_fixed_update(
    mut commands: Commands,
    time: Res<Time>,
    mut images: ResMut<Assets<Image>>,
    mut generated_image_assets: ResMut<image_gen::GeneratedImageAssets>,
    mut query: Query<(
        &mut Minigame,
        &GlobalTransform,
        &RectangularArea,
        Entity,
    )>,
) {
    for (minigame, minigame_transform, minigame_area, minigame_entity) in
        query.iter_mut()
    {
        let minigame = match minigame.into_inner() {
            Minigame::Foundry(minigame) => minigame,
            _ => continue,
        };
        if minigame.last_cook == 0.0 {
            minigame.last_cook = time.elapsed_seconds();
        } else if minigame.last_cook + time.elapsed_seconds()
            >= COOK_PERIOD_SECONDS
        {
            // first try priority cooking
            if let Some(special) = minigame.special_cooking.pop_front() {
                commands.spawn(ItemBundle::new_from_minigame(
                    &mut images,
                    &mut generated_image_assets,
                    FoundryMinigame::transmute(special.r#type)
                        .to_item(special.amount),
                    minigame_transform,
                    minigame_area,
                ));
                minigame.last_cook = time.elapsed_seconds();

                return;
            }

            // remove first item in cooking, map, emit
            let raw = match minigame.cooking.pop_front() {
                Some(raw) => raw,
                None => continue,
            };
            minigame.last_cook = time.elapsed_seconds();

            commands.spawn(ItemBundle::new_from_minigame(
                &mut images,
                &mut generated_image_assets,
                FoundryMinigame::transmute(raw.r#type).to_item(raw.amount),
                minigame_transform,
                minigame_area,
            ));

            // update total cooked
            minigame.total_cooked += raw.amount;
            // level up
            let level =
                FoundryMinigame::level_by_total_cooked(minigame.total_cooked);
            if level > minigame.level {
                commands.entity(minigame_entity).insert(LevelingUp);
            }
        }
    }
}
