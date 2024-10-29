use std::collections::HashSet;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::area::*;
use crate::collision::*;
use crate::player::*;

pub const MAX_RESOURCE_DISTANCE: f32 = 10000.0;

#[derive(Debug, Bundle)]
pub struct LooseResourceBundle {
    pub resource: LooseResource,
    pub area: CircularArea,
    pub sprite: SpriteBundle,
    pub rigid_body: RigidBody,
    pub ccd: Ccd,
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
    pub damping: Damping,
    pub velocity: Velocity,
    pub collider_mass_properties: ColliderMassProperties,
    pub active_events: ActiveEvents,
}

impl LooseResourceBundle {
    pub fn new(
        asset_server: &AssetServer,
        resource: GalaxiaResource,
        amount: f32,
        transform: Transform,
        velocity: Velocity,
    ) -> Self {
        // radius is cross-section of a cylinder with volume proportional to amount
        // plus a constant to make it visible
        let area = CircularArea {
            radius: 9.0
                + ((3.0 * amount) / (4.0 * std::f32::consts::PI)).cbrt(),
        };
        Self {
            resource: LooseResource { resource, amount },
            area,
            sprite: SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(area.into()),
                    ..default()
                },
                texture: asset_server.load(resource_to_asset(resource)),
                transform,
                ..default()
            },
            rigid_body: RigidBody::Dynamic,
            ccd: Ccd::enabled(),
            collider: area.into(),
            collision_groups: CollisionGroups::new(ETHER_GROUP, ether_filter()),
            damping: Damping {
                linear_damping: 1.0,
                angular_damping: 1.0,
            },
            velocity,
            collider_mass_properties: ColliderMassProperties::Mass(amount),
            active_events: ActiveEvents::COLLISION_EVENTS,
        }
    }

    pub fn new_from_minigame(
        asset_server: &AssetServer,
        resource: GalaxiaResource,
        amount: f32,
        minigame_global_transform: &GlobalTransform,
        minigame_area: &RectangularArea,
    ) -> Self {
        let transform = Transform::from_translation(
            minigame_global_transform.translation()
                + minigame_area.dimensions3() / 1.8,
        );
        Self::new(
            asset_server,
            resource,
            amount,
            transform,
            Velocity::linear(Vec2::new(70.0, -70.0)),
        )
    }
}

#[derive(Debug, Component)]
#[component(storage = "SparseSet")]
pub struct LooseResource {
    pub resource: GalaxiaResource,
    pub amount: f32,
}

impl LooseResource {
    pub fn new(resource: GalaxiaResource, amount: f32) -> Self {
        Self { resource, amount }
    }

    pub fn combine(&self, other: &Self) -> Option<Self> {
        if self.resource != other.resource {
            return None;
        }
        // TODO update when resources have form, so rigid solids do not combine
        Some(Self {
            resource: self.resource,
            amount: self.amount + other.amount,
        })
    }
}

#[derive(Debug, Copy, Clone, Component)]
pub struct Stuck {
    pub player: Entity,
}

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct Sticky;

