use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::entities::*;
use crate::libs::*;

pub const NAME: &str = "chest";
pub const NAME_WITH_BAGS: &str = "chest with bags";
pub const NAME_WITH_BARRELS: &str = "barrels and chest with bags";
pub const NAME_WITH_TANKS: &str = "tanks, barrels, and chest with bags";
pub const DESCRIPTION: &str = "Store your items!";

const STORAGE_SIZE: f32 = 50.0;
const ITEMS_PER_ROW: usize = 5;
const VISIBLE_ROWS: usize = 3;
const SCROLL_BUTTON_WIDTH: f32 = 20.0;

#[derive(Debug, Clone, Default, Component)]
pub struct ChestMinigame {
    pub level: u8,
    pub storage: HashMap<PhysicalItem, f32>, // Item -> amount mapping
    pub scroll_offset: usize,
    pub filter: String,
    pub displayed_items: Vec<PhysicalItem>,
}

impl ChestMinigame {
    pub fn new(level: u8) -> Self {
        Self {
            level,
            storage: HashMap::new(),
            scroll_offset: 0,
            filter: String::new(),
            displayed_items: Vec::new(),
        }
    }

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

    pub fn capacity(&self) -> f32 {
        2.0f32.powi(self.level as i32)
    }

    pub fn total_stored(&self) -> f32 {
        self.storage.values().sum()
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

    pub fn spawn(
        &self,
        parent: &mut ChildBuilder,
        _asset_server: &AssetServer,
    ) {
        let minigame_entity = parent.parent_entity();
        // Background
        parent.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.5, 0.5, 0.5),
                custom_size: Some(self.area().into()),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, -1.0),
            ..default()
        });

        // Search box
        // parent.spawn(SearchBoxBundle::new(Vec2::new(
        //     -self.area().width / 2.0 + 100.0,
        //     self.area().height / 2.0 + 10.0,
        // )));

        // Scroll buttons
        // parent.spawn(ScrollButtonBundle::new(
        //     asset_server,
        //     true,
        //     Vec2::new(
        //         -self.area().width / 2.0 + SCROLL_BUTTON_WIDTH / 2.0,
        //         0.0,
        //     ),
        // ));
        // parent.spawn(ScrollButtonBundle::new(
        //     asset_server,
        //     false,
        //     Vec2::new(self.area().width / 2.0 - SCROLL_BUTTON_WIDTH / 2.0, 0.0),
        // ));

        // Grid slots for items
        for row in 0..VISIBLE_ROWS {
            for col in 0..ITEMS_PER_ROW {
                let x = (col as f32 - (ITEMS_PER_ROW as f32 - 1.0) / 2.0)
                    * STORAGE_SIZE;
                let y = (row as f32 - (VISIBLE_ROWS as f32 - 1.0) / 2.0)
                    * STORAGE_SIZE;
                parent.spawn(ItemSlotBundle::new(
                    minigame_entity,
                    Vec2::new(x, y),
                ));
            }
        }
    }

    pub fn add_item(&mut self, item: Item) -> bool {
        if !self.can_accept(&item) {
            return false;
        }
        let amount = item.amount;
        let item = match item.r#type {
            ItemType::Physical(x) => x,
            _ => return false, // should not happen
        };

        *self.storage.entry(item).or_insert(0.0) += amount;

        // Check if we need to level up
        self.total_stored() >= self.capacity()
    }

    pub fn remove_item(
        &mut self,
        item: &PhysicalItem,
        amount: f32,
    ) -> Option<(PhysicalItem, f32)> {
        if let Some(stored_amount) = self.storage.get_mut(item) {
            let amount_to_remove = amount.min(*stored_amount);
            *stored_amount -= amount_to_remove;

            if *stored_amount <= 0.0 {
                self.storage.remove(item);
            }

            Some((item.clone(), amount_to_remove))
        } else {
            None
        }
    }

    pub fn filtered_items(&self) -> Vec<(PhysicalItem, f32)> {
        let filter = self.filter.to_lowercase();
        self.storage
            .iter()
            .filter(|(item, _)| {
                if filter.is_empty() {
                    return true;
                }
                let name = item.identifier().name().to_lowercase();
                name.contains(&filter)
            })
            .map(|(item, &amount)| (item.clone(), amount))
            .collect()
    }
}

