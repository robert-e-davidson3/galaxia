use bevy::prelude::*;
use wyrand::WyRand;

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

// Fixed-update ticks between evolution steps (FixedUpdate is 20 Hz → ~1s), so
// the simulation is watchable rather than an instant blur. A tick countdown
// (not an elapsed-time check) so it survives respawn on level-up.
const EVOLVE_TICKS: u32 = 20;

#[derive(Debug, Clone, Component)]
pub struct LifeMinigame {
    pub level: u8,
    // Cumulative |births - deaths| over evolution steps; drives leveling.
    pub xp: f32,
    pub energy: f32,
    pub cells: Vec<Vec<Option<ItemType>>>,
    // Fixed-update ticks until the next evolution step.
    pub evolve_cooldown: u32,
}

impl Default for LifeMinigame {
    fn default() -> Self {
        Self::new(0.0, 0.0)
    }
}

impl LifeMinigame {
    pub fn new(xp: f32, energy: f32) -> Self {
        let level = Self::level_by_xp(xp);
        let blocks_per_row = Self::_blocks_per_row(level) as usize;
        let blocks_per_column = Self::_blocks_per_column(level) as usize;
        let cells = vec![vec![None; blocks_per_row]; blocks_per_column];
        Self {
            level,
            xp,
            energy,
            cells,
            evolve_cooldown: EVOLVE_TICKS,
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
        // Preserve the colony into the (larger) new grid rather than wiping it.
        let mut next = Self::new(self.xp, self.energy);
        for (y, row) in self.cells.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                if y < next.cells.len() && x < next.cells[y].len() {
                    next.cells[y][x] = *cell;
                }
            }
        }
        next
    }

    pub fn spawn(&self, parent: &mut ChildSpawnerCommands) {
        let (area, blocks_per_row, blocks_per_column) =
            (self.area(), self.blocks_per_row(), self.blocks_per_column());

        let _background = parent.spawn((
            Sprite {
                color: Color::srgb(0.5, 0.5, 0.5),
                custom_size: Some(area.into()),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, -1.0),
        ));

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

    pub fn ingest_item(&mut self, rand: &mut Random, item: &Item) -> f32 {
        match item.r#type {
            // Energy fuels evolution.
            ItemType::Energy(_) => {
                self.energy += item.amount;
                item.amount
            }
            // Anything else seeds a new life cell.
            _ => {
                if self.seed_random_cell(rand) {
                    1.0
                } else {
                    0.0
                }
            }
        }
    }

    //
    // SPECIFIC
    //

    // XP is cumulative |births - deaths|. Levels are geometric: level N at
    // XP >= 2^(N-1), so the first level comes from a single death, but high
    // levels demand a large net birth surplus (only achievable later with
    // rule-bending items — see the design note in logs/2026-06-28.md).
    pub fn level_by_xp(xp: f32) -> u8 {
        if xp <= 0.0 {
            0
        } else {
            ((xp.log2() + 1.0) as u8).min(99)
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
        if level.is_multiple_of(2) {
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
        self.cells[y][x] = value;
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
        for cell in self.cells.iter_mut().flatten() {
            *cell = None;
        }
    }

    // One step of Conway's Game of Life: a live cell survives with 2-3 live
    // neighbors; a dead cell with exactly 3 is born, inheriting a neighbor's
    // species. No wraparound at the edges.
    // Advance one Conway step; returns the XP earned: |births - deaths|.
    // Balanced patterns (oscillators, still lifes, gliders) net zero and so
    // score nothing — only genuine population change counts.
    pub fn step(&mut self) -> u32 {
        let height = self.cells.len();
        if height == 0 {
            return 0;
        }
        let width = self.cells[0].len();
        let next: Vec<Vec<Option<ItemType>>> = (0..height)
            .map(|y| {
                (0..width)
                    .map(|x| self.next_cell(x, y, width, height))
                    .collect()
            })
            .collect();
        let mut births = 0u32;
        let mut deaths = 0u32;
        for (old_row, new_row) in self.cells.iter().zip(next.iter()) {
            for (old, new) in old_row.iter().zip(new_row.iter()) {
                match (old.is_some(), new.is_some()) {
                    (false, true) => births += 1,
                    (true, false) => deaths += 1,
                    _ => {}
                }
            }
        }
        self.cells = next;
        births.abs_diff(deaths)
    }

    // The next state of one cell under Conway's rules.
    fn next_cell(
        &self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> Option<ItemType> {
        let mut live = 0;
        let mut neighbor = None;
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32
                {
                    continue;
                }
                if let Some(t) = self.cells[ny as usize][nx as usize] {
                    live += 1;
                    neighbor = Some(t);
                }
            }
        }
        match self.cells[y][x] {
            Some(_) if live == 2 || live == 3 => self.cells[y][x],
            None if live == 3 => neighbor,
            _ => None,
        }
    }

    // Place a life form in a random empty cell. Returns false if none are free.
    fn seed_random_cell(&mut self, rand: &mut Random) -> bool {
        let empty: Vec<(usize, usize)> = self
            .cells
            .iter()
            .enumerate()
            .flat_map(|(y, row)| {
                row.iter().enumerate().filter_map(move |(x, cell)| {
                    cell.is_none().then_some((x, y))
                })
            })
            .collect();
        if empty.is_empty() {
            return false;
        }
        let (x, y) = empty[(rand.next() as usize) % empty.len()];
        self.cells[y][x] = Some(Self::life_form());
        true
    }

    fn life_form() -> ItemType {
        Item::organism(Species::Archaea, LifeStage::Adult, 1.0).r#type
    }
}

