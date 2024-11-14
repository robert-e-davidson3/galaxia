use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use wyrand::WyRand;

use crate::entities::item::*;
use crate::entities::minigame::*;
use crate::libs::*;

#[derive(Debug, Clone, Bundle)]
pub struct InventoryBundle {
    pub inventory: Inventory,
    pub spatial: SpatialBundle,
}

impl InventoryBundle {
    pub fn new(inventory: Inventory, position: Vec2) -> Self {
        let (x, y) = position.into();
        InventoryBundle {
            inventory,
            spatial: SpatialBundle {
                transform: Transform::from_translation(Vec3::new(x, y, 0.0)),
                ..default()
            },
        }
    }

    pub fn spawn(
        parent: &mut ChildBuilder,
        images: &mut Assets<Image>,
        generated_image_assets: &mut image_gen::GeneratedImageAssets,
        mut inventory: Inventory,
        position: Vec2,
        inventory_size: Vec2,
    ) -> Entity {
        let (width, height) = inventory.dimensions;
        let slot_size = Vec2::new(
            inventory_size.x / width as f32,
            inventory_size.y / height as f32,
        );
        let items = filter_items(
            &inventory.items,
            inventory.filter.clone(),
            (width * height) as usize,
            0,
        );
        let inventory_area =
            RectangularArea::new(inventory_size.x, inventory_size.y);
        parent
            .spawn_empty()
            .with_children(|parent| {
                let inventory_entity = parent.parent_entity();
                let mut item_index = 0;
                for y in 0..height {
                    let y = height - y - 1;
                    for x in 0..width {
                        let slot_entity = SlotBundle::spawn(
                            parent,
                            images,
                            generated_image_assets,
                            Slot {
                                inventory: inventory_entity,
                                item: items
                                    .get(item_index)
                                    .map(|item| item.r#type),
                            },
                            (x, y),
                            slot_size,
                            inventory_area,
                        );
                        inventory.slots.push(slot_entity);
                        item_index += 1;
                    }
                }
            })
            .insert(InventoryBundle::new(inventory, position))
            .id()
    }
}

#[derive(Debug, Clone, Component)]
pub struct Inventory {
    pub owner: Entity,
    pub slots: Vec<Entity>,
    pub dimensions: (u32, u32), // (x,y)
    pub items: Arc<Mutex<HashMap<ItemType, f32>>>,
    pub filter: String,
    pub page: usize,
}

impl Inventory {
    pub fn new(
        owner: Entity,
        slots: Vec<Entity>,
        dimensions: (u32, u32),
        items: &Arc<Mutex<HashMap<ItemType, f32>>>,
    ) -> Self {
        Inventory {
            owner,
            slots,
            dimensions,
            items: items.clone(),
            filter: String::new(),
            page: 0,
        }
    }
}

#[derive(Debug, Clone, Bundle)]
pub struct SlotBundle {
    pub slot: Slot,
    pub area: RectangularArea,
    pub sprite: SpriteBundle,
}

impl SlotBundle {
    pub fn new(
        images: &mut Assets<Image>,
        generated_image_assets: &mut image_gen::GeneratedImageAssets,
        slot: Slot,
        slot_position: (u32, u32),
        slot_size: Vec2,
        inventory_area: RectangularArea,
    ) -> Self {
        let area = RectangularArea::new(slot_size.x, slot_size.y);
        let sprite = match &slot.item {
            Some(item) => SpriteBundle {
                sprite: Self::present_sprite(&slot_size),
                texture: Self::get_texture(
                    images,
                    generated_image_assets,
                    item,
                ),
                transform: Self::slot_transform(
                    slot_size,
                    slot_position,
                    inventory_area,
                ),
                ..default()
            },
            None => SpriteBundle {
                sprite: Self::missing_sprite(),
                transform: Self::slot_transform(
                    slot_size,
                    slot_position,
                    inventory_area,
                ),
                ..default()
            },
        };
        SlotBundle { slot, area, sprite }
    }

