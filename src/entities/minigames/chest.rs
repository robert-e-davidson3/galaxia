use std::collections::HashMap;

use bevy::prelude::*;

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
    pub items: HashMap<ItemType, f32>,
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
        parent: &mut ChildSpawnerCommands,
        _asset_server: &AssetServer,
    ) {
        // TODO draw background chest, barrels, etc
        let inventory = InventoryBundle::spawn(
            parent,
            Inventory::new(
                parent.target_entity(),
                Vec::new(),
                (ITEMS_PER_ROW, VISIBLE_ROWS),
            ),
            &self.items,
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
        if !self.can_accept(item) {
            return 0.0; // Reject the item
        }
        add_item(&mut self.items, item.r#type, item.amount);
        let added = item.amount;

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
        let ItemType::Physical(_) = item.r#type else {
            return false;
        };

        // Fruit is a basic harvest; always storable.
        if item.r#type.is_fruit() {
            return true;
        }

        let is_powder = matches!(
            item.r#type,
            ItemType::Physical(PhysicalItem::Bulk(BulkItem {
                structure: BulkStructure::Powder,
                ..
            }))
        );
        let is_liquid = matches!(
            item.r#type,
            ItemType::Physical(PhysicalItem::Bulk(BulkItem {
                structure: BulkStructure::Liquid,
                ..
            }))
        );

        // Level-based restrictions
        match self.level {
            0..=4 => item.r#type.is_solid(),
            5..=9 => item.r#type.is_solid() || is_powder,
            10..=19 => item.r#type.is_solid() || is_powder || is_liquid,
            _ => true, // all forms allowed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Leveling up must carry the stored items forward: the minigame entity
    // (and its inventory entity) is despawned and respawned on levelup, so the
    // item store has to live on the persistent struct, not the inventory entity.
    #[test]
    fn levelup_preserves_stored_items() {
        let mut chest = ChestMinigame::default();
        let stored = Item::solid(Substance::Iron, BulkShape::Block, 4.0);
        add_item(&mut chest.items, stored.r#type, stored.amount);

        let leveled = chest.levelup();

        assert_eq!(leveled.level, 1);
        assert_eq!(total_stored(&leveled.items), 4.0);
    }

    // Tree fruit (Apple) must be storable even in a level-0 chest, which
    // otherwise only accepts solid lumps/blocks/balls.
    #[test]
    fn chest_accepts_fruit_at_level_zero() {
        let chest = ChestMinigame::default();
        let apple = Item::fruit(Species::Apple, 1.0);
        assert!(chest.can_accept(&apple));
    }
}
