use bevy::prelude::*;
use wyrand::WyRand;

use crate::entities::*;
use crate::libs::*;

pub const ID: &str = "land";
pub const POSITION: Vec2 = Vec2::new(600.0, -600.0);

pub const NAME: &str = "Land";
pub const DESCRIPTION: &str = "Evolve life";

const MIN_WIDTH: f32 = 100.0;
const MIN_HEIGHT: f32 = 100.0;
const CELL_SIZE: f32 = 25.0;
const CELL_AREA: RectangularArea = RectangularArea {
    width: CELL_SIZE,
    height: CELL_SIZE,
};

// Fixed-update ticks between evolution steps (FixedUpdate is 20 Hz → ~1s), so
// the simulation is watchable rather than an instant blur. A tick countdown
// (not an elapsed-time check) so it survives respawn on level-up. Mirrors life.
const EVOLVE_TICKS: u32 = 20;

// Content is thin (only mud/water terrain and archaea exist as life), so leveling
// is capped low until the food web and species pyramid arrive (see design).
const MAX_LEVEL: u8 = 6;

// A single cell: a stack of coexisting layers, one occupant per layer. Terrain
// is always present (default Mud); the rest are optional. The layers mirror the
// item-model taxonomy classes so insertion routes by class.
#[derive(Debug, Clone)]
pub struct LandCell {
    pub terrain: ItemType, // always present; default Mud
    pub micro: Option<ItemType>,
    pub plant: Option<ItemType>,
    pub animal: Option<ItemType>,
    pub other: Option<ItemType>,
}

impl LandCell {
    fn new(terrain: ItemType) -> Self {
        Self {
            terrain,
            micro: None,
            plant: None,
            animal: None,
            other: None,
        }
    }

    // The texture shown for this cell: the topmost occupied non-terrain layer
    // (other > animal > plant > micro), else the terrain.
    fn top(&self) -> ItemType {
        self.other
            .or(self.animal)
            .or(self.plant)
            .or(self.micro)
            .unwrap_or(self.terrain)
    }
}

#[derive(Debug, Clone, Component)]
pub struct LandMinigame {
    pub level: u8, // derived from max_achieved_complexity, capped at MAX_LEVEL
    pub max_achieved_complexity: u8, // used for levelup
    pub energy: f32,
    pub cells: Vec<Vec<LandCell>>,
    // Fixed-update ticks until the next evolution step.
    pub evolve_cooldown: u32,
}

impl Default for LandMinigame {
    fn default() -> Self {
        Self::new(0, 0.0)
    }
}

impl LandMinigame {
    pub fn new(max_achieved_complexity: u8, energy: f32) -> Self {
        let level = max_achieved_complexity.min(MAX_LEVEL);
        let blocks_per_row = Self::_blocks_per_row(level) as usize;
        let blocks_per_column = Self::_blocks_per_column(level) as usize;
        let default_terrain = Self::default_terrain();
        let cells = vec![
            vec![LandCell::new(default_terrain); blocks_per_row];
            blocks_per_column
        ];
        Self {
            level,
            max_achieved_complexity,
            energy,
            cells,
            evolve_cooldown: EVOLVE_TICKS,
        }
    }

    fn default_terrain() -> ItemType {
        Item::solid(Substance::Mud, BulkShape::Lump, 1.0).r#type
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
        // Preserve the existing cells into the (larger) new grid.
        let mut next = Self::new(self.max_achieved_complexity, self.energy);
        for (y, row) in self.cells.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                if y < next.cells.len() && x < next.cells[y].len() {
                    next.cells[y][x] = cell.clone();
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

    // Route an ingested item onto a random cell, placing one unit and ejecting
    // the remainder. Energy goes into the pool; bulk into terrain; organisms
    // into their class layer; everything else into `other`.
    #[allow(clippy::too_many_arguments)]
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
            // Energy fuels evolution.
            ItemType::Energy(_) => {
                self.energy += item.amount;
                item.amount
            }
            // Bulk substances replace the cell's terrain layer.
            ItemType::Physical(PhysicalItem::Bulk(_)) => {
                self.place(commands, rand, images, generated_image_assets,
                    minigame_transform, minigame_area, item, Layer::Terrain)
            }
            // Organisms route to their taxonomic class layer.
            ItemType::Physical(PhysicalItem::Discrete(d)) => {
                let layer = match d.species.class() {
                    DiscreteClass::Microbe => Layer::Micro,
                    DiscreteClass::Plant => Layer::Plant,
                    DiscreteClass::Animal => Layer::Animal,
                    // Fruit/Tool/Weapon are not organisms here — stash them.
                    _ => Layer::Other,
                };
                self.place(commands, rand, images, generated_image_assets,
                    minigame_transform, minigame_area, item, layer)
            }
            // Mana, abstract, fruit, etc. go in the catch-all `other` layer.
            _ => self.place(commands, rand, images, generated_image_assets,
                minigame_transform, minigame_area, item, Layer::Other),
        }
    }

