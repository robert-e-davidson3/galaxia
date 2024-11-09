use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::entities::minigame::{spawn_minigame_container, MinigameAuraBundle};
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
    BallBreaker(ball_breaker::BallBreakerMinigame),
}

impl Minigame {
    pub fn name(&self) -> &str {
        match self {
            Minigame::Button(m) => m.name(),
            Minigame::PrimordialOcean(m) => m.name(),
            Minigame::Rune(m) => m.name(),
            Minigame::BallBreaker(m) => m.name(),
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Minigame::Button(m) => m.description(),
            Minigame::PrimordialOcean(m) => m.description(),
            Minigame::Rune(m) => m.description(),
            Minigame::BallBreaker(m) => m.description(),
        }
    }

    pub fn area(&self) -> RectangularArea {
        match self {
            Minigame::Button(m) => m.area(),
            Minigame::PrimordialOcean(m) => m.area(),
            Minigame::Rune(m) => m.area(),
            Minigame::BallBreaker(m) => m.area(),
        }
    }

    // The level the minigame currently has.
    pub fn level(&self) -> u8 {
        match self {
            Minigame::Button(m) => m.level(),
            Minigame::PrimordialOcean(m) => m.level(),
            Minigame::Rune(m) => m.level(),
            Minigame::BallBreaker(m) => m.level(),
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
            Minigame::BallBreaker(m) => Minigame::BallBreaker(m.levelup()),
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
        let level = self.level();
        let entity = commands
            .spawn(MinigameBundle::new(self.clone(), transform))
            .with_children(|parent| {
                spawn_minigame_container(parent, area, name.into(), level);
                parent.spawn(MinigameAuraBundle::new(
                    parent.parent_entity(),
                    area,
                ));
                match self {
                    Minigame::Button(m) => m.spawn(parent),
                    Minigame::Rune(m) => m.spawn(parent),
                    Minigame::PrimordialOcean(m) => m.spawn(parent),
                    Minigame::BallBreaker(m) => {
                        m.spawn(parent, random, asset_server)
                    }

                    _ => panic!("Minigame not implemented"),
                };
            })
            .id();

        entity
    }
}

pub fn levelup(
    mut commands: Commands,
    mut random: ResMut<Random>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut generated_image_assets: ResMut<image_gen::GeneratedImageAssets>,
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
    }
}

#[derive(Debug, Copy, Clone, Component)]
pub struct LevelingUp;
