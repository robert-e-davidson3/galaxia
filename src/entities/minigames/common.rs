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

    pub fn level(&self) -> u8 {
        match self {
            Minigame::Button(m) => m.level(),
            Minigame::PrimordialOcean(m) => m.level(),
            Minigame::Rune(m) => m.level(),
            Minigame::BallBreaker(m) => m.level(),
        }
    }

    pub fn spawn(
        &self,
        commands: &mut Commands,
        transform: Transform,
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
                    // Minigame::Button(m) => m.spawn(parent),
                    Minigame::Rune(m) => m.spawn(parent),
                    // Minigame::PrimordialOcean(m) => m.spawn(parent),
                    // Minigame::BallBreaker(m) => m.spawn(parent),
                    _ => panic!("Minigame not implemented"),
                };
            })
            .id();

        entity
    }
}
