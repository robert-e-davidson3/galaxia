use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::area::*;

pub const MAX_RESOURCE_DISTANCE: f32 = 10000.0;

#[derive(Debug, Component)]
#[component(storage = "SparseSet")]
pub struct LooseResource {
    pub resource: GalaxiaResource,
    pub amount: f32,
}

#[derive(Debug, Copy, Clone, Component)]
pub struct Stuck {
    pub player: Entity,
}

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct Sticky;

pub fn spawn_loose_resource(
    commands: &mut Commands,
    asset_server: &AssetServer,
    resource: GalaxiaResource,
    amount: f32,
    transform: Transform,
) {
    let radius = 10.0 + (amount / 1_000_000.0);
    let area = CircularArea { radius };
    commands.spawn((
        LooseResource { resource, amount },
        area,
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(radius * 2.0, radius * 2.0)),
                ..default()
            },
            texture: asset_server.load(resource_to_asset(resource)),
            transform,
            ..default()
        },
        RigidBody::Dynamic,
        Ccd::enabled(),
        Collider::from(area),
        Damping {
            linear_damping: 1.0,
            angular_damping: 1.0,
        },
        Velocity::linear(Vec2::new(70.0, -70.0)),
    ));
}

pub fn despawn_distant_loose_resources(
    mut commands: Commands,
    query: Query<(Entity, &Transform), (With<LooseResource>, Without<Stuck>)>,
) {
    for (entity, transform) in query.iter() {
        if transform.translation.length() > MAX_RESOURCE_DISTANCE {
            commands.entity(entity).despawn();
        }
    }
}

