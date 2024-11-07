use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use crate::entities::*;
use crate::libs::*;

pub const NAME: &str = "draw";
pub const _DESCRIPTION: &str = "Draw runes!";

const MIN_WIDTH: f32 = 100.0;
const MIN_HEIGHT: f32 = 100.0;
const PIXEL_SIZE: f32 = 25.0;
const PIXEL_AREA: RectangularArea = RectangularArea {
    width: PIXEL_SIZE,
    height: PIXEL_SIZE,
};
const PIXEL_ON_COLOR: Color = Color::srgb(0.0, 0.0, 0.0);
const PIXEL_OFF_COLOR: Color = Color::srgb(1.0, 1.0, 1.0);

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

    pub fn area(&self) -> RectangularArea {
        const BUFFER: f32 = 20.0;
        let blocks_per_row = self.blocks_per_row();
        let blocks_per_column = self.blocks_per_column();
        RectangularArea {
            width: BUFFER + MIN_WIDTH.max(PIXEL_SIZE * blocks_per_row as f32),
            height: BUFFER
                + MIN_HEIGHT.max(PIXEL_SIZE * blocks_per_column as f32),
        }
    }

    pub fn to_rune(&self) -> Option<rune::Rune> {
        rune::pixels_to_rune(&self.pixels)
    }

    pub fn toggle_pixel(&mut self, x: u8, y: u8) {
        let (x, y) = (x as usize, y as usize);
        if y >= self.pixels.len() {
            return;
        }
        if x >= self.pixels[y].len() {
            return;
        }
        self.pixels[y][x] = !self.pixels[y][x];
    }

    pub fn clear(&mut self) {
        for row in self.pixels.iter_mut() {
            for pixel in row.iter_mut() {
                *pixel = false;
            }
        }
    }
}

pub fn spawn(
    commands: &mut Commands,
    transform: Transform,
    minigame: DrawMinigame,
) {
    let area = minigame.area();
    let blocks_per_row = minigame.blocks_per_row();
    let blocks_per_column = minigame.blocks_per_column();
    commands
        .spawn(DrawMinigameBundle::new(minigame, area, transform))
        .with_children(|parent| {
            let _background = parent.spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(0.5, 0.5, 0.5),
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
                    parent.spawn(PixelBundle::new(
                        x,
                        y,
                        blocks_per_row,
                        blocks_per_column,
                    ));
                }
            }
        });
}

#[derive(Bundle)]
pub struct PixelBundle {
    pub pixel: Pixel,
    pub toggleable: Toggleable,
    pub shape: ShapeBundle,
    pub fill: Fill,
}

impl PixelBundle {
    pub fn new(x: u8, y: u8, cols: u8, rows: u8) -> Self {
        let t_y = rows - y; // top to bottom
        let dx = -PIXEL_SIZE * ((cols - 1) as f32 / 2.0);
        let dy = -PIXEL_SIZE * ((rows + 1) as f32 / 2.0);
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
                        x as f32 * PIXEL_SIZE + dx,
                        t_y as f32 * PIXEL_SIZE + dy,
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

    pub fn turn_off(entity: Entity, query: &mut Query<&mut Fill, With<Pixel>>) {
        if let Ok(mut fill) = query.get_mut(entity) {
            fill.color = PIXEL_OFF_COLOR;
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
    leveling_up_query: Query<&LevelingUp, With<DrawMinigame>>,
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
        let minigame_entity = pixel_parent.get();
        if leveling_up_query.get(minigame_entity).is_ok() {
            continue;
        }
        if PIXEL_AREA.is_within(
            click_position,
            pixel_global_transform.translation().truncate(),
        ) {
            let mut minigame =
                draw_minigame_query.get_mut(minigame_entity).unwrap();
            PixelBundle::toggle(pixel_entity, &mut fill_query);
            minigame.toggle_pixel(pixel.x, pixel.y);
            let is_ready = ready_query.get(minigame_entity).is_ok();
            match minigame.to_rune() {
                Some(_) => {
                    if !is_ready {
                        commands
                            .entity(minigame_entity)
                            .insert(Ready::new(time.elapsed_seconds()));
                    }
                }
                None => {
                    if is_ready {
                        commands.entity(minigame_entity).remove::<Ready>();
                    }
                }
            }
        }
    }
}

pub fn levelup(
    mut commands: Commands,
    draw_minigame_query: Query<
        (&DrawMinigame, Entity, &Transform),
        With<LevelingUp>,
    >,
) {
    for (minigame, entity, transform) in draw_minigame_query.iter() {
        let level = if minigame.level < 99 {
            minigame.level + 1
        } else {
            99
        };
        commands.entity(entity).despawn_recursive();
        spawn(&mut commands, transform.clone(), DrawMinigame::new(level));
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
    leveling_up_query: Query<&LevelingUp, With<DrawMinigame>>,
    ready_query: Query<(&Ready, Entity), With<DrawMinigame>>,
    pixel_query: Query<(Entity, &Parent)>,
    mut fill_query: Query<&mut Fill, With<Pixel>>,
) {
    for (ready, minigame_entity) in ready_query.iter() {
        if leveling_up_query.get(minigame_entity).is_ok() {
            continue;
        }
        if time.elapsed_seconds() - ready.since_time > RUNE_TRIGGER_SECONDS {
            commands.entity(minigame_entity).remove::<Ready>();
            let (mut minigame, minigame_transform, minigame_area) =
                draw_minigame_query.get_mut(minigame_entity).unwrap();
            match minigame.to_rune() {
                Some(rune) => {
                    for (pixel_entity, pixel_parent) in pixel_query.iter() {
                        if pixel_parent.get() == minigame_entity {
                            PixelBundle::turn_off(
                                pixel_entity,
                                &mut fill_query,
                            );
                        }
                    }
                    minigame.clear();
                    commands.spawn(ItemBundle::new_from_minigame(
                        &mut images,
                        &mut generated_image_assets,
                        Item::new_abstract(
                            AbstractItemKind::Rune,
                            rune as u8,
                            1.0,
                        ),
                        minigame_transform,
                        minigame_area,
                    ));
                    if rune as u8 == minigame.level {
                        commands.entity(minigame_entity).insert(LevelingUp {
                            minigame: minigame_entity,
                        });
                    }
                }
                None => {}
            }
        }
    }
}