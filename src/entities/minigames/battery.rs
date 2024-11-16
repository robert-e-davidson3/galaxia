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

pub fn ingest_resource_fixed_update(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut minigame_query: Query<&mut Minigame>,
    aura_query: Query<&MinigameAura>,
    item_query: Query<&Item>,
    mut inventory_query: Query<&mut Inventory>,
) {
    let mut ingested: HashSet<Entity> = HashSet::new();
    for event in collision_events.read() {
        let (item_entity, aura_entity, item) = match event {
            CollisionEvent::Started(e1, e2, _) => match item_query.get(*e1) {
                Ok(item) => (*e1, *e2, item),
                Err(_) => match item_query.get(*e2) {
                    Ok(item) => (*e2, *e1, item),
                    Err(_) => continue,
                },
            },
            _ => continue,
        };

        if ingested.contains(&item_entity) {
            continue;
        }

        // Get the minigame
        let aura = match aura_query.get(aura_entity) {
            Ok(x) => x,
            Err(_) => continue,
        };
        let minigame = match minigame_query.get_mut(aura.minigame) {
            Ok(x) => match x.into_inner() {
                Minigame::Battery(m) => m,
                _ => continue,
            },
            Err(_) => continue,
        };

        if !minigame.can_accept(&item) {
            continue;
        }

        // add item
        match minigame.inventory {
            Some(inventory_entity) => {
                let mut inventory =
                    inventory_query.get_mut(inventory_entity).unwrap();
                inventory.page = inventory.page; // mark inventory as changed
                add_item(&inventory.items, item.r#type, item.amount);
            }
            None => panic!("Minigame has no inventory"),
        }

        if total_stored(&minigame.items) >= minigame.capacity() {
            commands.entity(aura.minigame).insert(LevelingUp);
        }

        commands.entity(item_entity).despawn();
        ingested.insert(item_entity);
    }
}
