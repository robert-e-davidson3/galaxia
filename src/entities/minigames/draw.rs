use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use crate::entities::*;
use crate::libs::*;

pub const NAME: &str = "draw";
pub const _DESCRIPTION: &str = "Draw runes!";

pub const BLOCK_SIZE: f32 = 20.0;

#[derive(Debug, Clone, Bundle)]
pub struct DrawMinigameBundle {
    pub minigame: DrawMinigame,
    pub area: RectangularArea,
    pub tag: Minigame,
    pub spatial: SpatialBundle,
}

impl DrawMinigameBundle {
    pub fn new(
        minigame: DrawMinigame,
        area: RectangularArea,
        transform: Transform,
    ) -> Self {
        Self {
            minigame,
            area,
            tag: Minigame,
            spatial: SpatialBundle {
                transform,
                ..default()
            },
        }
    }
}

#[derive(Debug, Clone, Default, Component)]
pub struct DrawMinigame {
    pub level: u8,
    pub pixels: Vec<Vec<bool>>,
}

impl DrawMinigame {
    pub fn new(level: u8) -> Self {
        if level > 99 {
            panic!("Invalid level: {}", level);
        }
        let blocks_per_row = Self::_blocks_per_row(level) as usize;
        let blocks_per_column = Self::_blocks_per_column(level) as usize;
        let pixels = vec![vec![false; blocks_per_row]; blocks_per_column];
        Self { level, pixels }
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
        1 + level / 2
    }

    // level -> blocks_per_column
    // 0 -> 1
    // 1 -> 2
    // 2 -> 2
    fn _blocks_per_column(level: u8) -> u8 {
        if level % 2 == 0 {
            1 + level / 2
        } else {
            2 + level / 2
        }
    }

    const MIN_WIDTH: f32 = 100.0;
    const MIN_HEIGHT: f32 = 100.0;

    pub fn area(&self) -> RectangularArea {
        RectangularArea {
            width: Self::MIN_WIDTH
                .max(BLOCK_SIZE * self.blocks_per_row() as f32),
            height: Self::MIN_HEIGHT
                .max(BLOCK_SIZE * self.blocks_per_column() as f32),
        }
    }

    pub fn to_rune(&self) -> Option<Rune> {
        pixels_to_rune(self.pixels.clone())
    }

    pub fn toggle_pixel(&mut self, x: u8, y: u8) {
        if x as usize >= self.pixels.len() {
            return;
        }
        if y as usize >= self.pixels[x as usize].len() {
            return;
        }
        self.pixels[x as usize][y as usize] =
            !self.pixels[x as usize][y as usize];
    }

    pub fn clear(&mut self) {
        for row in self.pixels.iter_mut() {
            for pixel in row.iter_mut() {
                *pixel = false;
            }
        }
    }
}

// A Rune is a magical symbol that can be drawn in a Draw minigame.
// Each rune is a 2D grid of pixels, where each pixel can be on or off.
// For a Rune, only connected pixels are considered.
// Orientation also matters - a rune cannot be rotated or flipped.
#[repr(u8)]
pub enum Rune {
    // 1x1 pixels
    // magically, refers to the inclusive self
    InclusiveSelf,
    // 2x1
    // magically, acts as connector
    Connector,
    // 2x2
    // magically, refers to the EXCLUSIVE self
    ExclusiveSelf,
    // 3x2, missing middle bottom
    // magically, refers to shelter or protection
    Shelter,
    // 3x3, missing middle
    // magically, refers to the inclusive other (not-self)
    InclusiveOther,
    // 4x3 TODO
    // 4x4, missing middle
    // magically, refers to the EXCLUSIVE other (not-self)
    ExclusiveOther,
    // TODO: add more runes - at least 100 in total
    //       each expansion of space should require a new rune
}

const BIG_DOT_PATTERN: [[bool; 2]; 2] = [[true, true], [true, true]];
const SHELTER_PATTERN: [[bool; 3]; 2] =
    [[true, true, true], [true, false, true]];
const OTHER_PATTERN: [[bool; 3]; 3] =
    [[true, true, true], [true, false, true], [true, true, true]];
const EXCLUSIVE_OTHER_PATTERN: [[bool; 4]; 4] = [
    [true, true, true, true],
    [true, false, false, true],
    [true, false, false, true],
    [true, true, true, true],
];

