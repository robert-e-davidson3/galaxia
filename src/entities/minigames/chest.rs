use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::entities::*;
use crate::libs::*;

pub const ID: &str = "chest";
pub const POSITION: Vec2 = Vec2::new(300.0, 150.0);

pub const NAME: &str = "chest";
pub const NAME_WITH_BAGS: &str = "chest with bags";
pub const NAME_WITH_BARRELS: &str = "barrels and chest with bags";
pub const NAME_WITH_TANKS: &str = "tanks, barrels, and chest with bags";
pub const DESCRIPTION: &str = "Store your items!";

const STORAGE_SIZE: f32 = 50.0;
const ITEMS_PER_ROW: u32 = 5;
const VISIBLE_ROWS: u32 = 3;

#[derive(Debug, Clone, Default, Component)]
pub struct ChestMinigame {
    pub level: u8,
    pub items: Arc<Mutex<HashMap<ItemType, f32>>>,
    pub inventory: Option<Entity>,
}

impl ChestMinigame {
    //
    // COMMON
    //

    pub fn name(&self) -> &str {
        match self.level {
            0..=4 => NAME,
            5..=9 => NAME_WITH_BAGS,
            10..=19 => NAME_WITH_BARRELS,
            _ => NAME_WITH_TANKS,
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
        let physical = match item.r#type {
            ItemType::Physical(data) => data,
            _ => return false,
        };

        // Level-based restrictions
        match self.level {
            0..=4 => {
                // Only solid items
                matches!(
                    physical.form,
                    PhysicalItemForm::Object
                        | PhysicalItemForm::Lump
                        | PhysicalItemForm::Block
                        | PhysicalItemForm::Ball
                ) // && !physical.material.is_goo() // TODO re-add
            }
            5..=9 => {
                // Add powders and goos
                matches!(
                    physical.form,
                    PhysicalItemForm::Object
                        | PhysicalItemForm::Lump
                        | PhysicalItemForm::Block
                        | PhysicalItemForm::Ball
                        | PhysicalItemForm::Powder
                )
            }
            10..=19 => {
                // Add liquids
                matches!(
                    physical.form,
                    PhysicalItemForm::Object
                        | PhysicalItemForm::Lump
                        | PhysicalItemForm::Block
                        | PhysicalItemForm::Ball
                        | PhysicalItemForm::Powder
                        | PhysicalItemForm::Liquid
                )
            }
            _ => {
                // All forms allowed
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
                Minigame::Chest(m) => m,
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
