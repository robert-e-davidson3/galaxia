#![allow(warnings)]

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use crate::entities::*;
use crate::libs::*;

pub const ID: &str = "life";
pub const POSITION: Vec2 = Vec2::new(-600.0, -600.0);

pub const NAME: &str = "Life";
pub const DESCRIPTION: &str = "Conway's Game of Life";

const MIN_WIDTH: f32 = 100.0;
const MIN_HEIGHT: f32 = 100.0;
const CELL_SIZE: f32 = 25.0;
const CELL_AREA: RectangularArea = RectangularArea {
    width: CELL_SIZE,
    height: CELL_SIZE,
};

#[derive(Debug, Clone, Component)]
pub struct LifeMinigame {
    pub level: u8,
    pub extracted: f32,
    pub energy: f32,
    pub cells: Vec<Vec<Option<ItemType>>>,
}

impl Default for LifeMinigame {
    fn default() -> Self {
        Self::new(0.0, 0.0)
    }
}

impl LifeMinigame {
    pub fn new(extracted: f32, energy: f32) -> Self {
        let level = Self::level_by_extracted(extracted);
        let blocks_per_row = Self::_blocks_per_row(level) as usize;
        let blocks_per_column = Self::_blocks_per_column(level) as usize;
        let cells = vec![vec![None; blocks_per_row]; blocks_per_column];
        Self {
            level,
            extracted,
            energy,
            cells,
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
        const BUFFER: f32 = 20.0;
        let blocks_per_row = self.blocks_per_row();
        let blocks_per_column = self.blocks_per_column();
        RectangularArea {
            width: BUFFER + MIN_WIDTH.max(CELL_SIZE * blocks_per_row as f32),
            height: BUFFER
                + MIN_HEIGHT.max(CELL_SIZE * blocks_per_column as f32),
        }
    }

    pub fn level(&self) -> u8 {
        self.level
    }

    pub fn levelup(&self) -> Self {
        Self::new(self.extracted, self.energy)
    }

    pub fn spawn(&self, parent: &mut ChildBuilder) {
        let (area, blocks_per_row, blocks_per_column) =
            (self.area(), self.blocks_per_row(), self.blocks_per_column());

        let _background = parent.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.5, 0.5, 0.5),
                custom_size: Some(area.into()),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, -1.0),
            ..default()
        });

        for y in 0..blocks_per_column {
            for x in 0..blocks_per_row {
                parent.spawn(CellBundle::new(
                    x,
                    y,
                    blocks_per_row,
                    blocks_per_column,
                ));
            }
        }
    }

    pub fn ingest_item(&mut self, _: &Item) -> f32 {
        0.0 // does not ingest items
    }

    //
    // SPECIFIC
    //

    pub fn level_by_extracted(extracted: f32) -> u8 {
        if extracted == 0.0 {
            0
        } else {
            ((extracted.log2() + 1.0) as u8).min(99)
        }
    }

    pub fn blocks_per_row(&self) -> u8 {
        Self::_blocks_per_row(self.level)
    }

    pub fn blocks_per_column(&self) -> u8 {
        Self::_blocks_per_column(self.level)
    }

    // level -> blocks_per_row
    // 0 -> 1
    // 1 -> 1
    // 2 -> 2
    fn _blocks_per_row(level: u8) -> u8 {
        if level % 2 == 0 {
            1 + level / 2
        } else {
            2 + level / 2
        }
    }

    // level -> blocks_per_column
    // 0 -> 1
    // 1 -> 2
    // 2 -> 2
    fn _blocks_per_column(level: u8) -> u8 {
        1 + level / 2
    }

    pub fn set_cell(&mut self, x: u8, y: u8, value: Option<ItemType>) {
        let (x, y) = (x as usize, y as usize);
        if y >= self.cells.len() {
            return;
        }
        if x >= self.cells[y].len() {
            return;
        }
        self.cells[y][x] = value.clone();
    }

    pub fn get_cell(&self, x: u8, y: u8) -> Option<ItemType> {
        let (x, y) = (x as usize, y as usize);
        if y >= self.cells.len() {
            return None;
        }
        if x >= self.cells[y].len() {
            return None;
        }
        self.cells[y][x]
    }

    pub fn clear(&mut self) {
        for row in self.cells.iter_mut() {
            for cell in row.iter_mut() {
                *cell = None;
            }
        }
    }
}

#[derive(Bundle)]
pub struct CellBundle {
    pub cell: Cell,
    pub toggleable: Toggleable,
    pub sprite: SpriteBundle,
}