fn pattern_to_pixels<const W: usize, const H: usize>(
    pattern: &[[bool; W]; H],
) -> Vec<Vec<bool>> {
    pattern.iter().map(|row| row.to_vec()).collect()
}

pub fn pixels_to_rune(pixels: Vec<Vec<bool>>) -> Option<Rune> {
    let pixels = strip_empty_rows(strip_empty_columns(pixels));
    if pixels.is_empty() {
        return None;
    }
    let width = pixels[0].len();
    let height = pixels.len();
    if width == 1 && height == 1 {
        return Some(Rune::InclusiveSelf);
    }
    if width == 2 && height == 1 {
        return Some(Rune::Connector);
    }
    if width == 2 && height == 2 {
        return (pattern_to_pixels(&BIG_DOT_PATTERN) == pixels)
            .then_some(Rune::ExclusiveSelf);
    }
    if width == 3 && height == 2 {
        return (pattern_to_pixels(&SHELTER_PATTERN) == pixels)
            .then_some(Rune::Shelter);
    }
    if width == 3 && height == 3 {
        return (pattern_to_pixels(&OTHER_PATTERN) == pixels)
            .then_some(Rune::InclusiveOther);
    }
    // TODO 4x3
    if width == 4 && height == 4 {
        return (pattern_to_pixels(&EXCLUSIVE_OTHER_PATTERN) == pixels)
            .then_some(Rune::ExclusiveOther);
    }

    None
}

fn strip_empty_rows(pixels: Vec<Vec<bool>>) -> Vec<Vec<bool>> {
    if pixels.is_empty() {
        return pixels;
    }

    let mut first_row = 0;
    let mut last_row = pixels.len();

    // Find first non-empty row
    while first_row < last_row && pixels[first_row].iter().all(|&p| !p) {
        first_row += 1;
    }

    // Find last non-empty row
    while last_row > first_row && pixels[last_row - 1].iter().all(|&p| !p) {
        last_row -= 1;
    }

    pixels[first_row..last_row].to_vec()
}

fn strip_empty_columns(pixels: Vec<Vec<bool>>) -> Vec<Vec<bool>> {
    if pixels.is_empty() || pixels[0].is_empty() {
        return pixels;
    }

    let width = pixels[0].len();
    let mut first_col = 0;
    let mut last_col = width;

    // Find first non-empty column
    'outer: while first_col < last_col {
        for row in &pixels {
            if row[first_col] {
                break 'outer;
            }
        }
        first_col += 1;
    }

    // Find last non-empty column
    'outer: while last_col > first_col {
        for row in &pixels {
            if row[last_col - 1] {
                break 'outer;
            }
        }
        last_col -= 1;
    }

    pixels
        .into_iter()
        .map(|row| row[first_col..last_col].to_vec())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_empty_rows() {
        let input = vec![
            vec![false, false],
            vec![false, true],
            vec![true, false],
            vec![false, false],
        ];
        let expected = vec![vec![false, true], vec![true, false]];
        assert_eq!(strip_empty_rows(input), expected);
    }

    #[test]
    fn test_strip_empty_columns() {
        let input = vec![
            vec![false, false, true, false],
            vec![false, true, false, false],
        ];
        let expected = vec![vec![false, true], vec![true, false]];
        assert_eq!(strip_empty_columns(input), expected);
    }

    #[test]
    fn test_empty_input() {
        let empty: Vec<Vec<bool>> = vec![];
        assert_eq!(strip_empty_rows(empty.clone()), empty.clone());
        assert_eq!(strip_empty_columns(empty.clone()), empty);
    }
}

pub fn spawn(
    commands: &mut Commands,
    transform: Transform,
    frozen: &DrawMinigame,
) {
    let minigame = frozen.clone();
    let area = minigame.area();
    let blocks_per_row = minigame.blocks_per_row();
    let blocks_per_column = minigame.blocks_per_column();
    println!(
        "Spawning draw minigame with {}x{} blocks",
        blocks_per_row, blocks_per_column
    );
    commands
        .spawn(DrawMinigameBundle::new(minigame, area, transform))
        .with_children(|parent| {
            let _background = parent.spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::WHITE,
                    custom_size: Some(area.into()),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, 0.0, -1.0),
                ..default()
            });
            parent.spawn(MinigameAuraBundle::new(parent.parent_entity(), area));
            spawn_minigame_container(parent, area, NAME);

            for y in 0..blocks_per_column {
                for x in 0..blocks_per_row {
                    println!("Spawning pixel at ({}, {})", x, y);
                    parent.spawn(PixelBundle::new(x, y));
                }
            }
        });
}

