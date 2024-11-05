use std::collections::HashSet;

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::*;
use bevy_rapier2d::prelude::*;
use int_enum::IntEnum;
use wyrand::WyRand;

use crate::entities::*;
use crate::libs::*;

pub const MAX_ITEM_DISTANCE: f32 = 10000.0;

#[derive(Debug, Bundle)]
pub struct ItemBundle {
    pub item: Item,
    pub area: CircularArea,
    pub sprite: SpriteBundle,
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
    pub damping: Damping,
    pub velocity: Velocity,
    pub collider_mass_properties: ColliderMassProperties,
    pub active_events: ActiveEvents,
}

impl ItemBundle {
    pub fn new(
        asset_server: &AssetServer,
        item: Item,
        transform: Transform,
        velocity: Velocity,
    ) -> Self {
        let amount = item.amount;
        let area = Self::calculate_area(amount);
        // must be at least 1.0 to avoid tunneling
        let density =
            1.0 + (amount / (std::f32::consts::PI * area.radius * area.radius));
        Self {
            item,
            area,
            sprite: SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(area.into()),
                    ..default()
                },
                texture: asset_server.load(item.asset()),
                transform,
                ..default()
            },
            rigid_body: RigidBody::Dynamic,
            collider: area.into(),
            collision_groups: CollisionGroups::new(ETHER_GROUP, ether_filter()),
            damping: Damping {
                linear_damping: 1.0,
                angular_damping: 1.0,
            },
            velocity,
            collider_mass_properties: ColliderMassProperties::Density(density),
            active_events: ActiveEvents::COLLISION_EVENTS,
        }
    }

    pub fn new_from_minigame(
        asset_server: &AssetServer,
        item: Item,
        minigame_global_transform: &GlobalTransform,
        minigame_area: &RectangularArea,
    ) -> Self {
        let transform = Transform::from_translation(
            minigame_global_transform.translation()
                + minigame_area.dimensions3() / 1.8,
        );
        Self::new(
            asset_server,
            item,
            transform,
            Velocity::linear(Vec2::new(70.0, -70.0)),
        )
    }

    pub fn calculate_area(amount: f32) -> CircularArea {
        // Radius is cross-section of a cylinder with volume proportional to amount
        // plus a constant to make it visible.
        // Also <1.0 is much smaller than 1.0 which is much smaller than >1.0.
        let radius = if amount < 1.0 {
            4.0
        } else if amount == 1.0 {
            8.0
        } else {
            9.0 + ((3.0 * amount) / (4.0 * std::f32::consts::PI)).cbrt()
        };
        CircularArea { radius }
    }
}

#[derive(Debug, Clone, Copy, Component)]
#[component(storage = "SparseSet")]
#[repr(C, align(8))]
pub struct Item {
    pub item_type: ItemType,
    pub item_data: ItemData,
    pub amount: f32,
}

impl Item {
    pub fn new(item_type: ItemType, item_data: ItemData, amount: f32) -> Self {
        Self {
            item_type,
            item_data,
            amount,
        }
    }

    pub fn new_one(item_type: ItemType, item_data: ItemData) -> Self {
        Self::new(item_type, item_data, 1.0)
    }

    pub fn new_abstract(
        kind: AbstractItemKind,
        variant: u8,
        amount: f32,
    ) -> Self {
        Self::new(
            ItemType::Abstract,
            ItemData {
                r#abstract: AbstractItem { kind, variant },
            },
            amount,
        )
    }

    pub fn new_physical(
        form: PhysicalItemForm,
        material: PhysicalItemMaterial,
        amount: f32,
    ) -> Self {
        Self::new(
            ItemType::Physical,
            ItemData {
                physical: PhysicalItem { form, material },
            },
            amount,
        )
    }

    pub fn combine(&self, other: &Self) -> Option<Self> {
        if self.item_type != other.item_type {
            return None;
        }
        // TODO handle item combinations correctly
        Some(Self {
            item_type: self.item_type,
            item_data: self.item_data,
            amount: self.amount + other.amount,
        })
    }

    pub fn name(&self) -> String {
        self.identifier().adjective
    }

    pub fn asset(&self) -> String {
        self.identifier().asset()
    }

