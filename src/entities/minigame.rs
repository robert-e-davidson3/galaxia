use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::libs::*;

const META_HEIGHT: f32 = 25.0;
const BUTTON_WIDTH: f32 = 25.0;
const BUTTON_COUNT: f32 = 1.0;
const WALL_THICKNESS: f32 = 1.0;

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct MinigameTag;

#[derive(Debug, Bundle)]
pub struct MinigameAuraBundle {
    pub aura: MinigameAura,
    pub collider: Collider,
    pub sensor: Sensor,
    pub collision_groups: CollisionGroups,
    pub active_events: ActiveEvents,
    pub spatial: SpatialBundle,
}

impl MinigameAuraBundle {
    pub fn new(minigame: Entity, area: RectangularArea) -> Self {
        Self {
            aura: MinigameAura { minigame },
            collider: area.grow(1.0, 1.0).into(),
            sensor: Sensor,
            collision_groups: CollisionGroups::new(
                MINIGAME_AURA_GROUP,
                minigame_aura_filter(),
            ),
            active_events: ActiveEvents::COLLISION_EVENTS,
            spatial: SpatialBundle { ..default() },
        }
    }
}

#[derive(Debug, Copy, Clone, Component)]
pub struct MinigameAura {
    pub minigame: Entity,
}

// Draw bounds around the minigame, plus the meta buttons.
pub fn spawn_minigame_container(
    parent: &mut ChildBuilder,
    area: RectangularArea,
    name: &str,
    level: u8,
) {
    let minigame = parent.parent_entity();
    spawn_minigame_bounds(parent, area);
    let meta_area = RectangularArea {
        width: area.width,
        height: META_HEIGHT,
    };
    // Prevents player and resources from directly entering the minigame.
    // Necessary because resource speed can allow tunneling.
    parent.spawn((
        Collider::from(area),
        CollisionGroups::new(ETHER_GROUP, ether_filter()),
        SpatialBundle { ..default() },
    ));
    // Spawn the rest
    parent
        .spawn(SpatialBundle {
            transform: Transform::from_xyz(
                0.0,
                area.top() + META_HEIGHT / 2.0,
                0.0,
            ),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                ShapeBundle {
                    path: GeometryBuilder::build_as(&shapes::Rectangle {
                        extents: meta_area.into(),
                        ..default()
                    }),
                    spatial: SpatialBundle {
                        transform: Transform::from_xyz(
                            0.0, 0.0, -1.0, // background
                        ),
                        ..default()
                    },
                    ..default()
                },
                Fill::color(Color::WHITE),
                Stroke::new(Color::BLACK, WALL_THICKNESS),
            ));
            spawn_minigame_name(parent, name);
            spawn_minigame_buttons(parent, meta_area, minigame, level);
        });
}

pub fn spawn_minigame_name(parent: &mut ChildBuilder, name: &str) {
    parent.spawn(Text2dBundle {
        text: Text {
            sections: vec![TextSection {
                value: name.into(),
                style: TextStyle {
                    font_size: 24.0,
                    color: Color::BLACK,
                    ..default()
                },
            }],
            justify: JustifyText::Left,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(
                -(BUTTON_WIDTH * BUTTON_COUNT) / 2.0,
                0.0,
                0.0,
            ),
            ..default()
        },
        ..default()
    });
}

pub fn spawn_minigame_buttons(
    parent: &mut ChildBuilder,
    area: RectangularArea,
    minigame: Entity,
    level: u8,
) {
    spawn_minigame_engage_button(parent, area, minigame, level);
}

#[derive(Debug, Copy, Clone, Component)]
pub struct MinigameEngageButton {
    pub minigame: Entity,
}

#[derive(Debug, Copy, Clone, Resource)]
pub struct Engaged {
    pub game: Option<Entity>,
}

pub fn spawn_minigame_engage_button(
    parent: &mut ChildBuilder,
    area: RectangularArea,
    minigame: Entity,
    level: u8,
) {
    parent
        .spawn((
            MinigameEngageButton { minigame },
            Toggleable::new(),
            CircularArea { radius: 90.0 },
            ShapeBundle {
                path: GeometryBuilder::build_as(&shapes::Rectangle {
                    extents: Vec2::new(BUTTON_WIDTH, META_HEIGHT),
                    ..default()
                }),
                spatial: SpatialBundle {
                    transform: Transform::from_xyz(
                        area.right() - BUTTON_WIDTH / 2.0,
                        0.0,
                        0.0,
                    ),
                    ..default()
                },
                ..default()
            },
            Fill::color(Color::srgba(0.2, 0.8, 0.8, 1.0)),
            Stroke::new(Color::BLACK, 1.0),
            RectangularArea {
                width: BUTTON_WIDTH,
                height: META_HEIGHT,
            },
        ))
        .with_children(|parent| {
            parent.spawn(Text2dBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: format!("{}", level).into(),
                        style: TextStyle {
                            font_size: 24.0,
                            color: Color::BLACK,
                            ..default()
                        },
                    }],
                    justify: JustifyText::Center,
                    ..default()
                },
                transform: Transform::from_xyz(0.0, 0.0, 1.0),
                ..default()
            });
        });
}