const PIXEL_SIZE: f32 = 10.0;
const PIXEL_AREA: RectangularArea = RectangularArea {
    width: PIXEL_SIZE,
    height: PIXEL_SIZE,
};
const PIXEL_ON_COLOR: Color = Color::srgb(1.0, 1.0, 1.0);
const PIXEL_OFF_COLOR: Color = Color::srgb(0.0, 0.0, 0.0);

#[derive(Bundle)]
pub struct PixelBundle {
    pub pixel: Pixel,
    pub toggleable: Toggleable,
    pub shape: ShapeBundle,
    pub fill: Fill,
}

impl PixelBundle {
    pub fn new(x: u8, y: u8) -> Self {
        Self {
            pixel: Pixel { x, y },
            toggleable: Toggleable::new(),
            shape: ShapeBundle {
                path: GeometryBuilder::build_as(&shapes::Rectangle {
                    extents: PIXEL_AREA.into(),
                    ..default()
                }),
                spatial: SpatialBundle {
                    transform: Transform::from_xyz(
                        PIXEL_SIZE * (x as f32),
                        PIXEL_SIZE * (y as f32),
                        0.0,
                    ),
                    ..default()
                },
                ..default()
            },
            fill: Fill::color(PIXEL_OFF_COLOR),
        }
    }

    pub fn toggle(entity: Entity, query: &mut Query<&mut Fill, With<Pixel>>) {
        if let Ok(mut fill) = query.get_mut(entity) {
            fill.color = match fill.color {
                PIXEL_ON_COLOR => PIXEL_OFF_COLOR,
                PIXEL_OFF_COLOR => PIXEL_ON_COLOR,
                _ => panic!("Invalid color"),
            };
        }
    }
}

#[derive(Debug, Clone, Component)]
pub struct Pixel {
    pub x: u8,
    pub y: u8,
}

// Pixel was clicked.
pub fn pixel_update(
    mut commands: Commands,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    window_query: Query<&Window>,
    mut draw_minigame_query: Query<&mut DrawMinigame>,
    ready_query: Query<&Ready, With<DrawMinigame>>,
    pixel_query: Query<(&Pixel, Entity, &Parent, &GlobalTransform)>,
    mut fill_query: Query<&mut Fill, With<Pixel>>,
) {
    let click_position = match get_click_release_position(
        camera_query,
        window_query,
        mouse_button_input,
    ) {
        Some(position) => position,
        None => return,
    };

    for (pixel, pixel_entity, pixel_parent, pixel_global_transform) in
        pixel_query.iter()
    {
        if PIXEL_AREA.is_within(
            click_position,
            pixel_global_transform.translation().truncate(),
        ) {
            let minigame_entity = pixel_parent.get();
            PixelBundle::toggle(pixel_entity, &mut fill_query);
            let mut minigame =
                draw_minigame_query.get_mut(minigame_entity).unwrap();
            minigame.toggle_pixel(pixel.x, pixel.y);
            if minigame.to_rune().is_some()
                && ready_query.get(minigame_entity).is_ok()
            {
                commands
                    .entity(minigame_entity)
                    .insert(Ready::new(time.elapsed_seconds()));
            }
        }
    }
}

const RUNE_TRIGGER_SECONDS: f32 = 2.0;

pub fn fixed_update(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut generated_image_assets: ResMut<image_gen::GeneratedImageAssets>,
    time: Res<Time>,
    mut draw_minigame_query: Query<(
        &mut DrawMinigame,
        &GlobalTransform,
        &RectangularArea,
    )>,
    ready_query: Query<(&Ready, Entity), With<DrawMinigame>>,
) {
    // TODO if minigame is Ready, and enough time has passed, emit rune then set pixels off
    for (ready, entity) in ready_query.iter() {
        if time.elapsed_seconds() - ready.since_time > RUNE_TRIGGER_SECONDS {
            commands.entity(entity).remove::<Ready>();
            let (mut minigame, minigame_transform, minigame_area) =
                draw_minigame_query.get_mut(entity).unwrap();
            minigame.clear();
            commands.spawn(ItemBundle::new_from_minigame(
                &mut images,
                &mut generated_image_assets,
                Item::new_abstract(AbstractItemKind::Rune, 0, 1.0),
                minigame_transform,
                minigame_area,
            ));
        }
    }
}
