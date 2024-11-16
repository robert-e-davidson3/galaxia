use std::collections::HashMap;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::libs::*;
use crate::minigames::*;

#[derive(Debug, Bundle)]
pub struct MinigameBundle {
    pub minigame: Minigame,
    pub spatial: SpatialBundle,
    pub area: RectangularArea,
}

impl MinigameBundle {
    pub fn new(minigame: Minigame, transform: Transform) -> Self {
        let area = minigame.area();
        MinigameBundle {
            minigame,
            spatial: SpatialBundle {
                transform,
                ..default()
            },
            area,
        }
    }
}

#[derive(Debug, Clone, Component)]
pub enum Minigame {
    Button(button::ButtonMinigame),
    PrimordialOcean(primordial_ocean::PrimordialOceanMinigame),
    Rune(rune::RuneMinigame),
    Chest(chest::ChestMinigame),
    Battery(battery::BatteryMinigame),
    BallBreaker(ball_breaker::BallBreakerMinigame),
    Tree(tree::TreeMinigame),
}

impl Minigame {
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            button::ID => {
                Some(Minigame::Button(button::ButtonMinigame::default()))
            }
            primordial_ocean::ID => Some(Minigame::PrimordialOcean(
                primordial_ocean::PrimordialOceanMinigame::default(),
            )),
            rune::ID => Some(Minigame::Rune(rune::RuneMinigame::default())),
            chest::ID => Some(Minigame::Chest(chest::ChestMinigame::default())),
            battery::ID => {
                Some(Minigame::Battery(battery::BatteryMinigame::default()))
            }
            ball_breaker::ID => Some(Minigame::BallBreaker(
                ball_breaker::BallBreakerMinigame::default(),
            )),
            _ => None,
        }
    }

    pub fn id(&self) -> &str {
        match self {
            Minigame::Button(_) => button::ID,
            Minigame::PrimordialOcean(_) => primordial_ocean::ID,
            Minigame::Rune(_) => rune::ID,
            Minigame::Chest(_) => chest::ID,
            Minigame::Battery(_) => battery::ID,
            Minigame::BallBreaker(_) => ball_breaker::ID,
            Minigame::Tree(_) => tree::ID,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Minigame::Button(m) => m.name(),
            Minigame::PrimordialOcean(m) => m.name(),
            Minigame::Rune(m) => m.name(),
            Minigame::Chest(m) => m.name(),
            Minigame::Battery(m) => m.name(),
            Minigame::BallBreaker(m) => m.name(),
            Minigame::Tree(m) => m.name(),
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Minigame::Button(m) => m.description(),
            Minigame::PrimordialOcean(m) => m.description(),
            Minigame::Rune(m) => m.description(),
            Minigame::Chest(m) => m.description(),
            Minigame::Battery(m) => m.description(),
            Minigame::BallBreaker(m) => m.description(),
            Minigame::Tree(m) => m.description(),
        }
    }

    pub fn position(&self) -> Vec2 {
        match self {
            Minigame::Button(_) => button::POSITION,
            Minigame::PrimordialOcean(_) => primordial_ocean::POSITION,
            Minigame::Rune(_) => rune::POSITION,
            Minigame::Chest(_) => chest::POSITION,
            Minigame::Battery(_) => battery::POSITION,
            Minigame::BallBreaker(_) => ball_breaker::POSITION,
            Minigame::Tree(_) => tree::POSITION,
        }
    }

    pub fn area(&self) -> RectangularArea {
        match self {
            Minigame::Button(m) => m.area(),
            Minigame::PrimordialOcean(m) => m.area(),
            Minigame::Rune(m) => m.area(),
            Minigame::Chest(m) => m.area(),
            Minigame::Battery(m) => m.area(),
            Minigame::BallBreaker(m) => m.area(),
            Minigame::Tree(m) => m.area(),
        }
    }

    // The level the minigame currently has.
    pub fn level(&self) -> u8 {
        match self {
            Minigame::Button(m) => m.level(),
            Minigame::PrimordialOcean(m) => m.level(),
            Minigame::Rune(m) => m.level(),
            Minigame::Chest(m) => m.level(),
            Minigame::Battery(m) => m.level(),
            Minigame::BallBreaker(m) => m.level(),
            Minigame::Tree(m) => m.level(),
        }
    }

    // Recreate minigame with correct new level, by its internal logic.
    pub fn levelup(&self) -> Self {
        match self {
            Minigame::Button(m) => Minigame::Button(m.levelup()),
            Minigame::PrimordialOcean(m) => {
                Minigame::PrimordialOcean(m.levelup())
            }
            Minigame::Rune(m) => Minigame::Rune(m.levelup()),
            Minigame::Chest(m) => Minigame::Chest(m.levelup()),
            Minigame::Battery(m) => Minigame::Battery(m.levelup()),
            Minigame::BallBreaker(m) => Minigame::BallBreaker(m.levelup()),
            Minigame::Tree(m) => Minigame::Tree(m.levelup()),
        }
    }

    pub fn spawn(
        &self,
        commands: &mut Commands,
        transform: Transform,
        random: &mut Random,
        asset_server: &AssetServer,
        _images: &mut Assets<Image>,
        _generated_image_assets: &mut image_gen::GeneratedImageAssets,
    ) -> Entity {
        let area = self.area();
        let name = self.name();
        let description = self.description();
        let level = self.level();
        let mut new_minigame = self.clone();
        let entity = commands
            .spawn_empty()
            .with_children(|parent| {
                spawn_minigame_container(
                    parent,
                    area,
                    name.into(),
                    description,
                    level,
                );
                parent.spawn(MinigameAuraBundle::new(
                    parent.parent_entity(),
                    area,
                ));
                match &mut new_minigame {
                    Minigame::Button(m) => m.spawn(parent),
                    Minigame::Rune(m) => m.spawn(parent),
                    Minigame::PrimordialOcean(m) => m.spawn(parent),
                    Minigame::Chest(m) => m.spawn(parent, asset_server),
                    Minigame::Battery(m) => m.spawn(parent, asset_server),
                    Minigame::BallBreaker(m) => {
                        m.spawn(parent, random, asset_server)
                    }
                    Minigame::Tree(m) => m.spawn(parent, asset_server),
                };
            })
            .id();
        // TODO somehow clear everything from this space first
        commands
            .entity(entity)
            .insert(MinigameBundle::new(new_minigame, transform));
        entity
    }
}