#[derive(Bundle)]
struct SearchBoxBundle {
    text_box: TextBox,
    sprite: SpriteBundle,
}

impl SearchBoxBundle {
    fn new(position: Vec2) -> Self {
        Self {
            text_box: TextBox,
            sprite: SpriteBundle {
                sprite: Sprite {
                    color: Color::WHITE,
                    custom_size: Some(Vec2::new(200.0, 30.0)),
                    ..default()
                },
                transform: Transform::from_xyz(position.x, position.y, 0.0),
                ..default()
            },
        }
    }
}

#[derive(Component)]
struct TextBox;

#[derive(Bundle)]
struct ScrollButtonBundle {
    button: ScrollButton,
    sprite: SpriteBundle,
}

impl ScrollButtonBundle {
    fn new(asset_server: &AssetServer, left: bool, position: Vec2) -> Self {
        Self {
            button: ScrollButton { left },
            sprite: SpriteBundle {
                texture: asset_server.load(if left {
                    "left_arrow.png"
                } else {
                    "right_arrow.png"
                }),
                transform: Transform::from_xyz(position.x, position.y, 0.0),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(
                        SCROLL_BUTTON_WIDTH,
                        STORAGE_SIZE,
                    )),
                    ..default()
                },
                ..default()
            },
        }
    }
}

#[derive(Component)]
struct ScrollButton {
    left: bool,
}

#[derive(Bundle)]
pub struct ItemSlotBundle {
    slot: ItemSlot,
    sprite: SpriteBundle,
}

impl ItemSlotBundle {
    fn new(minigame: Entity, position: Vec2) -> Self {
        Self {
            slot: ItemSlot {
                minigame,
                position,
                item: None,
            },
            sprite: SpriteBundle {
                sprite: Sprite {
                    color: Color::srgba(1.0, 1.0, 1.0, 0.2),
                    custom_size: Some(Vec2::new(
                        STORAGE_SIZE * 0.9,
                        STORAGE_SIZE * 0.9,
                    )),
                    ..default()
                },
                transform: Transform::from_xyz(position.x, position.y, 0.0),
                ..default()
            },
        }
    }
}

#[derive(Component)]
pub struct ItemSlot {
    minigame: Entity,
    position: Vec2,
    item: Option<PhysicalItem>,
}

pub fn ingest_resource_fixed_update(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut minigame_query: Query<&mut Minigame>,
    aura_query: Query<&MinigameAura>,
    item_query: Query<&Item>,
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

        let should_level_up = minigame.add_item(*item);
        if should_level_up {
            commands.entity(aura.minigame).insert(LevelingUp);
        }

        commands.entity(item_entity).despawn();
        ingested.insert(item_entity);
    }
}

pub fn handle_item_clicks(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut generated_image_assets: ResMut<image_gen::GeneratedImageAssets>,
    mouse_state: Res<MouseState>,
    time: Res<Time>,
    mut minigame_query: Query<(&mut Minigame, &GlobalTransform)>,
    slot_query: Query<&ItemSlot>,
) {
    if !mouse_state.just_pressed {
        return;
    }
    let mouse_position = mouse_state.current_position;

    for (mut minigame, transform) in minigame_query.iter_mut() {
        let minigame = match minigame.as_mut() {
            Minigame::Chest(m) => m,
            _ => {
                continue;
            }
        };

        let is_within = minigame
            .area()
            .is_within(mouse_position, transform.translation().truncate());
        if !is_within {
            continue;
        }

        let click_type = mouse_state.get_click_type(time.elapsed_seconds());
        if click_type == ClickType::Invalid {
            continue;
        }

        let filtered_items = minigame.filtered_items();

        for (item, amount) in filtered_items.iter() {
            let amount_to_remove = if click_type == ClickType::Long {
                *amount
            } else {
                1.0f32.min(*amount)
            };

            if let Some((item, amount)) =
                minigame.remove_item(item, amount_to_remove)
            {
                commands.spawn(ItemBundle::new_from_minigame(
                    &mut images,
                    &mut generated_image_assets,
                    Item::new_physical(item.form, item.material, amount),
                    transform,
                    &minigame.area(),
                ));
            }
        }
    }
}