pub enum ResourceKind {
    // clicks, shapes, colors
    // usually inert but in the right context can combine to create a new
    // resource or effect
    Abstract,
    // solid, liquid, and gas are physical
    // they behave like they do IRL
    Solid,
    Liquid,
    Gas,
    // Fire, Water, Earth, Air, and much more esoteric magical energies
    // behavior varies wildly by type
    Mana,
    // electricity, heat, potential and kinetic energy, sunlight, light, sound
    // expended for an effect as soon as possible
    Energy,
    // special resource acquired when the player beats a minigame
    // behaves like a solid resource
    Minigame,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum GalaxiaResource {
    // abstract
    ShortClick,
    LongClick,

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
    Bronze,
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
    // minigame
}

pub fn resource_to_kind(resource: GalaxiaResource) -> ResourceKind {
    match resource {
        // abstract
        GalaxiaResource::ShortClick => ResourceKind::Abstract,
        GalaxiaResource::LongClick => ResourceKind::Abstract,
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
        GalaxiaResource::Bronze => ResourceKind::Solid,
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
        GalaxiaResource::ShortClick => {
            "abstract/short_left_click.png".to_string()
        }
        GalaxiaResource::LongClick => {
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
        GalaxiaResource::Bronze => "solid/bronze.png".to_string(),
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
            GalaxiaResource::ShortClick => "Short Left Click".to_string(),
            GalaxiaResource::LongClick => "Long Left Click".to_string(),
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
            GalaxiaResource::Bronze => "Bronze".to_string(),
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
            GalaxiaResource::ShortClick => "Click".to_string(),
            GalaxiaResource::LongClick => "Click".to_string(),
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
            GalaxiaResource::Bronze => "Metal".to_string(),
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

pub fn teleport_distant_loose_resources(
    mut query: Query<&mut Transform, (With<LooseResource>, Without<Stuck>)>,
) {
    for mut transform in query.iter_mut() {
        if transform.translation.length() > MAX_RESOURCE_DISTANCE {
            transform.translation = Vec3::ZERO;
        }
    }
}

pub fn combine_loose_resources(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    loose_resource_query: Query<
        (&LooseResource, &Transform, &Velocity),
        Without<Stuck>,
    >,
    mut collision_events: EventReader<CollisionEvent>,
) {
    let mut eliminated: HashSet<Entity> = HashSet::new();
    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(entity1, entity2, _) => {
                // already handled
                if eliminated.contains(entity1) || eliminated.contains(entity2)
                {
                    continue;
                }
                // only loose resources handled
                let (resource1, transform1, velocity1) =
                    match loose_resource_query.get(*entity1) {
                        Ok(r) => r,
                        Err(_) => continue,
                    };
                let (resource2, _, velocity2) =
                    match loose_resource_query.get(*entity2) {
                        Ok(r) => r,
                        Err(_) => continue,
                    };

                // combine if possible
                let combined = match resource1.combine(&resource2) {
                    Some(c) => c,
                    None => continue,
                };

                // despawn both and add a new one
                commands.entity(*entity1).despawn();
                commands.entity(*entity2).despawn();
                eliminated.insert(*entity1);
                eliminated.insert(*entity2);
                commands.spawn(LooseResourceBundle::new(
                    &asset_server,
                    combined.resource,
                    combined.amount,
                    *transform1,
                    Velocity {
                        linvel: velocity1.linvel + velocity2.linvel,
                        angvel: velocity1.angvel + velocity2.angvel,
                    },
                ));
            }
            _ => {}
        }
    }
}

pub fn grab_resources(
    mut commands: Commands,
    rapier_context: Res<RapierContext>,
    player_query: Query<(Entity, &CircularArea), (With<Player>, With<Sticky>)>,
    mut loose_resource_query: Query<
        (&CircularArea, &mut Velocity),
        (With<LooseResource>, Without<Stuck>),
    >,
    mut collision_events: EventReader<CollisionEvent>,
) {
    let Ok(player) = player_query.get_single() else {
        return;
    };
    let (player_entity, player_area) = player;

    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(entity1, entity2, _) => {
                let other: Entity;
                let player_is_first: bool;
                if *entity1 == player_entity {
                    other = *entity2;
                    player_is_first = true;
                } else if *entity2 == player_entity {
                    other = *entity1;
                    player_is_first = false;
                } else {
                    continue;
                }

                let Ok(resource) = loose_resource_query.get_mut(other) else {
                    continue;
                };
                let (resource_area, mut resource_velocity) = resource;

                let Some(contact_pair) =
                    rapier_context.contact_pair(player_entity, other)
                else {
                    continue;
                };
                let Some(manifold) = contact_pair.manifold(0) else {
                    continue;
                };
                let direction = (if player_is_first {
                    manifold.local_n1()
                } else {
                    manifold.local_n2()
                })
                .normalize();
                let distance = player_area.radius + resource_area.radius;

                let joint = FixedJointBuilder::new()
                    .local_anchor1(direction * distance);
                commands
                    .entity(other)
                    .insert(ImpulseJoint::new(player_entity, joint))
                    .insert(Stuck {
                        player: player_entity,
                    });
                resource_velocity.linvel = Vec2::ZERO;
                resource_velocity.angvel = 0.0;
            }
            _ => {}
        }
    }
}
