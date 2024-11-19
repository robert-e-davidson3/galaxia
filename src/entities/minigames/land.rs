#![allow(warnings)]

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use crate::entities::*;
use crate::libs::*;

pub const ID: &str = "land";
pub const POSITION: Vec2 = Vec2::new(600.0, -600.0);

pub const NAME: &str = "Land";
pub const DESCRIPTION: &str = "Evolve life";

const MIN_WIDTH: f32 = 100.0;
const MIN_HEIGHT: f32 = 100.0;

#[derive(Debug, Clone, Component)]
pub struct LandMinigame {
    pub level: u8, // equivalent to max achieved complexity
    pub max_achieved_complexity: u8, // used for levelup
    pub energy: f32,
    // mud, water, etc. also some kinds of mana
    pub terrain: Vec<Vec<ItemType>>,
    // algae, mammals, etc. also some kinds of mana
    pub life: Vec<Vec<Option<ItemType>>>,
}

impl Default for LandMinigame {
    fn default() -> Self {
        Self::new(0, 0.0)
    }
}

impl LandMinigame {
    pub fn new(max_achieved_complexity: u8, energy: f32) -> Self {
        let level = max_achieved_complexity;
        let default_terrain = ItemType::Physical(PhysicalItem {
            form: PhysicalForm::Land,
            material: PhysicalMaterial::Mud,
        });
        let terrain =
            vec![
                vec![default_terrain; Self::width_in_cells(level) as usize];
                Self::height_in_cells(level) as usize
            ];
        let life = vec![
            vec![None; Self::width_in_cells(level) as usize];
            Self::height_in_cells(level) as usize
        ];
        Self {
            level,
            max_achieved_complexity,
            energy,
            terrain,
            life,
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
        RectangularArea {
            width: BUFFER
                + MIN_WIDTH.max(Self::width_in_cells(self.level) as f32),
            height: BUFFER
                + MIN_HEIGHT.max(Self::height_in_cells(self.level) as f32),
        }
    }

    pub fn width_in_cells(level: u8) -> u32 {
        4 * level as u32
    }

    pub fn height_in_cells(level: u8) -> u32 {
        4 * level as u32
    }

    pub fn level(&self) -> u8 {
        self.level
    }

    pub fn levelup(&self) -> Self {
        Self::new(self.level, self.energy)
    }

    pub fn spawn(&self, parent: &mut ChildBuilder) {
        let area = self.area();
        let _background = parent.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.5, 0.5, 0.5),
                custom_size: Some(area.into()),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, -1.0),
            ..default()
        });

        for y in 0..area.height as u32 {
            for x in 0..area.width as u32 {
                parent.spawn(CellBundle::new(
                    x,
                    y,
                    area.width as u32,
                    area.height as u32,
                ));
            }
        }
    }

    pub fn ingest_item(
        &mut self,
        commands: &mut Commands,
        rand: &mut Random,
        images: &mut Assets<Image>,
        generated_image_assets: &mut image_gen::GeneratedImageAssets,
        minigame_transform: &GlobalTransform,
        minigame_area: &RectangularArea,
        item: &Item,
    ) -> f32 {
        match item.r#type {
            ItemType::Energy(energy) => {
                self.energy += item.amount;
                item.amount
            }
            ItemType::Physical(physical) => match physical.form {
                PhysicalForm::Liquid | PhysicalForm::Lump => {
                    if item.amount > 1.0 {
                        let (x, y) = self.random_coordinate(rand);
                        let old = self.set_terrain_cell(x, y, item.r#type);
                        commands.spawn(ItemBundle::new_from_minigame(
                            images,
                            generated_image_assets,
                            Item::new(item.r#type, item.amount - 1.0),
                            minigame_transform,
                            minigame_area,
                        ));

                        1.0
                    } else {
                        0.0
                    }
                }
                _ => 0.0,
            },
            _ => 0.0,
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

    pub fn set_terrain_cell(
        &mut self,
        x: u32,
        y: u32,
        value: ItemType,
    ) -> ItemType {
        let (x, y) = (x as usize, y as usize);
        let old = self.terrain[y][x];
        self.terrain[y][x] = value;
        old
    }

    pub fn get_terrain_cell(&self, x: u32, y: u32) -> ItemType {
        let (x, y) = (x as usize, y as usize);
        self.terrain[y][x]
    }

    // returns the item that was replaced
    pub fn set_life_cell(
        &mut self,
        x: u32,
        y: u32,
        value: Option<ItemType>,
    ) -> Option<ItemType> {
        let (x, y) = (x as usize, y as usize);
        let old = self.life[y][x].clone();
        self.life[y][x] = value;
        old
    }

    pub fn get_life_cell(&self, x: u32, y: u32) -> Option<ItemType> {
        let (x, y) = (x as usize, y as usize);
        self.life[y][x]
    }

    // Run the simulation.
    // Note that this has a bias towards the top-left corner due to the order
    // of iteration.
    pub fn evolve(&mut self, rand: &mut Random) {
        let mut life_exists = false;
        let bounds = (self.life[0].len() as u32, self.life.len() as u32);
        for y in 0..bounds.0 as usize {
            for x in 0..bounds.1 as usize {
                let mut cell = match self.life[y][x] {
                    Some(cell) => cell,
                    None => {
                        continue;
                    }
                };
                life_exists = true;
                match cell {
                    ItemType::Physical(cell) => {
                        match cell.form {
                            PhysicalForm::Archaea => {
                                // TODO
                                // 1. if current cell is not water, die
                                // 2. get random direction
                                // 3. if cell is empty of life but has water, make a copy there
                                match self.get_terrain_cell(x as u32, y as u32)
                                {
                                    ItemType::Physical(terrain) => {
                                        if !terrain.material.is_water() {
                                            self.set_life_cell(
                                                x as u32, y as u32, None,
                                            );
                                        }
                                    }
                                    _ => {
                                        // mana is inhospitable
                                        self.set_life_cell(
                                            x as u32, y as u32, None,
                                        );
                                    }
                                }
                                let (nx, ny) = self.random_neighbor(
                                    rand,
                                    (x as u32, y as u32),
                                );
                                match self
                                    .get_terrain_cell(nx as u32, ny as u32)
                                {
                                    ItemType::Physical(terrain) => {
                                        if terrain.material.is_water() {
                                            match self.get_life_cell(
                                                nx as u32, ny as u32,
                                            ) {
                                                Some(_) => {}
                                                None => {
                                                    self.set_life_cell(
                                                        nx as u32,
                                                        ny as u32,
                                                        Some(ItemType::Physical(
                                                            PhysicalItem {
                                                                form: PhysicalForm::Archaea,
                                                                material: PhysicalMaterial::Adult,
                                                            },
                                                        )),
                                                    );
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    }
                    ItemType::Mana(cell) => {
                        // TODO
                    }
                    _ => {}
                }
            }
        }

        if !life_exists {
            // TODO stick archaea on a random water-terrain cell
        }
    }

    fn random_coordinate(&self, rand: &mut Random) -> (u32, u32) {
        let bound_x = self.life[0].len() as u32;
        let bound_y = self.life.len() as u32;
        (
            (rand.next() as u32) % bound_x,
            (rand.next() as u32) % bound_y,
        )
    }

    // Returns a random neighbor of the given cell.
    // Can return itself.
    fn random_neighbor(
        &self,
        rand: &mut Random,
        here: (u32, u32),
    ) -> (i32, i32) {
        let bounds = (self.life[0].len() as u32, self.life.len() as u32);
        (
            Self::random_1d(rand, here.0, bounds.0) as i32,
            Self::random_1d(rand, here.1, bounds.1) as i32,
        )
    }

    fn random_1d(rand: &mut Random, here: u32, bounds: u32) -> u32 {
        let mut x = here + (rand.next() as u32 % 3);
        if x > 0 {
            x -= 1 // only go negative if possible
        }
        if x >= bounds {
            x = bounds - 1
        }
        x
    }
}

#[derive(Bundle)]
pub struct CellBundle {
    pub cell: Cell,
    pub toggleable: Toggleable,
    pub sprite: SpriteBundle,
}

impl CellBundle {
    pub fn new(x: u32, y: u32, cols: u32, rows: u32) -> Self {
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
    pub x: u32,
    pub y: u32,
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
                Minigame::Land(minigame) => minigame,
                _ => continue,
            };

            // Clear cell and emit item, if present.
            // Otherwise, do nothing.
            let item_type = match minigame.set_life_cell(cell.x, cell.y, None) {
                Some(c) => c,
                None => continue,
            };
            commands.spawn(ItemBundle::new_from_minigame(
                &mut images,
                &mut generated_image_assets,
                item_type.to_item(1.0),
                minigame_transform,
                minigame_area,
            ));

            CellBundle::turn_off(cell_entity, &mut cell_draw_query);
        }
    }
}

// Run minigame simulation according to its rules.
// Life grows, etc.
// Only when minigame has stored energy.
pub fn evolve_fixed_update(
    mut commands: Commands,
    mut rand: ResMut<Random>,
    mut images: ResMut<Assets<Image>>,
    mut generated_image_assets: ResMut<image_gen::GeneratedImageAssets>,
    time: Res<Time>,
    mut minigame_query: Query<(&mut Minigame, Entity), Changed<Minigame>>,
    cell_query: Query<(Entity, &Parent)>,
    mut fill_query: Query<&mut Fill, With<Cell>>,
) {
    for (mut minigame, minigame_entity) in minigame_query.iter_mut() {
        let minigame = match minigame.into_inner() {
            Minigame::Land(minigame) => minigame,
            _ => continue,
        };
        if minigame.energy < 1.0 {
            continue;
        }
        minigame.evolve(&mut rand);
        minigame.energy -= 1.0;
    }
}
