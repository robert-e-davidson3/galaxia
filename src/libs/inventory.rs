use std::collections::HashMap;

use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use wyrand::WyRand;

use crate::entities::item::*;
use crate::entities::minigame::*;
use crate::libs::*;

#[derive(Debug, Clone, Bundle)]
pub struct InventoryBundle {
    pub inventory: Inventory,
    pub transform: Transform,
    pub visibility: Visibility,
}

impl InventoryBundle {
    pub fn new(inventory: Inventory, position: Vec2) -> Self {
        let (x, y) = position.into();
        InventoryBundle {
            inventory,
            transform: Transform::from_translation(Vec3::new(x, y, 0.0)),
            visibility: Visibility::default(),
        }
    }

    pub fn spawn(
        parent: &mut ChildSpawnerCommands,
        mut inventory: Inventory,
        items: &HashMap<ItemType, f32>,
        position: Vec2,
        inventory_size: Vec2,
    ) -> Entity {
        let (width, height) = inventory.dimensions;
        let slot_size = Vec2::new(
            inventory_size.x / width as f32,
            inventory_size.y / height as f32,
        );
        let items = filter_items(
            items,
            inventory.filter.clone(),
            (width * height) as usize,
            0,
        );
        let inventory_area =
            RectangularArea::new(inventory_size.x, inventory_size.y);
        let (origin_x, origin_y) = position.into();
        parent
            // Spatial components must exist on the parent before its slot children
            // spawn, or each slot's GlobalTransform fires B0004 (parent without
            // one). Same reasoning as the minigame entity spawn.
            .spawn((
                Transform::from_translation(Vec3::new(origin_x, origin_y, 0.0)),
                Visibility::default(),
            ))
            .with_children(|parent| {
                let inventory_entity = parent.target_entity();
                let mut item_index = 0;
                for y in 0..height {
                    let y = height - y - 1;
                    for x in 0..width {
                        let slot_entity = SlotBundle::spawn(
                            parent,
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

// The inventory UI. The items it displays are not stored here — they live on
// the owning minigame entity (`owner`'s `Minigame::items()`), which is the
// single source of truth and survives the despawn/respawn on levelup. This
// component only holds the layout and view state.
#[derive(Debug, Clone, Component)]
pub struct Inventory {
    pub owner: Entity,
    pub slots: Vec<Entity>,
    pub dimensions: (u32, u32), // (x,y)
    pub filter: String,
    pub page: usize,
}

impl Inventory {
    pub fn new(
        owner: Entity,
        slots: Vec<Entity>,
        dimensions: (u32, u32),
    ) -> Self {
        Inventory {
            owner,
            slots,
            dimensions,
            filter: String::new(),
            page: 0,
        }
    }
}

#[derive(Debug, Clone, Bundle)]
pub struct SlotBundle {
    pub slot: Slot,
    pub area: RectangularArea,
    pub sprite: Sprite,
    pub transform: Transform,
}

impl SlotBundle {
    pub fn new(
        slot: Slot,
        slot_position: (u32, u32),
        slot_size: Vec2,
        inventory_area: RectangularArea,
    ) -> Self {
        let area = RectangularArea::new(slot_size.x, slot_size.y);
        let sprite = Self::missing_sprite();
        let transform = Self::slot_transform(
            slot_size,
            slot_position,
            inventory_area,
        );
        SlotBundle {
            slot,
            area,
            sprite,
            transform,
        }
    }

    // Spawns the background as well as the slot.
    pub fn spawn(
        parent: &mut ChildSpawnerCommands,
        slot: Slot,
        slot_position: (u32, u32),
        slot_size: Vec2,
        inventory_area: RectangularArea,
    ) -> Entity {
        parent
            .spawn(SlotBundle::new(
                slot,
                slot_position,
                slot_size,
                inventory_area,
            ))
            .with_children(|parent| {
                let _background = parent.spawn((
                    Sprite {
                        color: Color::srgba(0.5, 0.5, 0.5, 0.2),
                        custom_size: Some(slot_size * 0.9),
                        ..default()
                    },
                    Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)),
                ));
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
                let texture = Self::get_texture(
                    images,
                    generated_image_assets,
                    &item,
                );
                commands.insert(Self::present_sprite(texture, &size));
            }
            None => {
                commands.insert(Self::missing_sprite());
            }
        }
    }

    fn missing_sprite() -> Sprite {
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, 0.0), // transparent
            ..default()
        }
    }

    fn present_sprite(image: Handle<Image>, size: &Vec2) -> Sprite {
        Sprite {
            image,
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
    inventory: &mut HashMap<ItemType, f32>,
    item: ItemType,
    amount: f32,
) -> f32 {
    let current = inventory.entry(item).or_insert(0.0);
    *current += amount;
    *current
}

// Returns (removed, remaining)
pub fn remove_item(
    inventory: &mut HashMap<ItemType, f32>,
    item: ItemType,
    amount: f32,
) -> (f32, f32) {
    if !inventory.contains_key(&item) {
        return (0.0, amount);
    }
    let current = inventory.get_mut(&item).unwrap();
    let removed = amount.min(*current);
    *current -= removed;
    if *current > 0.0 {
        (removed, *current)
    } else {
        inventory.remove(&item);
        (removed, 0.0)
    }
}

pub fn total_stored(inventory: &HashMap<ItemType, f32>) -> f32 {
    inventory.values().sum()
}

pub fn filter_items(
    inventory: &HashMap<ItemType, f32>,
    filter: String,
    per_page: usize,
    page: usize,
) -> Vec<Item> {
    let offset = per_page * page;
    let filter = filter.to_lowercase();
    let mut result = Vec::with_capacity(per_page);
    let page_items = inventory
        .iter()
        .filter(|(item_type, _)| {
            item_type.uid().to_lowercase().contains(&filter)
        })
        .skip(offset)
        .take(per_page);
    for (item_type, amount) in page_items {
        result.push(Item {
            r#type: *item_type,
            amount: *amount,
        });
    }
    result
}

//
// TEXT ENTRY and SEARCH
//

const SCROLL_BUTTON_SIZE: f32 = 20.0;

#[derive(Bundle)]
struct SearchBoxBundle {
    text_box: TextBox,
    sprite: Sprite,
    transform: Transform,
}

impl SearchBoxBundle {
    fn new(position: Vec2) -> Self {
        Self {
            text_box: TextBox,
            sprite: Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2::new(200.0, 30.0)),
                ..default()
            },
            transform: Transform::from_xyz(position.x, position.y, 0.0),
        }
    }
}

#[derive(Component)]
struct TextBox;

#[derive(Bundle)]
struct ScrollButtonBundle {
    button: ScrollButton,
    sprite: Sprite,
    transform: Transform,
}

impl ScrollButtonBundle {
    fn new(asset_server: &AssetServer, left: bool, position: Vec2) -> Self {
        Self {
            button: ScrollButton { left },
            sprite: Sprite {
                image: asset_server.load(if left {
                    "left_arrow.png"
                } else {
                    "right_arrow.png"
                }),
                custom_size: Some(Vec2::new(
                    SCROLL_BUTTON_SIZE,
                    SCROLL_BUTTON_SIZE,
                )),
                ..default()
            },
            transform: Transform::from_xyz(position.x, position.y, 0.0),
        }
    }
}

#[derive(Component)]
struct ScrollButton {
    left: bool,
}

//
// SYSTEMS
//

pub fn handle_slot_click(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut generated_image_assets: ResMut<image_gen::GeneratedImageAssets>,
    mouse_state: Res<MouseState>,
    inventory_query: Query<&Inventory>,
    mut minigame_query: Query<(&mut Minigame, &GlobalTransform)>,
    mut slot_query: Query<(&mut Slot, &GlobalTransform, &RectangularArea)>,
) {
    if !mouse_state.just_released {
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
    let (mut minigame, minigame_transform) =
        minigame_query.get_mut(inventory.owner).unwrap();
    let minigame_transform = *minigame_transform;
    let minigame_area = minigame.area();
    let Some(items) = minigame.items_mut() else {
        return;
    };

    let amount: f32 = match items.get(&item_type) {
        Some(amount) => match mouse_state.get_click_type() {
            ClickType::Short => amount.min(1.0),
            ClickType::Long => *amount,
            ClickType::Invalid => return,
        },
        None => return,
    };
    let (removed, remaining) = remove_item(items, item_type, amount);
    commands.spawn(ItemBundle::new_from_minigame(
        &mut images,
        &mut generated_image_assets,
        Item::new(item_type, removed),
        &minigame_transform,
        &minigame_area,
    ));
    if remaining == 0.0 {
        slot.item.take();
    }
}

pub fn set_slots(
    mut slot_query: Query<&mut Slot>,
    inventory_query: Query<&Inventory, Changed<Inventory>>,
    minigame_query: Query<&Minigame>,
    leveling_query: Query<&LevelingUp>,
) {
    for inventory in inventory_query.iter() {
        if leveling_query.get(inventory.owner).is_ok() {
            continue;
        }

        let Ok(minigame) = minigame_query.get(inventory.owner) else {
            continue;
        };
        let Some(stored) = minigame.items() else {
            continue;
        };

        let (width, height) = inventory.dimensions;
        let items = filter_items(
            stored,
            inventory.filter.clone(),
            (width * height) as usize,
            inventory.page,
        );
        for (index, slot_entity) in inventory.slots.iter().enumerate() {
            let mut slot = slot_query.get_mut(*slot_entity).unwrap();
            if let Some(item) = items.get(index) {
                slot.item = Some(item.r#type);
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
            slot,
            area.dimensions(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    // Builds an item store pre-loaded with the given (type, amount) pairs.
    fn store(pairs: &[(ItemType, f32)]) -> HashMap<ItemType, f32> {
        pairs.iter().copied().collect()
    }

    // A distinct physical item type, identified by its form/material.
    fn ptype(form: PhysicalForm, material: PhysicalMaterial) -> ItemType {
        Item::new_physical(form, material, 0.0).r#type
    }

    #[test]
    fn add_item_accumulates_and_returns_new_total() {
        let a = ptype(PhysicalForm::Powder, PhysicalMaterial::Fruit);
        let mut s = store(&[]);
        assert_eq!(add_item(&mut s, a, 3.0), 3.0);
        assert_eq!(add_item(&mut s, a, 2.0), 5.0);
        assert_eq!(total_stored(&s), 5.0);
    }

    #[test]
    fn remove_item_partial_then_full_drops_the_key() {
        let a = ptype(PhysicalForm::Powder, PhysicalMaterial::Fruit);
        let mut s = store(&[(a, 5.0)]);
        // Partial: removes the requested amount, reports the remainder.
        assert_eq!(remove_item(&mut s, a, 2.0), (2.0, 3.0));
        // Over-request: removes only what's left, leaving zero.
        assert_eq!(remove_item(&mut s, a, 10.0), (3.0, 0.0));
        // Emptied keys are removed entirely, not left at 0.0.
        assert_eq!(total_stored(&s), 0.0);
        assert!(filter_items(&s, String::new(), 10, 0).is_empty());
    }

    #[test]
    fn remove_item_absent_removes_nothing() {
        let a = ptype(PhysicalForm::Powder, PhysicalMaterial::Fruit);
        let mut s = store(&[]);
        assert_eq!(remove_item(&mut s, a, 1.0), (0.0, 1.0));
    }

    #[test]
    fn total_stored_sums_all_amounts() {
        let a = ptype(PhysicalForm::Powder, PhysicalMaterial::Fruit);
        let b = ptype(PhysicalForm::Block, PhysicalMaterial::Iron);
        let s = store(&[(a, 2.0), (b, 3.0)]);
        assert_eq!(total_stored(&s), 5.0);
    }

    #[test]
    fn filter_items_empty_filter_returns_all_up_to_per_page() {
        let a = ptype(PhysicalForm::Powder, PhysicalMaterial::Fruit);
        let b = ptype(PhysicalForm::Block, PhysicalMaterial::Iron);
        let c = ptype(PhysicalForm::Ball, PhysicalMaterial::Gold);
        let s = store(&[(a, 1.0), (b, 2.0), (c, 3.0)]);
        assert_eq!(filter_items(&s, String::new(), 10, 0).len(), 3);
    }

    #[test]
    fn filter_items_paginates_without_dropping_or_duplicating() {
        let a = ptype(PhysicalForm::Powder, PhysicalMaterial::Fruit);
        let b = ptype(PhysicalForm::Block, PhysicalMaterial::Iron);
        let c = ptype(PhysicalForm::Ball, PhysicalMaterial::Gold);
        let s = store(&[(a, 1.0), (b, 2.0), (c, 3.0)]);
        // HashMap order is unspecified, so assert on counts and coverage
        // rather than which item lands on which page.
        let page0 = filter_items(&s, String::new(), 2, 0);
        let page1 = filter_items(&s, String::new(), 2, 1);
        assert_eq!(page0.len(), 2);
        assert_eq!(page1.len(), 1);
        let seen: HashSet<ItemType> =
            page0.iter().chain(page1.iter()).map(|i| i.r#type).collect();
        assert_eq!(seen.len(), 3);
    }

    #[test]
    fn filter_items_matches_uid_substring_and_keeps_amount() {
        let a = ptype(PhysicalForm::Powder, PhysicalMaterial::Fruit);
        let b = ptype(PhysicalForm::Block, PhysicalMaterial::Iron);
        let s = store(&[(a, 7.0), (b, 2.0)]);
        // Filtering by a's full uid matches only a (uids are unique).
        let result = filter_items(&s, a.uid(), 10, 0);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].r#type, a);
        assert_eq!(result[0].amount, 7.0);
    }
}
