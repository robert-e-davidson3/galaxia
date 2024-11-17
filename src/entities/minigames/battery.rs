use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::entities::*;
use crate::libs::*;

pub const ID: &str = "battery";
pub const POSITION: Vec2 = Vec2::new(0.0, -300.0);

pub const NAME_FIRST: &str = "spring";
pub const NAME_SECOND: &str = "spring and battery";
pub const NAME_THIRD: &str = "spring, battery, heat stone";
pub const NAME_FOURTH: &str = "tesseract";
pub const DESCRIPTION: &str = "Store your energy!";

const STORAGE_SIZE: f32 = 50.0;
const ITEMS_PER_ROW: u32 = 3;
const VISIBLE_ROWS: u32 = 3;

#[derive(Debug, Clone, Default, Component)]
pub struct BatteryMinigame {
    pub level: u8,
    pub items: Arc<Mutex<HashMap<ItemType, f32>>>,
    pub inventory: Option<Entity>,
}

impl BatteryMinigame {
    //
    // COMMON
    //

    pub fn name(&self) -> &str {
        match self.level {
            0..=9 => NAME_FIRST,
            10..=19 => NAME_SECOND,
            20..=49 => NAME_THIRD,
            _ => NAME_FOURTH,
        }
    }

    pub fn description(&self) -> &str {
        DESCRIPTION
    }

    pub fn area(&self) -> RectangularArea {
        RectangularArea {
            width: STORAGE_SIZE * ITEMS_PER_ROW as f32,
            height: STORAGE_SIZE * VISIBLE_ROWS as f32,
        }
    }

    pub fn level(&self) -> u8 {
        self.level
    }

    pub fn levelup(&self) -> Self {
        Self {
            level: self.level + 1,
            ..self.clone()
        }
    }

    pub fn spawn(
        &mut self,
        parent: &mut ChildBuilder,
        _asset_server: &AssetServer,
    ) {
        // TODO draw background chest, barrels, etc
        let inventory = InventoryBundle::spawn(
            parent,
            Inventory::new(
                parent.parent_entity(),
                Vec::new(),
                (ITEMS_PER_ROW, VISIBLE_ROWS),
                &self.items,
            ),
            Vec2::ZERO,
            self.area().into(),
        );
        self.inventory = Some(inventory);
    }

    pub fn ingest_item(
        &mut self,
        commands: &mut Commands,
        minigame_entity: Entity,
        item: &Item,
    ) -> f32 {
        let added = if self.can_accept(item) {
            add_item(&self.items, item.r#type, item.amount);
            item.amount
        } else {
            return 0.0; // Reject the item
        };

        // Poke Inventory so it redraws
        mark_component_changed::<Inventory>(commands, self.inventory.unwrap());

        // Level up if needed
        if total_stored(&self.items) > self.capacity() {
            commands.entity(minigame_entity).insert(LevelingUp);
        }

        added
    }

    //
    // SPECIFIC
    //

    pub fn capacity(&self) -> f32 {
        2.0f32.powi(self.level as i32)
    }

    pub fn can_accept(&self, item: &Item) -> bool {
        let energy = match item.r#type {
            ItemType::Energy(data) => data,
            _ => return false,
        };

        // Level-based restrictions
        match self.level {
            0..=9 => {
                // Spring - only kinetic
                matches!(energy.kind, EnergyKind::Kinetic)
            }
            10..=19 => {
                // Spring and battery - kinetic and electric
                matches!(
                    energy.kind,
                    EnergyKind::Kinetic | EnergyKind::Electric
                )
            }
            20..=49 => {
                // Spring, battery, heat stone - kinetic, electric, thermal
                matches!(
                    energy.kind,
                    EnergyKind::Kinetic
                        | EnergyKind::Electric
                        | EnergyKind::Thermal
                )
            }
            _ => {
                // Tesseract - all
                true
            }
        }
    }
}