pub enum ResourceKind {
    Abstract,
    Solid,
    Liquid,
    Gas,
    Mana,
    Energy,
    Heat,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum GalaxiaResource {
    // abstract
    ShortLeftClick,
    LongLeftClick,

    // solid
    Apple,
    Lemon,
    Lime,
    Mud,
    Dirt,
    Sandstone,
    Granite,
    Marble,
    Obsidian,
    Copper,
    Tin,
    Iron,
    Silver,
    Gold,
    Diamond,
    Amethyst,
    Moss,
    Unobtainium,

    // liquid
    SaltWater,
    FreshWater,
    // gas
    // mana
    // energy
    // heat
}

pub fn resource_to_kind(resource: GalaxiaResource) -> ResourceKind {
    match resource {
        // abstract
        GalaxiaResource::ShortLeftClick => ResourceKind::Abstract,
        GalaxiaResource::LongLeftClick => ResourceKind::Abstract,
        // solid
        GalaxiaResource::Apple => ResourceKind::Solid,
        GalaxiaResource::Lemon => ResourceKind::Solid,
        GalaxiaResource::Lime => ResourceKind::Solid,
        GalaxiaResource::Mud => ResourceKind::Solid,
        GalaxiaResource::Dirt => ResourceKind::Solid,
        GalaxiaResource::Sandstone => ResourceKind::Solid,
        GalaxiaResource::Granite => ResourceKind::Solid,
        GalaxiaResource::Marble => ResourceKind::Solid,
        GalaxiaResource::Obsidian => ResourceKind::Solid,
        GalaxiaResource::Copper => ResourceKind::Solid,
        GalaxiaResource::Tin => ResourceKind::Solid,
        GalaxiaResource::Iron => ResourceKind::Solid,
        GalaxiaResource::Silver => ResourceKind::Solid,
        GalaxiaResource::Gold => ResourceKind::Solid,
        GalaxiaResource::Diamond => ResourceKind::Solid,
        GalaxiaResource::Amethyst => ResourceKind::Solid,
        GalaxiaResource::Moss => ResourceKind::Solid,
        GalaxiaResource::Unobtainium => ResourceKind::Solid,
        // liquid
        GalaxiaResource::SaltWater => ResourceKind::Liquid,
        GalaxiaResource::FreshWater => ResourceKind::Liquid,
        // gas
        // mana
        // energy
        // heat
    }
}

pub fn resource_to_asset(resource: GalaxiaResource) -> String {
    match resource {
        // abstract
        GalaxiaResource::ShortLeftClick => {
            "abstract/short_left_click.png".to_string()
        }
        GalaxiaResource::LongLeftClick => {
            "abstract/long_left_click.png".to_string()
        }
        // solid
        GalaxiaResource::Apple => "solid/apple.png".to_string(),
        GalaxiaResource::Lemon => "solid/lemon.png".to_string(),
        GalaxiaResource::Lime => "solid/lime.png".to_string(),
        GalaxiaResource::Mud => "solid/mud.png".to_string(),
        GalaxiaResource::Dirt => "solid/dirt.png".to_string(),
        GalaxiaResource::Sandstone => "solid/sandstone.png".to_string(),
        GalaxiaResource::Granite => "solid/granite.png".to_string(),
        GalaxiaResource::Marble => "solid/marble.png".to_string(),
        GalaxiaResource::Obsidian => "solid/obsidian.png".to_string(),
        GalaxiaResource::Copper => "solid/copper.png".to_string(),
        GalaxiaResource::Tin => "solid/tin.png".to_string(),
        GalaxiaResource::Iron => "solid/iron.png".to_string(),
        GalaxiaResource::Silver => "solid/silver.png".to_string(),
        GalaxiaResource::Gold => "solid/gold.png".to_string(),
        GalaxiaResource::Diamond => "solid/diamond.png".to_string(),
        GalaxiaResource::Amethyst => "solid/amethyst.png".to_string(),
        GalaxiaResource::Moss => "solid/moss.png".to_string(),
        GalaxiaResource::Unobtainium => "solid/unobtainium.png".to_string(),
        // liquid
        GalaxiaResource::SaltWater => "liquid/salt_water.png".to_string(),
        GalaxiaResource::FreshWater => "liquid/fresh_water.png".to_string(),
        // gas
        // mana
        // energy
        // heat
    }
}

pub fn resource_to_name(resource: GalaxiaResource, full: bool) -> String {
    if full {
        match resource {
            // abstract
            GalaxiaResource::ShortLeftClick => "Short Left Click".to_string(),
            GalaxiaResource::LongLeftClick => "Long Left Click".to_string(),
            // solid
            GalaxiaResource::Apple => "Apple".to_string(),
            GalaxiaResource::Lemon => "Lemon".to_string(),
            GalaxiaResource::Lime => "Lime".to_string(),
            GalaxiaResource::Mud => "Mud".to_string(),
            GalaxiaResource::Dirt => "Dirt".to_string(),
            GalaxiaResource::Sandstone => "Sandstone".to_string(),
            GalaxiaResource::Granite => "Granite".to_string(),
            GalaxiaResource::Marble => "Marble".to_string(),
            GalaxiaResource::Obsidian => "Obsidian".to_string(),
            GalaxiaResource::Copper => "Copper".to_string(),
            GalaxiaResource::Tin => "Tin".to_string(),
            GalaxiaResource::Iron => "Iron".to_string(),
            GalaxiaResource::Silver => "Silver".to_string(),
            GalaxiaResource::Gold => "Gold".to_string(),
            GalaxiaResource::Diamond => "Diamond".to_string(),
            GalaxiaResource::Amethyst => "Amethyst".to_string(),
            GalaxiaResource::Moss => "Moss".to_string(),
            GalaxiaResource::Unobtainium => "Unobtainium".to_string(),
            // liquid
            GalaxiaResource::SaltWater => "Salt Water".to_string(),
            GalaxiaResource::FreshWater => "Fresh Water".to_string(),
            // gas
            // mana
            // energy
            // heat
        }
    } else {
        match resource {
            // abstract
            GalaxiaResource::ShortLeftClick => "Click".to_string(),
            GalaxiaResource::LongLeftClick => "Click".to_string(),
            // solid
            GalaxiaResource::Apple => "Fruit".to_string(),
            GalaxiaResource::Lemon => "Fruit".to_string(),
            GalaxiaResource::Lime => "Fruit".to_string(),
            GalaxiaResource::Mud => "Dirt".to_string(),
            GalaxiaResource::Dirt => "Dirt".to_string(),
            GalaxiaResource::Sandstone => "Stone".to_string(),
            GalaxiaResource::Granite => "Stone".to_string(),
            GalaxiaResource::Marble => "Stone".to_string(),
            GalaxiaResource::Obsidian => "Stone".to_string(),
            GalaxiaResource::Copper => "Metal".to_string(),
            GalaxiaResource::Tin => "Metal".to_string(),
            GalaxiaResource::Iron => "Metal".to_string(),
            GalaxiaResource::Silver => "Metal".to_string(),
            GalaxiaResource::Gold => "Metal".to_string(),
            GalaxiaResource::Diamond => "Gem".to_string(),
            GalaxiaResource::Amethyst => "Gem".to_string(),
            GalaxiaResource::Moss => "Plant".to_string(),
            GalaxiaResource::Unobtainium => "Metal".to_string(),
            // liquid
            GalaxiaResource::SaltWater => "Water".to_string(),
            GalaxiaResource::FreshWater => "Water".to_string(),
            // gas
            // mana
            // energy
            // heat
        }
    }
}