    // Spawns the background as well as the slot.
    pub fn spawn(
        parent: &mut ChildBuilder,
        images: &mut Assets<Image>,
        generated_image_assets: &mut image_gen::GeneratedImageAssets,
        slot: Slot,
        slot_position: (u32, u32),
        slot_size: Vec2,
        inventory_area: RectangularArea,
    ) -> Entity {
        parent
            .spawn(SlotBundle::new(
                images,
                generated_image_assets,
                slot,
                slot_position,
                slot_size,
                inventory_area,
            ))
            .with_children(|parent| {
                let _background = parent.spawn(SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgba(0.5, 0.5, 0.5, 0.2),
                        custom_size: Some(slot_size * 0.9),
                        ..default()
                    },
                    ..default()
                });
            })
            .id()
    }

    pub fn redraw(
        commands: &mut EntityCommands,
        images: &mut Assets<Image>,
        generated_image_assets: &mut image_gen::GeneratedImageAssets,
        slot: &Slot,
        size: Vec2,
    ) {
        match slot.item {
            Some(item) => {
                commands
                    .insert(Self::get_texture(
                        images,
                        generated_image_assets,
                        &item,
                    ))
                    .insert(Self::present_sprite(&size));
            }
            None => {
                commands
                    .insert(Self::missing_texture())
                    .insert(Self::missing_sprite());
            }
        }
    }

    fn missing_sprite() -> Sprite {
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, 0.0), // transparent
            ..default()
        }
    }

    fn present_sprite(size: &Vec2) -> Sprite {
        Sprite {
            custom_size: Some(*size * 0.8),
            ..default()
        }
    }

    fn slot_transform(
        slot_size: Vec2,
        slot_pos: (u32, u32),
        inventory_area: RectangularArea,
    ) -> Transform {
        let delta_x = slot_size.x / 2.0 - inventory_area.width / 2.0;
        let delta_y = slot_size.y / 2.0 - inventory_area.height / 2.0;
        Transform::from_translation(Vec3::new(
            delta_x + (slot_pos.0 as f32 * slot_size.x),
            delta_y + (slot_pos.1 as f32 * slot_size.y),
            2.0,
        ))
    }

    fn get_texture(
        images: &mut Assets<Image>,
        generated_image_assets: &mut image_gen::GeneratedImageAssets,
        item: &ItemType,
    ) -> Handle<Image> {
        match generated_image_assets.get(&item.uid()) {
            Some(texture) => texture.clone(),
            None => {
                let image = item.draw(&mut WyRand::new(SEED));
                let texture = images.add(image.clone());
                generated_image_assets.insert(item.uid(), &texture);
                texture
            }
        }
    }

    fn missing_texture() -> Handle<Image> {
        Handle::<Image>::default()
    }
}

#[derive(Debug, Clone, Component)]
pub struct Slot {
    pub inventory: Entity,
    pub item: Option<ItemType>,
}

pub fn add_item(
    inventory: &Arc<Mutex<HashMap<ItemType, f32>>>,
    item: ItemType,
    amount: f32,
) -> f32 {
    let mut inventory = inventory.lock().unwrap();
    let current = inventory.entry(item).or_insert(0.0);
    *current += amount;
    *current
}

// Returns (removed, remaining)
pub fn remove_item(
    inventory: &Arc<Mutex<HashMap<ItemType, f32>>>,
    item: ItemType,
    amount: f32,
) -> (f32, f32) {
    let mut inventory = inventory.lock().unwrap();
    if !inventory.contains_key(&item) {
        return (0.0, amount);
    }
    let current = inventory.get_mut(&item).unwrap();
    let removed = amount.min(*current);
    *current -= removed;
    if *current > 0.0 {
        return (removed, *current);
    } else {
        inventory.remove(&item);
        return (removed, 0.0);
    }
}