#[derive(Bundle)]
pub struct CellBundle {
    pub cell: Cell,
    pub toggleable: Toggleable,
    pub sprite: Sprite,
    pub transform: Transform,
}

impl CellBundle {
    pub fn new(x: u8, y: u8, cols: u8, rows: u8) -> Self {
        let t_y = rows - y; // top to bottom
        let dx = -CELL_SIZE * ((cols - 1) as f32 / 2.0);
        let dy = -CELL_SIZE * ((rows + 1) as f32 / 2.0);
        Self {
            cell: Cell { x, y },
            toggleable: Toggleable::new(),
            sprite: Sprite {
                custom_size: Some(CELL_AREA.into()),
                ..default()
            },
            transform: Transform::from_xyz(
                x as f32 * CELL_SIZE + dx,
                t_y as f32 * CELL_SIZE + dy,
                0.0,
            ),
        }
    }

    pub fn turn_on(
        entity: Entity,
        query: &mut Query<&mut Sprite, With<Cell>>,
        new_handle: Handle<Image>,
    ) {
        if let Ok(mut sprite) = query.get_mut(entity) {
            sprite.image = new_handle;
            // TODO verify this means "no tint"
            sprite.color = Color::default();
        }
    }

    pub fn turn_off(
        entity: Entity,
        query: &mut Query<&mut Sprite, With<Cell>>,
    ) {
        if let Ok(mut sprite) = query.get_mut(entity) {
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
    cell_query: Query<(&Cell, Entity, &ChildOf, &GlobalTransform)>,
    mut cell_draw_query: Query<&mut Sprite, With<Cell>>,
) {
    if !mouse_state.just_pressed {
        return;
    }

    let mouse_position = mouse_state.current_position;
    for (cell, cell_entity, cell_parent, cell_global_transform) in
        cell_query.iter()
    {
        let minigame_entity = cell_parent.parent();
        if leveling_up_query.get(minigame_entity).is_ok() {
            continue;
        }
        if CELL_AREA.is_within(
            mouse_position,
            cell_global_transform.translation().truncate(),
        ) {
            let Ok((minigame, minigame_transform, minigame_area)) =
                minigame_query.get_mut(minigame_entity)
            else {
                continue;
            };
            let Minigame::Life(minigame) = minigame.into_inner() else {
                continue;
            };

            // Only "on" cells do something when clicked
            let Some(item_type) = minigame.get_cell(cell.x, cell.y) else {
                continue;
            };

            // Clear cell
            minigame.set_cell(cell.x, cell.y, None);
            CellBundle::turn_off(cell_entity, &mut cell_draw_query);
            // Emit item (harvesting is a payout, not XP — XP is births/deaths)
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

// Run the Game of Life rules, gated by stored energy and a step interval so the
// simulation is watchable. Each step consumes one energy.
pub fn evolve_fixed_update(
    mut commands: Commands,
    mut minigame_query: Query<(Entity, &mut Minigame)>,
    leveling_up_query: Query<&LevelingUp, With<Minigame>>,
) {
    for (entity, mut minigame) in minigame_query.iter_mut() {
        if leveling_up_query.get(entity).is_ok() {
            continue;
        }
        // Peek immutably first: skip non-Life and unfueled minigames without
        // marking them Changed.
        let Minigame::Life(life) = &*minigame else {
            continue;
        };
        if life.energy < 1.0 {
            continue;
        }
        let stepping = life.evolve_cooldown == 0;

        let Minigame::Life(life) = &mut *minigame else {
            continue;
        };
        if !stepping {
            life.evolve_cooldown -= 1;
            continue;
        }
        life.energy -= 1.0;
        life.evolve_cooldown = EVOLVE_TICKS;
        life.xp += life.step() as f32;
        // Level up once XP crosses the next geometric threshold; the generic
        // levelup system respawns it at the larger grid.
        if LifeMinigame::level_by_xp(life.xp) > life.level {
            commands.entity(entity).insert(LevelingUp);
        }
    }
}

// Repaint each cell to match the model — covers seeding, evolution, and spawn.
// Alive cells show their life form; dead cells are transparent. Cheap: Life
// grids are small.
pub fn render_cells(
    mut images: ResMut<Assets<Image>>,
    mut generated_image_assets: ResMut<image_gen::GeneratedImageAssets>,
    minigame_query: Query<(Entity, &Minigame)>,
    cell_query: Query<(Entity, &Cell, &ChildOf)>,
    mut cell_draw_query: Query<&mut Sprite, With<Cell>>,
) {
    for (minigame_entity, minigame) in minigame_query.iter() {
        let Minigame::Life(life) = minigame else {
            continue;
        };
        for (cell_entity, cell, cell_parent) in cell_query.iter() {
            if cell_parent.parent() != minigame_entity {
                continue;
            }
            match life.get_cell(cell.x, cell.y) {
                Some(item_type) => {
                    let texture = cell_texture(
                        item_type,
                        &mut images,
                        &mut generated_image_assets,
                    );
                    CellBundle::turn_on(
                        cell_entity,
                        &mut cell_draw_query,
                        texture,
                    );
                }
                None => CellBundle::turn_off(cell_entity, &mut cell_draw_query),
            }
        }
    }
}

fn cell_texture(
    item_type: ItemType,
    images: &mut Assets<Image>,
    generated_image_assets: &mut image_gen::GeneratedImageAssets,
) -> Handle<Image> {
    let uid = item_type.uid();
    generated_image_assets.get(&uid).unwrap_or_else(|| {
        let image = item_type.draw(&mut WyRand::new(SEED));
        let handle = images.add(image);
        generated_image_assets.insert(uid, &handle);
        handle
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid(
        alive: &[(usize, usize)],
        w: usize,
        h: usize,
    ) -> Vec<Vec<Option<ItemType>>> {
        let mut cells = vec![vec![None; w]; h];
        for &(x, y) in alive {
            cells[y][x] = Some(LifeMinigame::life_form());
        }
        cells
    }

    fn life_with(cells: Vec<Vec<Option<ItemType>>>) -> LifeMinigame {
        LifeMinigame {
            level: 0,
            xp: 0.0,
            energy: 0.0,
            cells,
            evolve_cooldown: EVOLVE_TICKS,
        }
    }

    fn alive_coords(life: &LifeMinigame) -> Vec<(usize, usize)> {
        life.cells
            .iter()
            .enumerate()
            .flat_map(|(y, row)| {
                row.iter().enumerate().filter_map(move |(x, cell)| {
                    cell.is_some().then_some((x, y))
                })
            })
            .collect()
    }

    #[test]
    fn blinker_oscillates() {
        // Vertical bar in the middle column of a 3x3 grid.
        let mut life = life_with(grid(&[(1, 0), (1, 1), (1, 2)], 3, 3));
        life.step();
        // Period-2: becomes a horizontal bar in the middle row...
        assert_eq!(alive_coords(&life), vec![(0, 1), (1, 1), (2, 1)]);
        life.step();
        // ...and back to vertical.
        assert_eq!(alive_coords(&life), vec![(1, 0), (1, 1), (1, 2)]);
    }

    #[test]
    fn lone_cell_dies_of_underpopulation() {
        let mut life = life_with(grid(&[(1, 1)], 3, 3));
        life.step();
        assert!(alive_coords(&life).is_empty());
    }

    #[test]
    fn seed_fills_an_empty_cell() {
        let mut life = life_with(grid(&[], 2, 2));
        let mut rand = Random::new(1);
        assert_eq!(life.ingest_item(&mut rand, &Item::new_abstract(
            AbstractKind::Click,
            0,
            1.0,
        )), 1.0);
        assert_eq!(alive_coords(&life).len(), 1);
    }

    #[test]
    fn step_scores_absolute_net_change() {
        // A lone cell dies: 0 births, 1 death -> |0 - 1| = 1.
        let mut lone = life_with(grid(&[(1, 1)], 3, 3));
        assert_eq!(lone.step(), 1);
        // A blinker is balanced: 2 born, 2 die -> 0 (no free XP from cycling).
        let mut blinker = life_with(grid(&[(1, 0), (1, 1), (1, 2)], 3, 3));
        assert_eq!(blinker.step(), 0);
    }

    #[test]
    fn level_by_xp_is_geometric() {
        assert_eq!(LifeMinigame::level_by_xp(0.0), 0);
        assert_eq!(LifeMinigame::level_by_xp(1.0), 1);
        assert_eq!(LifeMinigame::level_by_xp(2.0), 2);
        assert_eq!(LifeMinigame::level_by_xp(4.0), 3);
        assert_eq!(LifeMinigame::level_by_xp(8.0), 4);
    }
}