    pub fn as_abstract(&self) -> Option<AbstractItem> {
        unsafe {
            if self.item_type == ItemType::Abstract {
                Some(self.item_data.r#abstract)
            } else {
                None
            }
        }
    }

    pub fn as_physical(&self) -> Option<PhysicalItem> {
        unsafe {
            if self.item_type == ItemType::Physical {
                Some(self.item_data.physical)
            } else {
                None
            }
        }
    }

    pub fn as_mana(&self) -> Option<ManaItem> {
        unsafe {
            if self.item_type == ItemType::Mana {
                Some(self.item_data.mana)
            } else {
                None
            }
        }
    }

    pub fn as_energy(&self) -> Option<EnergyItem> {
        unsafe {
            if self.item_type == ItemType::Energy {
                Some(self.item_data.energy)
            } else {
                None
            }
        }
    }

    pub fn as_minigame(&self) -> Option<MinigameItem> {
        unsafe {
            if self.item_type == ItemType::Minigame {
                Some(self.item_data.minigame)
            } else {
                None
            }
        }
    }

    fn identifier(&self) -> ItemIdentifier {
        unsafe {
            match self.item_type {
                ItemType::Abstract => self.item_data.r#abstract.identifier(),
                ItemType::Physical => self.item_data.physical.identifier(),
                ItemType::Mana => self.item_data.mana.identifier(),
                ItemType::Energy => self.item_data.energy.identifier(),
                ItemType::Minigame => self.item_data.minigame.identifier(),
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ItemType {
    // clicks, shapes, colors
    // usually inert but in the right context can combine to create a new
    // resource or effect
    Abstract,
    // behave kinda like they do in real life
    Physical,
    // Fire, Water, Earth, Air, and much more esoteric magical energies
    // behavior varies wildly by type
    Mana,
    // electricity, heat, potential and kinetic energy, sunlight, light, sound
    // expended for an effect as soon as possible
    Energy,
    // special resource acquired when the player beats a minigame
    // behaves like a physical solid resource
    Minigame,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub union ItemData {
    pub r#abstract: AbstractItem,
    pub physical: PhysicalItem,
    pub mana: ManaItem,
    pub energy: EnergyItem,
    pub minigame: MinigameItem,
}

impl std::fmt::Debug for ItemData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ItemData{{...}}")
    }
}

pub struct ItemIdentifier {
    pub domain: String,    // ex: "physical" or "abstract"
    pub noun: String,      // ex: "powder" or "click"
    pub adjective: String, // ex: "marble" or "short"
}

impl ItemIdentifier {
    pub fn name(&self) -> String {
        if self.adjective.is_empty() {
            self.noun.clone()
        } else {
            format!("{} {}", self.adjective, self.noun)
        }
    }

    // Returns the asset path for the texture/material of the item.
    pub fn asset(&self) -> String {
        let filename = match self.domain.as_str() {
            "abstract" => self.noun.clone(),
            _ => self.adjective.clone(),
        };
        format!("{}/{}.png", self.domain, filename)
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct AbstractItem {
    pub kind: AbstractItemKind,
    pub variant: u8,
}

impl AbstractItem {
    pub fn object(&self) -> String {
        "assets/objects/abstract/".to_string()
            + match self.kind {
                AbstractItemKind::Click => {
                    match self.variant {
                        0 => "click_short".to_string(),
                        1 => "click_long".to_string(),
                        _ => panic!("Invalid abstract item variant {} for click", self.variant),
                    }
                }
}
                _ => panic!("Material {:?} not implemented", self),
            }
    }

    pub fn identifier(&self) -> ItemIdentifier {
        let noun: &str;
        let adjective: &str;
        match self.kind {
            AbstractItemKind::Click => {
                noun = "Click";
                match self.variant {
                    0 => adjective = "Short",
                    1 => adjective = "Long",
                    _ => panic!(
                        "Invalid abstract item variant {} for click",
                        self.variant
                    ),
                }
            }
            AbstractItemKind::XP => {
                noun = "XP";
                adjective = "";
            }
            AbstractItemKind::Rune => {
                noun = "rune";
                match Rune::try_from(self.variant) {
                    Ok(Rune::InclusiveSelf) => adjective = "Inclusive Self",
                    Ok(Rune::Connector) => adjective = "Connector",
                    Ok(Rune::ExclusiveSelf) => adjective = "Exclusive Self",
                    Ok(Rune::Shelter) => adjective = "Shelter",
                    Ok(Rune::InclusiveOther) => adjective = "Inclusive Other",
                    Ok(Rune::ExclusiveOther) => adjective = "Exclusive Other",
                    Err(_) => panic!(
                        "Invalid abstract item variant {} for rune",
                        self.variant
                    ),
                }
            }
        }
        ItemIdentifier {
            domain: "abstract".to_string(),
            noun: noun.to_string(),
            adjective: adjective.to_string(),
        }
    }
}

// A Rune is a magical symbol that can be drawn in a Draw minigame.
// Each rune is a 2D grid of pixels, where each pixel can be on or off.
// For a Rune, only connected pixels are considered.
// Orientation also matters - a rune cannot be rotated or flipped.
#[repr(u8)]
#[derive(Debug, PartialEq, IntEnum)]
pub enum Rune {
    // 1x1 pixels
    // magically, refers to the inclusive self
    InclusiveSelf = 0,
    // 2x1
    // magically, acts as connector
    Connector = 1,
    // 2x2
    // magically, refers to the EXCLUSIVE self
    ExclusiveSelf = 2,
    // 3x2, missing middle bottom
    // magically, refers to shelter or protection
    Shelter = 3,
    // 3x3, missing middle
    // magically, refers to the inclusive other (not-self)
    InclusiveOther = 4,
    // 4x3 TODO
    // 4x4, missing middle
    // magically, refers to the EXCLUSIVE other (not-self)
    ExclusiveOther = 5,
    // TODO: add more runes - at least 100 in total
    //       each expansion of space should require a new rune
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum AbstractItemKind {
    Click,
    XP,
    Rune,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct PhysicalItem {
    pub form: PhysicalItemForm,
    pub material: PhysicalItemMaterial,
}

impl PhysicalItem {
    pub fn draw(&self, rand: &mut WyRand, size: u32) -> Image {
        match self.form {
            PhysicalItemForm::Object => load_image(self.material.object()),
            PhysicalItemForm::Block => {
                self.material.palette().draw_block(rand, size)
            }
            PhysicalItemForm::Ball => {
                self.material.palette().draw_ball(rand, size)
            }

            _ => panic!("Not implemented"),
        }
    }

    pub fn identifier(&self) -> ItemIdentifier {
        let noun: &str;
        let adjective: &str;
        match self.form {
            PhysicalItemForm::Gas => noun = "Gas",
            PhysicalItemForm::Liquid => noun = "Liquid",
            PhysicalItemForm::Goo => noun = "Goo",
            PhysicalItemForm::Powder => noun = "Powder",
            PhysicalItemForm::Object => noun = "Object",
            PhysicalItemForm::Lump => noun = "Lump",
            PhysicalItemForm::Block => noun = "Block",
            PhysicalItemForm::Ball => noun = "Ball",
        }
        match self.material {
            PhysicalItemMaterial::Apple => adjective = "Apple",
            PhysicalItemMaterial::Lemon => adjective = "Lemon",
            PhysicalItemMaterial::Lime => adjective = "Lime",
            PhysicalItemMaterial::Mud => adjective = "Mud",
            PhysicalItemMaterial::Dirt => adjective = "Dirt",
            PhysicalItemMaterial::Sandstone => adjective = "Sandstone",
            PhysicalItemMaterial::Granite => adjective = "Granite",
            PhysicalItemMaterial::Marble => adjective = "Marble",
            PhysicalItemMaterial::Obsidian => adjective = "Obsidian",
            PhysicalItemMaterial::Copper => adjective = "Copper",
            PhysicalItemMaterial::Tin => adjective = "Tin",
            PhysicalItemMaterial::Bronze => adjective = "Bronze",
            PhysicalItemMaterial::Iron => adjective = "Iron",
            PhysicalItemMaterial::Silver => adjective = "Silver",
            PhysicalItemMaterial::Gold => adjective = "Gold",
            PhysicalItemMaterial::Diamond => adjective = "Diamond",
            PhysicalItemMaterial::Amethyst => adjective = "Amethyst",
            PhysicalItemMaterial::Moss => adjective = "Moss",
            PhysicalItemMaterial::Unobtainium => adjective = "Unobtainium",
            PhysicalItemMaterial::SaltWater => adjective = "Salt Water",
            PhysicalItemMaterial::FreshWater => adjective = "Fresh Water",
        }
        ItemIdentifier {
            domain: "physical".to_string(),
            noun: noun.to_string(),
            adjective: adjective.to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum PhysicalItemForm {
    Gas,
    Liquid,
    Goo,
    Powder,
    // solids
    Object, // generic solid
    Lump,
    Block,
    Ball,
}

#[derive(Debug, Clone, Copy)]
#[repr(u64)]
pub enum PhysicalItemMaterial {
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
    SaltWater,
    FreshWater,
}

impl PhysicalItemMaterial {
    pub fn object(&self) -> String {
        "assets/objects/physical/".to_string()
            + match self {
                PhysicalItemMaterial::Apple => "apple",
                _ => panic!("Material {:?} not implemented", self),
            }
    }

    pub fn palette(&self) -> image_gen::ColorPalette {
        match self {
            PhysicalItemMaterial::Mud => Self::mud_palette(),
            _ => panic!("Material {:?} not implemented", self),
        }
    }

    fn mud_palette() -> image_gen::ColorPalette {
        let mut palette = image_gen::ColorPalette::new();
        palette.add_color(image_gen::Colorant::new_loose(240, 100, 10, 10, 1));
        palette
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ManaItem {
    pub kind: ManaResourceKind,
    pub subkind: u8,
    pub intent: ManaIntent,
}

impl ManaItem {
    pub fn identifier(&self) -> ItemIdentifier {
        panic!("ManaItem::identifier not implemented");
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ManaResourceKind {
    Fire,
    Water,
    Earth,
    Air,
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ManaIntent {
    Attack,
    Defense,
    Support,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct EnergyItem {
    pub kind: EnergyResourceKind,
}

impl EnergyItem {
    pub fn identifier(&self) -> ItemIdentifier {
        panic!("EnergyItem::identifier not implemented");
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum EnergyResourceKind {
    Kinetic,
    Potential,
    Thermal,
    Electric,
    Magnetic,
    Radiant,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct MinigameItem {
    pub kind: MinigameResourceKind,
    pub variant: u32,
}

impl MinigameItem {
    pub fn identifier(&self) -> ItemIdentifier {
        panic!("MinigameItem::identifier not implemented");
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum MinigameResourceKind {
    Button,
    PrimordialOcean,
    Draw,
    BlockBreaker,
    Tree,
}

#[derive(Debug, Copy, Clone, Component)]
pub struct Stuck {
    pub player: Entity,
}

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct Sticky;

pub fn teleport_distant_loose_resources(
    mut query: Query<&mut Transform, (With<Item>, Without<Stuck>)>,
) {
    for mut transform in query.iter_mut() {
        if transform.translation.length() > MAX_ITEM_DISTANCE {
            transform.translation = Vec3::ZERO;
        }
    }
}

pub fn combine_loose_resources(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    loose_resource_query: Query<(&Item, &Transform, &Velocity), Without<Stuck>>,
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
                commands.spawn(ItemBundle::new(
                    &asset_server,
                    combined,
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
        (With<Item>, Without<Stuck>),
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

pub fn release_resources(
    mut commands: Commands,
    loose_resource_query: Query<(Entity, &Stuck), With<Item>>,
    player_query: Query<Entity, (With<Player>, Without<Sticky>)>,
) {
    for (stuck_entity, stuck) in loose_resource_query.iter() {
        let player_entity = stuck.player;
        if !player_query.contains(player_entity) {
            continue;
        }
        commands.entity(stuck_entity).remove::<ImpulseJoint>();
        commands.entity(stuck_entity).remove::<Stuck>();
    }
}

pub fn stencil_circle(
    images: &mut ResMut<Assets<Image>>,
    image_handle: &Handle<Image>,
) -> Handle<Image> {
    let original = images.get(image_handle).unwrap();
    let mut new_image = original.clone();

    let width = new_image.width() as f32;
    let height = new_image.height() as f32;
    let center_x = width / 2.0;
    let center_y = height / 2.0;
    let radius = width.min(height) / 2.0;
    let square_radius = radius * radius;

    // Iterate through pixels
    for y in 0..new_image.height() {
        for x in 0..new_image.width() {
            let dx = x as f32 - center_x;
            let dy = y as f32 - center_y;
            let square_distance = dx * dx + dy * dy;

            if square_distance > square_radius {
                // Outside circle - make transparent
                let pixel_index = (y * new_image.width() + x) as usize * 4;
                new_image.data[pixel_index + 3] = 0; // Set alpha to 0
            }
        }
    }

    images.add(new_image)
}