pub fn total_stored(inventory: &Arc<Mutex<HashMap<ItemType, f32>>>) -> f32 {
    inventory.lock().unwrap().values().sum()
}

pub fn filter_items(
    inventory: &Arc<Mutex<HashMap<ItemType, f32>>>,
    filter: String,
    per_page: usize,
    page: usize,
) -> Vec<Item> {
    let mut count = 0;
    let offset = per_page * page;
    inventory
        .lock()
        .unwrap()
        .iter()
        .filter_map(|(item_type, amount)| {
            let matches = item_type
                .uid()
                .to_lowercase()
                .contains(&filter.to_lowercase());
            if !matches {
                return None;
            }
            count += 1;
            if count <= offset {
                return None;
            }
            if count > offset + per_page {
                // TODO rewrite to short-circuit
                return None;
            }
            Some(Item {
                r#type: item_type.clone(),
                amount: *amount,
            })
        })
        .collect()
    // TODO rewrite to pre-allocate
}

//
// SYSTEMS
//

pub fn handle_slot_click(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut generated_image_assets: ResMut<image_gen::GeneratedImageAssets>,
    mouse_state: Res<MouseState>,
    time: Res<Time>,
    inventory_query: Query<&Inventory>,
    minigame_query: Query<(&Minigame, &GlobalTransform)>,
    mut slot_query: Query<(&mut Slot, &GlobalTransform, &RectangularArea)>,
) {
    if !mouse_state.just_pressed {
        return;
    }
    let click_position = mouse_state.current_position;

    let mut slot = match slot_query.iter_mut().find(|(_, transform, area)| {
        area.is_within(click_position, transform.translation().truncate())
    }) {
        Some((slot, _, _)) => slot,
        None => return,
    };

    let item_type = match slot.item {
        Some(item) => item,
        None => return,
    };

    let inventory: &Inventory = inventory_query.get(slot.inventory).unwrap();
    let (minigame, minigame_transform) =
        minigame_query.get(inventory.owner).unwrap();

    let amount: f32 = match inventory.items.lock().unwrap().get(&item_type) {
        Some(amount) => {
            match mouse_state.get_click_type(time.elapsed_seconds()) {
                ClickType::Short => amount.min(1.0),
                ClickType::Long => *amount,
                ClickType::Invalid => return,
            }
        }
        None => return,
    };
    let (removed, remaining) = remove_item(&inventory.items, item_type, amount);
    commands.spawn(ItemBundle::new_from_minigame(
        &mut images,
        &mut generated_image_assets,
        Item::new(item_type, removed),
        minigame_transform,
        &minigame.area(),
    ));
    if remaining == 0.0 {
        slot.item.take();
    }
}

pub fn set_slots(
    mut slot_query: Query<&mut Slot>,
    inventory_query: Query<&Inventory, Changed<Inventory>>,
    leveling_query: Query<&LevelingUp>,
) {
    for inventory in inventory_query.iter() {
        if leveling_query.get(inventory.owner).is_ok() {
            continue;
        }

        let (width, height) = inventory.dimensions;
        let items = filter_items(
            &inventory.items,
            inventory.filter.clone(),
            (width * height) as usize,
            inventory.page,
        );
        for (index, slot_entity) in inventory.slots.iter().enumerate() {
            let mut slot = slot_query.get_mut(*slot_entity).unwrap();
            if let Some(item) = items.get(index as usize) {
                slot.item = Some(item.r#type.clone());
            } else {
                slot.item = None;
            }
        }
    }
}

pub fn redraw_slots(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut generated_image_assets: ResMut<image_gen::GeneratedImageAssets>,
    query: Query<(Entity, &Slot, &RectangularArea), Changed<Slot>>,
) {
    for (entity, slot, area) in query.iter() {
        SlotBundle::redraw(
            &mut commands.entity(entity),
            &mut images,
            &mut generated_image_assets,
            &slot,
            area.dimensions(),
        );
    }
}