// Respawn leveled-up minigames.
// Spawn unlocked minigames.
pub fn levelup(
    mut commands: Commands,
    mut random: ResMut<Random>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut generated_image_assets: ResMut<image_gen::GeneratedImageAssets>,
    mut minigames: ResMut<MinigamesResource>,
    mut query: Query<(&mut Minigame, &Transform, Entity), With<LevelingUp>>,
) {
    for (minigame, transform, entity) in query.iter_mut() {
        let new_minigame = minigame.levelup();
        commands.entity(entity).despawn_recursive();
        new_minigame.spawn(
            &mut commands,
            *transform,
            &mut random,
            &asset_server,
            &mut images,
            &mut generated_image_assets,
        );
        // Update minigame level
        minigames.set_level(&new_minigame);
        // Unlock minigames
        for id in minigames.to_unlock(&minigame.id().into()) {
            match Minigame::from_id(&id) {
                Some(unlocked_minigame) => {
                    let pos = unlocked_minigame.position();
                    let entity = unlocked_minigame.spawn(
                        &mut commands,
                        Transform::from_translation(Vec3::new(
                            pos.x, pos.y, 0.0,
                        )),
                        &mut random,
                        &asset_server,
                        &mut images,
                        &mut generated_image_assets,
                    );
                    minigames.set_entity(&id, entity);
                }
                None => {}
            }
        }
    }
}

#[derive(Debug, Copy, Clone, Component)]
pub struct LevelingUp;

const META_HEIGHT: f32 = 25.0;
const BUTTON_WIDTH: f32 = 25.0;
const BUTTON_COUNT: f32 = 1.0;
const WALL_THICKNESS: f32 = 1.0;

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
    description: &str,
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
            spawn_minigame_name(parent, name, &area);
            spawn_minigame_buttons(
                parent,
                meta_area,
                minigame,
                level,
                description,
            );
        });
}

