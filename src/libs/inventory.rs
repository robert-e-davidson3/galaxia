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
        inventory: Inventory,
        position: Vec2,
        inventory_size: Vec2,
    ) {
        let (width, height) = inventory.slots;
        let slot_size = Vec2::new(
            inventory_size.x / width as f32,
            inventory_size.y / height as f32,
        );
        parent
            .spawn(InventoryBundle::new(inventory, position))
            .with_children(|parent| {
                let inventory_entity = parent.parent_entity();
                for y in 0..height {
                    for x in 0..width {
                        SlotBundle::spawn(
                            parent,
                            images,
                            generated_image_assets,
                            Slot {
                                inventory: inventory_entity,
                                item: None,
                                x,
                                y,
                            },
                            (x, y),
                            slot_size,
                        );
                    }
                }
            });
    }
}

#[derive(Debug, Clone, Component)]
pub struct Inventory {
    pub owner: Entity,
    pub slots: (u32, u32), // (x,y)
    pub items: Arc<Mutex<HashMap<ItemType, f32>>>,
}

impl Inventory {
    pub fn new(
        owner: Entity,
        slots: (u32, u32),
        items: &Arc<Mutex<HashMap<ItemType, f32>>>,
    ) -> Self {
        Inventory {
            owner,
            items: items.clone(),
            slots,
        }
    }
}

#[derive(Debug, Clone, Bundle)]
pub struct SlotBundle {
    pub slot: Slot,
    pub sprite: SpriteBundle,
    pub area: RectangularArea,
}

impl SlotBundle {
    // TODO adjust slot positions to center on inventory
    pub fn new(
        images: &mut Assets<Image>,
        generated_image_assets: &mut image_gen::GeneratedImageAssets,
        slot: Slot,
        position: (u32, u32),
        size: Vec2,
    ) -> Self {
        let area = RectangularArea::new(size.x, size.y);
        let (x, y) = position;
        match slot.item {
            Some(item) => SlotBundle {
                slot,
                sprite: SpriteBundle {
                    sprite: Self::present_sprite(&size),
                    texture: Self::get_texture(
                        images,
                        generated_image_assets,
                        &item,
                    ),
                    transform: Transform::from_translation(Vec3::new(
                        x as f32 * size.x,
                        y as f32 * size.y,
                        0.0,
                    )),
                    ..default()
                },
                area,
            },
            None => SlotBundle {
                slot,
                sprite: SpriteBundle {
                    sprite: Self::missing_sprite(),
                    transform: Transform::from_translation(Vec3::new(
                        x as f32 * size.x,
                        y as f32 * size.y,
                        0.0,
                    )),
                    ..default()
                },
                area,
            },
        }
    }

    // Spawns the background as well as the slot.
    pub fn spawn(
        parent: &mut ChildBuilder,
        images: &mut Assets<Image>,
        generated_image_assets: &mut image_gen::GeneratedImageAssets,
        slot: Slot,
        position: (u32, u32),
        size: Vec2,
    ) {
        parent
            .spawn(SlotBundle::new(
                images,
                generated_image_assets,
                slot,
                position,
                size,
            ))
            .with_children(|parent| {
                let _background = parent.spawn(SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgba(1.0, 1.0, 1.0, 0.2),
                        custom_size: Some(size * 0.9),
                        ..Default::default()
                    },
                    ..Default::default()
                });
            });
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
                    .insert(Self::missing_sprite())
                    .insert(Self::missing_texture());
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
    pub x: u32,
    pub y: u32,
}

pub fn add_item(
    inventory: &Arc<Mutex<HashMap<ItemType, f32>>>,
    item: ItemType,
    amount: f32,
) {
    let mut inventory = inventory.lock().unwrap();
    let current = inventory.entry(item).or_insert(0.0);
    *current += amount;
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
) -> HashMap<ItemType, f32> {
    if filter.is_empty() {
        return inventory.lock().unwrap().clone();
    }
    inventory
        .lock()
        .unwrap()
        .iter()
        .filter(|(item, _)| {
            item.uid().to_lowercase().contains(&filter.to_lowercase())
        })
        .map(|(item, amount)| (item.clone(), *amount))
        .collect()
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