impl CellBundle {
    pub fn new(x: u8, y: u8, cols: u8, rows: u8) -> Self {
        let t_y = rows - y; // top to bottom
        let dx = -CELL_SIZE * ((cols - 1) as f32 / 2.0);
        let dy = -CELL_SIZE * ((rows + 1) as f32 / 2.0);
        Self {
            cell: Cell { x, y },
            toggleable: Toggleable::new(),
            sprite: SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(CELL_AREA.into()),
                    ..default()
                },
                transform: Transform::from_xyz(
                    x as f32 * CELL_SIZE + dx,
                    t_y as f32 * CELL_SIZE + dy,
                    0.0,
                ),
                ..default()
            },
        }
    }

    pub fn turn_on(
        entity: Entity,
        query: &mut Query<(&mut Handle<Image>, &mut Sprite), With<Cell>>,
        new_handle: Handle<Image>,
    ) {
        if let Ok((mut handle, mut sprite)) = query.get_mut(entity) {
            *handle = new_handle;
            // TODO verify this means "no tint"
            sprite.color = Color::default();
        }
    }

    pub fn turn_off(
        entity: Entity,
        query: &mut Query<(&mut Handle<Image>, &mut Sprite), With<Cell>>,
    ) {
        if let Ok((_, mut sprite)) = query.get_mut(entity) {
            sprite.color = Color::srgba(0.0, 0.0, 0.0, 0.0);
        }
    }
}

#[derive(Debug, Clone, Component)]
pub struct Cell {
    pub x: u8,
    pub y: u8,
}

// Cell was clicked.
pub fn cell_update(
    mut commands: Commands,
    mouse_state: Res<MouseState>,
    mut images: ResMut<Assets<Image>>,
    mut generated_image_assets: ResMut<image_gen::GeneratedImageAssets>,
    mut minigame_query: Query<(
        &mut Minigame,
        &GlobalTransform,
        &RectangularArea,
    )>,
    leveling_up_query: Query<&LevelingUp, With<Minigame>>,
    cell_query: Query<(&Cell, Entity, &Parent, &GlobalTransform)>,
    mut cell_draw_query: &mut Query<
        (&mut Handle<Image>, &mut Sprite),
        With<Cell>,
    >,
) {
    if !mouse_state.just_pressed {
        return;
    }

    let mouse_position = mouse_state.current_position;
    for (cell, cell_entity, cell_parent, cell_global_transform) in
        cell_query.iter()
    {
        let minigame_entity = cell_parent.get();
        if leveling_up_query.get(minigame_entity).is_ok() {
            continue;
        }
        if CELL_AREA.is_within(
            mouse_position,
            cell_global_transform.translation().truncate(),
        ) {
            let (minigame, minigame_transform, minigame_area) =
                match minigame_query.get_mut(minigame_entity) {
                    Ok((minigame, t, a)) => (minigame, t, a),
                    Err(_) => continue,
                };
            let minigame = match minigame.into_inner() {
                Minigame::Life(minigame) => minigame,
                _ => continue,
            };

            // Only "on" cells do something when clicked
            let item_type = match minigame.get_cell(cell.x, cell.y) {
                Some(c) => c,
                None => continue,
            };

            // Clear cell
            minigame.set_cell(cell.x, cell.y, None);
            CellBundle::turn_off(cell_entity, &mut cell_draw_query);
            // Record extraction
            minigame.extracted += 1.0;
            // Emit item
            commands.spawn(ItemBundle::new_from_minigame(
                &mut images,
                &mut generated_image_assets,
                item_type.to_item(1.0),
                minigame_transform,
                minigame_area,
            ));
        }
    }
}

// Run the Game of Life rules on the cells.
// Only when minigame has stored energy.
pub fn evolve_fixed_update(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut generated_image_assets: ResMut<image_gen::GeneratedImageAssets>,
    time: Res<Time>,
    mut minigame_query: Query<(
        &mut Minigame,
        &GlobalTransform,
        &RectangularArea,
    )>,
    leveling_up_query: Query<&LevelingUp, With<Minigame>>,
    cell_query: Query<(Entity, &Parent)>,
    mut fill_query: Query<&mut Fill, With<Cell>>,
) {
    return; // TODO
}

// TODO ingestion of items - fills a random cell
//      exception is energy of any kind, which enables fixed_update to run

pub fn ingest_fixed_update() {}