pub fn engage_button_update(
    mut button_query: Query<(
        &MinigameEngageButton,
        &mut Toggleable,
        &mut Fill,
        &GlobalTransform,
        &RectangularArea,
    )>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    window_query: Query<&Window>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut engaged: ResMut<Engaged>,
) {
    let click_position = match get_click_release_position(
        camera_query,
        window_query,
        mouse_button_input,
    ) {
        Some(world_position) => world_position,
        None => return,
    };

    for (engage_button, mut toggle, mut fill, global_transform, area) in
        button_query.iter_mut()
    {
        if area.is_within(
            click_position,
            global_transform.translation().truncate(),
        ) {
            if toggle.active {
                engaged.game = None;
                fill.color.set_alpha(1.0);
            } else {
                engaged.game = Some(engage_button.minigame);
                fill.color.set_alpha(0.8);
            }
            toggle.toggle();
        }
    }
}

#[derive(Bundle)]
pub struct MinigameBoundBundle {
    pub transform: TransformBundle,
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
    pub rigid_body: RigidBody,
    pub dominance: Dominance,
}

impl MinigameBoundBundle {
    pub fn horizontal(
        x_offset: f32,
        y_offset: f32,
        length: f32,
        thickness: f32,
    ) -> Self {
        Self::build(x_offset, y_offset, length, thickness)
    }

    pub fn vertical(
        x_offset: f32,
        y_offset: f32,
        length: f32,
        thickness: f32,
    ) -> Self {
        Self::build(x_offset, y_offset, thickness, length)
    }

    fn build(x_offset: f32, y_offset: f32, width: f32, height: f32) -> Self {
        Self {
            transform: TransformBundle::from(Transform::from_xyz(
                x_offset, y_offset, 0.0,
            )),
            collider: Collider::cuboid(width / 2.0, height / 2.0),
            collision_groups: CollisionGroups::new(
                BORDER_GROUP,
                border_filter(),
            ),
            rigid_body: RigidBody::Fixed,
            dominance: Dominance { groups: 2 },
        }
    }
}

pub fn spawn_minigame_bounds(parent: &mut ChildBuilder, area: RectangularArea) {
    parent
        .spawn((
            ShapeBundle {
                path: GeometryBuilder::build_as(&shapes::Rectangle {
                    extents: Vec2::new(area.width, area.height + META_HEIGHT),
                    origin: RectangleOrigin::CustomCenter(Vec2::new(
                        0.0,
                        META_HEIGHT / 2.0,
                    )),
                }),
                ..Default::default()
            },
            Fill::color(Color::NONE),
            Stroke::new(Color::BLACK, WALL_THICKNESS),
        ))
        .with_children(|parent| {
            // top wall
            parent.spawn(MinigameBoundBundle::horizontal(
                0.0,
                (area.height / 2.0) + META_HEIGHT,
                area.width,
                WALL_THICKNESS,
            ));
            // divider wall
            parent.spawn(MinigameBoundBundle::horizontal(
                0.0,
                area.height / 2.0,
                area.width,
                WALL_THICKNESS,
            ));
            // bottom wall
            parent.spawn(MinigameBoundBundle::horizontal(
                0.0,
                -area.height / 2.0,
                area.width,
                WALL_THICKNESS,
            ));
            // left wall
            parent.spawn(MinigameBoundBundle::vertical(
                -area.width / 2.0,
                META_HEIGHT / 2.0,
                area.height + META_HEIGHT,
                WALL_THICKNESS,
            ));
            // right wall
            parent.spawn(MinigameBoundBundle::vertical(
                area.width / 2.0,
                META_HEIGHT / 2.0,
                area.height + META_HEIGHT,
                WALL_THICKNESS,
            ));
        });
}

#[derive(Debug, Copy, Clone, Component)]
pub struct LevelingUp {
    pub minigame: Entity,
}
