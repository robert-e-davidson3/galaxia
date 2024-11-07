use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use crate::entities::*;
use crate::libs::*;

pub const NAME: &str = "rune";
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
pub struct RuneMinigameBundle {
    pub minigame: RuneMinigame,
    pub area: RectangularArea,
    pub tag: Minigame,
    pub spatial: SpatialBundle,
}

impl RuneMinigameBundle {
    pub fn new(
        minigame: RuneMinigame,
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
pub struct RuneMinigame {
    pub level: u8,
    pub pixels: Vec<Vec<bool>>,
    pub erasing: bool,
}

impl RuneMinigame {
    pub fn new(level: u8) -> Self {
        if level > 99 {
            panic!("Invalid level: {}", level);
        }
        let blocks_per_row = Self::_blocks_per_row(level) as usize;
        let blocks_per_column = Self::_blocks_per_column(level) as usize;
        let pixels = vec![vec![false; blocks_per_row]; blocks_per_column];
        Self {
            level,
            pixels,
            erasing: false,
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

    pub fn set_pixel(&mut self, x: u8, y: u8, value: bool) {
        let (x, y) = (x as usize, y as usize);
        if y >= self.pixels.len() {
            return;
        }
        if x >= self.pixels[y].len() {
            return;
        }
        self.pixels[y][x] = value;
    }

    pub fn get_pixel(&self, x: u8, y: u8) -> bool {
        let (x, y) = (x as usize, y as usize);
        if y >= self.pixels.len() {
            return false;
        }
        if x >= self.pixels[y].len() {
            return false;
        }
        self.pixels[y][x]
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
    minigame: RuneMinigame,
) {
    let area = minigame.area();
    let blocks_per_row = minigame.blocks_per_row();
    let blocks_per_column = minigame.blocks_per_column();
    commands
        .spawn(RuneMinigameBundle::new(minigame, area, transform))
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

    pub fn turn_on(entity: Entity, query: &mut Query<&mut Fill, With<Pixel>>) {
        if let Ok(mut fill) = query.get_mut(entity) {
            fill.color = PIXEL_ON_COLOR;
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
    mouse_state: Res<MouseState>,
    time: Res<Time>,
    mut rune_minigame_query: Query<&mut RuneMinigame>,
    leveling_up_query: Query<&LevelingUp, With<RuneMinigame>>,
    ready_query: Query<&Ready, With<RuneMinigame>>,
    pixel_query: Query<(&Pixel, Entity, &Parent, &GlobalTransform)>,
    mut fill_query: Query<&mut Fill, With<Pixel>>,
) {
    // reset erasing state when mouse is released
    if mouse_state.just_released {
        for mut minigame in rune_minigame_query.iter_mut() {
            minigame.erasing = false;
        }
        return;
    }
    // only draw/erase when mouse is continuously pressed (dragging)
    if !mouse_state.dragging() {
        return;
    }

    let mouse_position = match mouse_state.current_position {
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
            mouse_position,
            pixel_global_transform.translation().truncate(),
        ) {
            let mut minigame =
                rune_minigame_query.get_mut(minigame_entity).unwrap();
            // set erasing state so player can draw/erase multiple pixels
            if mouse_state.just_pressed {
                if minigame.get_pixel(pixel.x, pixel.y) {
                    minigame.erasing = true;
                } else {
                    minigame.erasing = false;
                }
            } else if mouse_state.just_released {
                minigame.erasing = false;
            }
            // draw/erase pixel
            if minigame.erasing {
                PixelBundle::turn_off(pixel_entity, &mut fill_query);
                minigame.set_pixel(pixel.x, pixel.y, false);
            } else {
                PixelBundle::turn_on(pixel_entity, &mut fill_query);
                minigame.set_pixel(pixel.x, pixel.y, true);
            }
            // emit rune or get ready to
            // TODO visual change when drawing is a valid rune
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

const RUNE_TRIGGER_SECONDS: f32 = 2.0;

pub fn fixed_update(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut generated_image_assets: ResMut<image_gen::GeneratedImageAssets>,
    time: Res<Time>,
    mut rune_minigame_query: Query<(
        &mut RuneMinigame,
        &GlobalTransform,
        &RectangularArea,
    )>,
    leveling_up_query: Query<&LevelingUp, With<RuneMinigame>>,
    ready_query: Query<(&Ready, Entity), With<RuneMinigame>>,
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
                rune_minigame_query.get_mut(minigame_entity).unwrap();
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

pub fn levelup(
    mut commands: Commands,
    rune_minigame_query: Query<
        (&RuneMinigame, Entity, &Transform),
        With<LevelingUp>,
    >,
) {
    for (minigame, entity, transform) in rune_minigame_query.iter() {
        let level = if minigame.level < 99 {
            minigame.level + 1
        } else {
            99
        };
        commands.entity(entity).despawn_recursive();
        spawn(&mut commands, transform.clone(), RuneMinigame::new(level));
    }
}