pub fn spawn_minigame_name(
    parent: &mut ChildBuilder,
    name: &str,
    area: &RectangularArea,
) {
    // set font size so it fits in the space
    let font_size = (area.width / name.len() as f32).clamp(10.0, 24.0);
    parent.spawn(Text2dBundle {
        text: Text {
            sections: vec![TextSection {
                value: name.into(),
                style: TextStyle {
                    font_size,
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
    description: &str,
) {
    spawn_minigame_engage_button(parent, area, minigame, level, description);
}

#[derive(Debug, Clone, Default, Resource)]
pub struct MinigamesResource(
    HashMap<String, (Option<Entity>, u8, Vec<Prerequisite>)>,
);

impl MinigamesResource {
    pub fn insert(&mut self, id: &str, prerequisites: Vec<Prerequisite>) {
        self.0.insert(id.into(), (None, 0, prerequisites));
    }

    pub fn set_level(&mut self, minigame: &Minigame) {
        self.0.get_mut(minigame.id()).map(|(_, level, _)| {
            *level += 1;
        });
    }

    pub fn level(&self, minigame: &String) -> u8 {
        self.0
            .get(minigame)
            .map(|(_, level, _)| *level)
            .unwrap_or(0)
    }

    pub fn set_entity(&mut self, minigame: &String, entity: Entity) {
        self.0.get_mut(minigame).map(|(e, _, _)| {
            *e = Some(entity);
        });
    }

    pub fn entity(&self, minigame: &String) -> Option<Entity> {
        self.0
            .get(minigame)
            .map(|(entity, _, _)| *entity)
            .unwrap_or(None)
    }

    pub fn is_unlocked(&self, minigame: &String) -> bool {
        self.entity(minigame).is_some()
    }

    pub fn prerequisites(&self, minigame: &String) -> Vec<Prerequisite> {
        self.0
            .get(minigame)
            .map(|(_, _, prerequisites)| prerequisites.clone())
            .unwrap_or_default()
    }

    // Given the leveled-up minigame, return minigames to unlock.
    // Only returns minigames that are not already unlocked.
    pub fn to_unlock(&self, minigame: &String) -> Vec<String> {
        self.unlocked_by(minigame)
            .iter()
            .filter(|minigame| self.needs_to_unlock(minigame))
            .cloned()
            .collect()
    }

    pub fn needs_to_unlock(&self, minigame: &String) -> bool {
        if self.is_unlocked(minigame) {
            return false;
        }
        self.prerequisites(minigame).iter().all(|prerequisite| {
            self.is_unlocked(&prerequisite.minigame)
                || self.level(&prerequisite.minigame) >= prerequisite.level
        })
    }

    // Reverse-lookup for prerequisites
    fn unlocked_by(&self, minigame: &String) -> Vec<String> {
        self.0
            .iter()
            .filter_map(|(key, (_, _, prerequisites))| {
                if prerequisites
                    .iter()
                    .any(|prerequisite| prerequisite.minigame == *minigame)
                {
                    Some(key.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct Prerequisite {
    pub minigame: String,
    pub level: u8,
}

pub fn setup_minigame_unlocks(mut unlocks: ResMut<MinigamesResource>) {
    unlocks.insert(button::ID, Vec::new());
    unlocks.insert(primordial_ocean::ID, Vec::new());
    unlocks.insert(rune::ID, Vec::new());

    unlocks.insert(
        chest::ID,
        vec![
            Prerequisite {
                minigame: button::ID.into(),
                level: 1,
            },
            Prerequisite {
                minigame: primordial_ocean::ID.into(),
                level: 1,
            },
        ],
    );
    unlocks.insert(
        battery::ID,
        vec![
            Prerequisite {
                minigame: rune::ID.into(),
                level: 1,
            },
            Prerequisite {
                minigame: primordial_ocean::ID.into(),
                level: 1,
            },
        ],
    );
    unlocks.insert(
        ball_breaker::ID,
        vec![Prerequisite {
            minigame: button::ID.into(),
            level: 1,
        }],
    );
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
    description: &str,
) {
    parent
        .spawn((
            MinigameEngageButton { minigame },
            Toggleable::new(),
            CircularArea { radius: 90.0 },
            HoverText::new(description.into()),
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