    //
    // SPECIFIC
    //

    pub fn blocks_per_row(&self) -> u8 {
        Self::_blocks_per_row(self.level)
    }

    pub fn blocks_per_column(&self) -> u8 {
        Self::_blocks_per_column(self.level)
    }

    // Shared gentle growth, same as life: 1×1, 2×1, 2×2, 3×2, 3×3, …
    // level -> blocks_per_row: 0->1, 1->1, 2->2
    fn _blocks_per_row(level: u8) -> u8 {
        if level.is_multiple_of(2) {
            1 + level / 2
        } else {
            2 + level / 2
        }
    }

    // level -> blocks_per_column: 0->1, 1->2, 2->2
    fn _blocks_per_column(level: u8) -> u8 {
        1 + level / 2
    }

    fn dimensions(&self) -> (usize, usize) {
        let height = self.cells.len();
        let width = if height == 0 { 0 } else { self.cells[0].len() };
        (width, height)
    }

    // Place one unit of `item` onto a random cell's `layer`, ejecting the
    // remainder as a loose item. Terrain is always replaced; other layers only
    // accept if empty (else the whole item is rejected). Returns amount ingested.
    #[allow(clippy::too_many_arguments)]
    fn place(
        &mut self,
        commands: &mut Commands,
        rand: &mut Random,
        images: &mut Assets<Image>,
        generated_image_assets: &mut image_gen::GeneratedImageAssets,
        minigame_transform: &GlobalTransform,
        minigame_area: &RectangularArea,
        item: &Item,
        layer: Layer,
    ) -> f32 {
        let (width, height) = self.dimensions();
        if width == 0 || height == 0 {
            return 0.0;
        }
        let x = (rand.next() as usize) % width;
        let y = (rand.next() as usize) % height;
        let cell = &mut self.cells[y][x];

        let placed = match layer {
            Layer::Terrain => {
                cell.terrain = item.r#type;
                true
            }
            Layer::Micro if cell.micro.is_none() => {
                cell.micro = Some(item.r#type);
                true
            }
            Layer::Plant if cell.plant.is_none() => {
                cell.plant = Some(item.r#type);
                true
            }
            Layer::Animal if cell.animal.is_none() => {
                cell.animal = Some(item.r#type);
                true
            }
            Layer::Other if cell.other.is_none() => {
                cell.other = Some(item.r#type);
                true
            }
            // Layer already occupied: reject the whole item.
            _ => false,
        };

        if !placed {
            return 0.0;
        }

        // Eject the remainder.
        if item.amount > 1.0 {
            commands.spawn(ItemBundle::new_from_minigame(
                images,
                generated_image_assets,
                Item::new(item.r#type, item.amount - 1.0),
                minigame_transform,
                minigame_area,
            ));
        }
        1.0
    }

    // A terrain cell counts as water if it is a bulk substance in the Water
    // class (salt/fresh water).
    fn terrain_is_water(terrain: ItemType) -> bool {
        matches!(
            terrain,
            ItemType::Physical(PhysicalItem::Bulk(bulk))
                if bulk.substance.is_water()
        )
    }

    fn is_archaea(item: ItemType) -> bool {
        matches!(
            item,
            ItemType::Physical(PhysicalItem::Discrete(d))
                if d.species == Species::Archaea
        )
    }

    fn archaea() -> ItemType {
        Item::organism(Species::Archaea, LifeStage::Adult, 1.0).r#type
    }

    // Advance one evolution step (v1: archaea only). An archaea on non-water
    // terrain dies; otherwise it may spread to a random neighbor that is water
    // and empty of micro. No spontaneous generation. Iterates over a snapshot
    // of the starting micro layer so a freshly-spread archaea isn't re-processed
    // this step.
    pub fn evolve(&mut self, rand: &mut Random) {
        let (width, height) = self.dimensions();
        if width == 0 || height == 0 {
            return;
        }
        let archaea_cells: Vec<(usize, usize)> = self
            .cells
            .iter()
            .enumerate()
            .flat_map(|(y, row)| {
                row.iter().enumerate().filter_map(move |(x, cell)| {
                    cell.micro
                        .is_some_and(Self::is_archaea)
                        .then_some((x, y))
                })
            })
            .collect();

        for (x, y) in archaea_cells {
            // 1. Archaea on non-water terrain dies.
            if !Self::terrain_is_water(self.cells[y][x].terrain) {
                self.cells[y][x].micro = None;
                continue;
            }
            // 2. Otherwise pick a random neighbor and spread into it if it is
            //    water and empty of micro.
            let (nx, ny) = self.random_neighbor(rand, (x, y));
            let neighbor = &self.cells[ny][nx];
            if Self::terrain_is_water(neighbor.terrain) && neighbor.micro.is_none()
            {
                self.cells[ny][nx].micro = Some(Self::archaea());
            }
        }
    }

    // A random neighbor of the given cell (may return the cell itself), clamped
    // to grid bounds.
    fn random_neighbor(
        &self,
        rand: &mut Random,
        (x, y): (usize, usize),
    ) -> (usize, usize) {
        let (width, height) = self.dimensions();
        (
            Self::random_1d(rand, x, width),
            Self::random_1d(rand, y, height),
        )
    }

    fn random_1d(rand: &mut Random, here: usize, bound: usize) -> usize {
        let mut v = here + (rand.next() as usize % 3);
        v = v.saturating_sub(1); // only go negative if possible
        if v >= bound {
            v = bound - 1;
        }
        v
    }

    // The number of distinct ItemTypes present across all layers of all cells
    // (terrain substances + organisms + other items). Drives leveling.
    pub fn distinct_complexity(&self) -> u8 {
        let mut seen: Vec<ItemType> = Vec::new();
        let add = |item: ItemType, seen: &mut Vec<ItemType>| {
            if !seen.contains(&item) {
                seen.push(item);
            }
        };
        for cell in self.cells.iter().flatten() {
            add(cell.terrain, &mut seen);
            for item in [cell.micro, cell.plant, cell.animal, cell.other]
                .into_iter()
                .flatten()
            {
                add(item, &mut seen);
            }
        }
        seen.len().min(u8::MAX as usize) as u8
    }

    // Remove and return the occupant of the highest occupied non-terrain layer
    // (other > animal > plant > micro). Terrain stays. Returns None if all
    // non-terrain layers are empty.
    pub fn extract_top(&mut self, x: u8, y: u8) -> Option<ItemType> {
        let (x, y) = (x as usize, y as usize);
        if y >= self.cells.len() || x >= self.cells[y].len() {
            return None;
        }
        let cell = &mut self.cells[y][x];
        if cell.other.is_some() {
            cell.other.take()
        } else if cell.animal.is_some() {
            cell.animal.take()
        } else if cell.plant.is_some() {
            cell.plant.take()
        } else {
            cell.micro.take()
        }
    }

    pub fn get_cell(&self, x: u8, y: u8) -> Option<&LandCell> {
        let (x, y) = (x as usize, y as usize);
        self.cells.get(y).and_then(|row| row.get(x))
    }
}

// Which layer of a cell an ingested item routes to.
#[derive(Debug, Clone, Copy)]
enum Layer {
    Terrain,
    Micro,
    Plant,
    Animal,
    Other,
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
                // Slightly smaller than the cell pitch so the grid reads as
                // distinct squares.
                custom_size: Some(CELL_AREA.dimensions() * 0.9),
                ..default()
            },
            transform: Transform::from_xyz(
                x as f32 * CELL_SIZE + dx,
                t_y as f32 * CELL_SIZE + dy,
                0.0,
            ),
        }
    }

    pub fn paint(
        entity: Entity,
        query: &mut Query<&mut Sprite, With<Cell>>,
        new_handle: Handle<Image>,
    ) {
        if let Ok(mut sprite) = query.get_mut(entity) {
            sprite.image = new_handle;
            sprite.color = Color::WHITE; // no tint — show the texture as-is
        }
    }
}

#[derive(Debug, Clone, Component)]
pub struct Cell {
    pub x: u8,
    pub y: u8,
}

// Cell was clicked: extract the topmost occupied non-terrain layer and eject it
// as a loose item. Terrain stays.
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
    cell_query: Query<(&Cell, &ChildOf, &GlobalTransform)>,
) {
    if !mouse_state.just_pressed {
        return;
    }

    let mouse_position = mouse_state.current_position;
    for (cell, cell_parent, cell_global_transform) in cell_query.iter() {
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
            let Minigame::Land(minigame) = minigame.into_inner() else {
                continue;
            };

            // Remove the top non-terrain occupant, if any.
            let Some(item_type) = minigame.extract_top(cell.x, cell.y) else {
                continue;
            };
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

// Run the evolution rules, gated by stored energy and a step interval so the
// simulation is watchable. Each step consumes one energy. Mirrors life.
pub fn evolve_fixed_update(
    mut commands: Commands,
    mut rand: ResMut<Random>,
    mut minigame_query: Query<(Entity, &mut Minigame)>,
    leveling_up_query: Query<&LevelingUp, With<Minigame>>,
) {
    for (entity, mut minigame) in minigame_query.iter_mut() {
        if leveling_up_query.get(entity).is_ok() {
            continue;
        }
        // Peek immutably first: skip non-Land and unfueled minigames without
        // marking them Changed.
        let Minigame::Land(land) = &*minigame else {
            continue;
        };
        if land.energy < 1.0 {
            continue;
        }
        let stepping = land.evolve_cooldown == 0;

        let Minigame::Land(land) = &mut *minigame else {
            continue;
        };
        if !stepping {
            land.evolve_cooldown -= 1;
            continue;
        }
        land.energy -= 1.0;
        land.evolve_cooldown = EVOLVE_TICKS;
        land.evolve(&mut rand);

        // Level up when the ecosystem grows more diverse (capped low for v1).
        let complexity = land.distinct_complexity();
        if complexity > land.max_achieved_complexity
            && land.max_achieved_complexity < MAX_LEVEL
        {
            land.max_achieved_complexity = complexity.min(MAX_LEVEL);
            commands.entity(entity).insert(LevelingUp);
        }
    }
}

// Repaint each cell every FixedUpdate to its topmost occupied layer's texture
// (other > animal > plant > micro), else the terrain's texture. Cells always
// show at least terrain. Cheap: Land grids are small.
pub fn render_cells(
    mut images: ResMut<Assets<Image>>,
    mut generated_image_assets: ResMut<image_gen::GeneratedImageAssets>,
    minigame_query: Query<(Entity, &Minigame)>,
    cell_query: Query<(Entity, &Cell, &ChildOf)>,
    mut cell_draw_query: Query<&mut Sprite, With<Cell>>,
) {
    for (minigame_entity, minigame) in minigame_query.iter() {
        let Minigame::Land(land) = minigame else {
            continue;
        };
        for (cell_entity, cell, cell_parent) in cell_query.iter() {
            if cell_parent.parent() != minigame_entity {
                continue;
            }
            let Some(land_cell) = land.get_cell(cell.x, cell.y) else {
                continue;
            };
            let texture = cell_texture(
                land_cell.top(),
                &mut images,
                &mut generated_image_assets,
            );
            CellBundle::paint(cell_entity, &mut cell_draw_query, texture);
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

    // A minigame with a fixed grid of all-mud cells, no growth involved.
    fn land(width: usize, height: usize) -> LandMinigame {
        let mud = LandMinigame::default_terrain();
        LandMinigame {
            level: 0,
            max_achieved_complexity: 0,
            energy: 0.0,
            cells: vec![vec![LandCell::new(mud); width]; height],
            evolve_cooldown: EVOLVE_TICKS,
        }
    }

    fn water() -> ItemType {
        Item::liquid(Substance::FreshWater, 1.0).r#type
    }

    fn ingest(land: &mut LandMinigame, item: &Item) -> f32 {
        let mut rand = Random::new(7);
        land.route(&mut rand, item)
    }

    impl LandMinigame {
        // Test-only routing helper that places without ECS (no ejection).
        fn route(&mut self, rand: &mut Random, item: &Item) -> f32 {
            let layer = match item.r#type {
                ItemType::Energy(_) => {
                    self.energy += item.amount;
                    return item.amount;
                }
                ItemType::Physical(PhysicalItem::Bulk(_)) => Layer::Terrain,
                ItemType::Physical(PhysicalItem::Discrete(d)) => {
                    match d.species.class() {
                        DiscreteClass::Microbe => Layer::Micro,
                        DiscreteClass::Plant => Layer::Plant,
                        DiscreteClass::Animal => Layer::Animal,
                        _ => Layer::Other,
                    }
                }
                _ => Layer::Other,
            };
            let (width, height) = self.dimensions();
            let x = (rand.next() as usize) % width;
            let y = (rand.next() as usize) % height;
            let cell = &mut self.cells[y][x];
            match layer {
                Layer::Terrain => cell.terrain = item.r#type,
                Layer::Micro => cell.micro = Some(item.r#type),
                Layer::Plant => cell.plant = Some(item.r#type),
                Layer::Animal => cell.animal = Some(item.r#type),
                Layer::Other => cell.other = Some(item.r#type),
            }
            1.0
        }
    }

    #[test]
    fn bulk_routes_to_terrain() {
        let mut l = land(1, 1);
        ingest(&mut l, &Item::liquid(Substance::FreshWater, 1.0));
        assert_eq!(l.cells[0][0].terrain, water());
        assert!(l.cells[0][0].micro.is_none());
    }

    #[test]
    fn archaea_routes_to_micro() {
        let mut l = land(1, 1);
        ingest(
            &mut l,
            &Item::organism(Species::Archaea, LifeStage::Adult, 1.0),
        );
        let cell = &l.cells[0][0];
        assert!(LandMinigame::is_archaea(cell.micro.unwrap()));
        assert!(cell.plant.is_none() && cell.animal.is_none());
    }

    #[test]
    fn fruit_routes_to_other() {
        let mut l = land(1, 1);
        ingest(&mut l, &Item::fruit(Species::Apple, 1.0));
        let cell = &l.cells[0][0];
        assert!(cell.other.is_some());
        assert!(cell.micro.is_none() && cell.plant.is_none());
    }

    #[test]
    fn archaea_on_non_water_dies() {
        // Single mud cell with an archaea: it should die (mud is not water).
        let mut l = land(1, 1);
        l.cells[0][0].micro = Some(LandMinigame::archaea());
        let mut rand = Random::new(1);
        l.evolve(&mut rand);
        assert!(l.cells[0][0].micro.is_none());
    }

    #[test]
    fn archaea_on_water_spreads_to_empty_water_neighbor() {
        // Two water cells in a row; archaea in the left one. It survives and
        // (eventually) spreads into the empty water neighbor.
        let mut l = land(2, 1);
        l.cells[0][0].terrain = water();
        l.cells[0][1].terrain = water();
        l.cells[0][0].micro = Some(LandMinigame::archaea());

        // random_neighbor can return self; loop a few times until it spreads.
        let mut rand = Random::new(1);
        let mut spread = false;
        for _ in 0..50 {
            l.evolve(&mut rand);
            if l.cells[0][1].micro.is_some() {
                spread = true;
                break;
            }
        }
        assert!(spread, "archaea never spread to the water neighbor");
        // The original archaea survives on water.
        assert!(l.cells[0][0].micro.is_some());
    }

    #[test]
    fn extract_removes_top_non_terrain_layer_first() {
        let mut l = land(1, 1);
        l.cells[0][0].micro = Some(LandMinigame::archaea());
        l.cells[0][0].other = Some(Item::fruit(Species::Apple, 1.0).r#type);

        // First click takes `other` (highest non-terrain), leaving micro.
        let first = l.extract_top(0, 0).unwrap();
        assert_eq!(first, Item::fruit(Species::Apple, 1.0).r#type);
        assert!(l.cells[0][0].other.is_none());
        assert!(l.cells[0][0].micro.is_some());

        // Second click takes micro.
        let second = l.extract_top(0, 0).unwrap();
        assert!(LandMinigame::is_archaea(second));
        assert!(l.cells[0][0].micro.is_none());

        // Terrain stays; nothing left to extract.
        assert!(l.extract_top(0, 0).is_none());
        assert_eq!(l.cells[0][0].terrain, LandMinigame::default_terrain());
    }

    #[test]
    fn distinct_complexity_counts_distinct_item_types() {
        let mut l = land(2, 1);
        // Both cells mud -> 1 distinct type.
        assert_eq!(l.distinct_complexity(), 1);
        l.cells[0][0].terrain = water(); // +water
        l.cells[0][0].micro = Some(LandMinigame::archaea()); // +archaea
        assert_eq!(l.distinct_complexity(), 3);
    }

    // --- Integration tests: drive the actual ECS systems through a real World —
    // the closest automated stand-in for a manual playthrough. They exercise the
    // wiring (systems run, queries match the spawned entities), evolution
    // stepping, rendering without panic, and click-to-extract. They do NOT cover
    // on-screen appearance, real input, or physics-collision-driven ingestion.

    // Spawn a Land minigame entity with its child Cell entities.
    fn spawn_land(world: &mut World, lm: LandMinigame, w: u8, h: u8) -> Entity {
        let mg = world
            .spawn((
                Minigame::Land(lm),
                GlobalTransform::default(),
                RectangularArea::new_square(1000.0),
            ))
            .id();
        let cells: Vec<Entity> = (0..h)
            .flat_map(|y| (0..w).map(move |x| (x, y)))
            .map(|(x, y)| {
                world
                    .spawn((
                        Cell { x, y },
                        Sprite::default(),
                        GlobalTransform::default(),
                    ))
                    .id()
            })
            .collect();
        world.entity_mut(mg).add_children(&cells);
        mg
    }

    #[test]
    fn evolve_and_render_run_through_the_ecs() {
        use bevy::ecs::system::RunSystemOnce;

        let mut world = World::new();
        world.insert_resource(Random::new(7));
        world.insert_resource(Assets::<Image>::default());
        world.insert_resource(image_gen::GeneratedImageAssets::default());

        // 2x2 with a water top row, an archaea seeded, and fuel.
        let mut lm = land(2, 2);
        lm.cells[0][0].terrain = water();
        lm.cells[0][1].terrain = water();
        lm.cells[0][0].micro = Some(LandMinigame::archaea());
        lm.energy = 100.0;
        let mg = spawn_land(&mut world, lm, 2, 2);

        // Enough fixed ticks to cross the cooldown and take a step.
        for _ in 0..(EVOLVE_TICKS + 5) {
            world.run_system_once(evolve_fixed_update).unwrap();
        }
        {
            let Some(Minigame::Land(land)) = world.get::<Minigame>(mg) else {
                panic!("land minigame missing");
            };
            assert!(land.energy < 100.0, "a step should have consumed energy");
        }

        // Render runs without panicking and paints the cell sprites.
        world.run_system_once(render_cells).unwrap();
        let mut sprites = world.query::<&Sprite>();
        assert!(
            sprites.iter(&world).any(|s| s.image != Handle::default()),
            "render_cells should set at least one cell texture"
        );
    }

    #[test]
    fn click_extracts_top_layer_through_the_ecs() {
        use bevy::ecs::system::RunSystemOnce;

        let mut world = World::new();
        world.insert_resource(Random::new(1));
        world.insert_resource(Assets::<Image>::default());
        world.insert_resource(image_gen::GeneratedImageAssets::default());
        let mut mouse = MouseState::new(1.0);
        mouse.just_pressed = true;
        mouse.current_position = Vec2::ZERO; // over the cell at the origin
        world.insert_resource(mouse);

        let mut lm = land(1, 1);
        lm.cells[0][0].other = Some(Item::fruit(Species::Apple, 1.0).r#type);
        let mg = spawn_land(&mut world, lm, 1, 1);

        world.run_system_once(cell_update).unwrap();

        {
            let Some(Minigame::Land(land)) = world.get::<Minigame>(mg) else {
                panic!("land minigame missing");
            };
            assert!(
                land.cells[0][0].other.is_none(),
                "the top (other) layer should have been extracted"
            );
        }
        let mut items = world.query::<&Item>();
        assert_eq!(
            items.iter(&world).count(),
            1,
            "extraction should eject one loose item"
        );
    }
}
