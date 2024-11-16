use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use crate::entities::*;
use crate::libs::*;

pub const ID: &str = "land";
pub const POSITION: Vec2 = Vec2::new(-600.0, -600.0);

pub const NAME: &str = "Land";
pub const DESCRIPTION: &str = "Conway's Game of Land";

const MIN_WIDTH: f32 = 100.0;
const MIN_HEIGHT: f32 = 100.0;

#[derive(Debug, Clone, Component)]
pub struct LandMinigame {
    pub level: u8, // equivalent to max achieved complexity
    pub max_achieved_complexity: u8, // used for levelup
    pub energy: f32,
    pub cells: Vec<Vec<Option<ItemType>>>,
}

impl Default for LandMinigame {
    fn default() -> Self {
        Self::new(0, 0.0)
    }
}

impl LandMinigame {
    pub fn new(max_achieved_complexity: u8, energy: f32) -> Self {
        let level = max_achieved_complexity;
        let blocks_per_row = Self::_blocks_per_row(level) as usize;
        let blocks_per_column = Self::_blocks_per_column(level) as usize;
        let cells = vec![vec![None; blocks_per_row]; blocks_per_column];
        Self {
            level,
            max_achieved_complexity,
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
            width: BUFFER + MIN_WIDTH.max(blocks_per_row as f32),
            height: BUFFER + MIN_HEIGHT.max(blocks_per_column as f32),
        }
    }

    pub fn level(&self) -> u8 {
        self.level
    }

    pub fn levelup(&self) -> Self {
        Self::new(self.level, self.energy)
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

    //
    // SPECIFIC
    //

    pub fn level_by_complexity(extracted: f32) -> u8 {
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
        let dx = -1.0 * ((cols - 1) as f32 / 2.0);
        let dy = -1.0 * ((rows + 1) as f32 / 2.0);
        Self {
            cell: Cell { x, y },
            toggleable: Toggleable::new(),
            sprite: SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(1.0, 1.0)),
                    ..default()
                },
                transform: Transform::from_xyz(
                    x as f32 + dx,
                    y as f32 + dy,
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

// Cell was clicked so emit that cell's item, if any.
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
        if RectangularArea::new(1.0, 1.0).is_within(
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

// Run minigame simulation according to its rules.
// Life grows, etc.
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
//      exception:energy of any kind, which enables fixed_update to run
//      exception: abstraction mostly doesn't make sense here
//      exception: some mana probably does not make sense here
pub fn ingest_fixed_update() {}
