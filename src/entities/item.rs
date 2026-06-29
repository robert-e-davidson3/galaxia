use std::collections::HashSet;
use std::mem::discriminant;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use int_enum::IntEnum;
use wyrand::WyRand;

use crate::entities::*;
use crate::libs::*;

pub const MAX_ITEM_DISTANCE: f32 = 10000.0;
pub const SEED: u64 = 91;

#[derive(Debug, Bundle)]
pub struct ItemBundle {
    pub item: Item,
    pub area: CircularArea,
    pub sprite: Sprite,
    pub transform: Transform,
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
    pub damping: Damping,
    pub velocity: Velocity,
    pub collider_mass_properties: ColliderMassProperties,
    pub active_events: ActiveEvents,
}

// TODO fn for altering item components when amount changes
//      IIRC can simply insert components on the entity and they overwrite them
impl ItemBundle {
    pub fn new(
        images: &mut Assets<Image>,
        generated_image_assets: &mut image_gen::GeneratedImageAssets,
        item: Item,
        transform: Transform,
        velocity: Velocity,
    ) -> Self {
        let area = CircularArea {
            radius: item.size(),
        };
        let density = item.density();
        let texture: Handle<Image> = generated_image_assets
            .get(&item.uid())
            .unwrap_or_else(|| {
                let image = item.draw(&mut WyRand::new(SEED));
                let texture = images.add(image.clone());
                generated_image_assets.insert(item.uid(), &texture);
                texture
            });
        Self {
            item,
            area,
            sprite: Sprite {
                image: texture,
                custom_size: Some(area.into()),
                ..default()
            },
            transform,
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
        images: &mut Assets<Image>,
        generated_image_assets: &mut image_gen::GeneratedImageAssets,
        item: Item,
        minigame_global_transform: &GlobalTransform,
        minigame_area: &RectangularArea,
    ) -> Self {
        let transform = Transform::from_translation(
            minigame_global_transform.translation()
                + minigame_area.dimensions3() / 1.5,
        );
        Self::new(
            images,
            generated_image_assets,
            item,
            transform,
            Velocity::linear(Vec2::new(70.0, -70.0)),
        )
    }

    pub fn eject_from_minigame(
        commands: &mut Commands,
        item_entity: Entity,
        minigame_global_transform: &GlobalTransform,
        minigame_area: &RectangularArea,
    ) {
        let transform = Transform::from_translation(
            minigame_global_transform.translation()
                + minigame_area.dimensions3() / 1.5,
        );
        let velocity = Velocity::linear(Vec2::new(70.0, -70.0));
        commands.entity(item_entity).insert((transform, velocity));
    }

    /// Clear items from a minigame area by ejecting them outside
    pub fn clear_minigame_area(
        commands: &mut Commands,
        minigame_global_transform: &GlobalTransform,
        minigame_area: &RectangularArea,
        item_query: &Query<
            (&Transform, Entity),
            (With<Item>, Without<LevelingUp>),
        >,
    ) {
        let minigame_pos = minigame_global_transform.translation().truncate();

        for (item_transform, item_entity) in item_query.iter() {
            let item_pos = item_transform.translation.truncate();
            if minigame_area.is_within(item_pos, minigame_pos) {
                Self::eject_from_minigame(
                    commands,
                    item_entity,
                    minigame_global_transform,
                    minigame_area,
                );
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Component)]
#[component(storage = "SparseSet")]
pub struct Item {
    pub r#type: ItemType,
    pub amount: f32,
}

impl Item {
    pub fn new(r#type: ItemType, amount: f32) -> Self {
        Self { r#type, amount }
    }

    pub fn uid(&self) -> String {
        self.identifier().uid()
    }

    pub fn new_abstract(kind: AbstractKind, variant: u8, amount: f32) -> Self {
        Self::new(ItemType::Abstract(AbstractItem { kind, variant }), amount)
    }

    fn bulk(
        structure: BulkStructure,
        substance: Substance,
        processing: Processing,
        shape: BulkShape,
        amount: f32,
    ) -> Self {
        // For non-solid structures, shape and processing are irrelevant to
        // identity; normalize them so e.g. two liquids of the same substance
        // are equal (and stack).
        let (processing, shape) = if structure == BulkStructure::Solid {
            (processing, shape)
        } else {
            (Processing::Refined, BulkShape::Lump)
        };
        Self::new(
            ItemType::Physical(PhysicalItem::Bulk(BulkItem {
                structure,
                substance,
                processing,
                shape,
                quality: 0,
            })),
            amount,
        )
    }

    pub fn solid(substance: Substance, shape: BulkShape, amount: f32) -> Self {
        Self::bulk(
            BulkStructure::Solid,
            substance,
            Processing::Refined,
            shape,
            amount,
        )
    }

    pub fn ore(substance: Substance, amount: f32) -> Self {
        Self::bulk(
            BulkStructure::Solid,
            substance,
            Processing::Raw,
            BulkShape::Gravel,
            amount,
        )
    }

    pub fn liquid(substance: Substance, amount: f32) -> Self {
        Self::bulk(
            BulkStructure::Liquid,
            substance,
            Processing::Refined,
            BulkShape::Lump,
            amount,
        )
    }

    pub fn powder(substance: Substance, amount: f32) -> Self {
        Self::bulk(
            BulkStructure::Powder,
            substance,
            Processing::Refined,
            BulkShape::Lump,
            amount,
        )
    }

    pub fn fruit(species: Species, amount: f32) -> Self {
        Self::new(
            ItemType::Physical(PhysicalItem::Discrete(DiscreteItem {
                species,
                state: State::Freshness(127),
            })),
            amount,
        )
    }

    pub fn organism(
        species: Species,
        stage: LifeStage,
        amount: f32,
    ) -> Self {
        Self::new(
            ItemType::Physical(PhysicalItem::Discrete(DiscreteItem {
                species,
                state: State::Stage(stage),
            })),
            amount,
        )
    }

    pub fn combine(&self, other: &Self) -> Option<Self> {
        if discriminant(&self.r#type) != discriminant(&other.r#type) {
            return None;
        }

        match (self.r#type, other.r#type) {
            (ItemType::Abstract(a), ItemType::Abstract(b)) => a
                .combine(&b, self.amount, other.amount)
                .map(|(t, a)| (ItemType::Abstract(t), a)),
            (ItemType::Physical(a), ItemType::Physical(b)) => a
                .combine(&b, self.amount, other.amount)
                .map(|(t, a)| (ItemType::Physical(t), a)),
            (ItemType::Mana(a), ItemType::Mana(b)) => a
                .combine(&b, self.amount, other.amount)
                .map(|(t, a)| (ItemType::Mana(t), a)),
            (ItemType::Energy(a), ItemType::Energy(b)) => a
                .combine(&b, self.amount, other.amount)
                .map(|(t, a)| (ItemType::Energy(t), a)),
            (ItemType::Minigame(a), ItemType::Minigame(b)) => a
                .combine(&b, self.amount, other.amount)
                .map(|(t, a)| (ItemType::Minigame(t), a)),
            _ => None, // mismatched types
        }
        .map(|(r#type, amount)| Self { r#type, amount })
    }

    pub fn name(&self) -> String {
        self.identifier().adjective
    }

    pub fn asset(&self) -> String {
        self.identifier().asset()
    }

    pub const MIN_RADIUS: f32 = 4.0;
    pub const MAX_RADIUS: f32 = 18.0;

    // Radius is cross-section of a cylinder with volume proportional to amount
    // plus a constant to make it visible.
    // Also <1.0 is much smaller than 1.0 which is much smaller than >1.0.
    // Max size is double
    pub fn size(&self) -> f32 {
        if self.amount < 1.0 {
            Self::MIN_RADIUS
        } else if self.amount == 1.0 {
            8.0
        } else {
            Self::MAX_RADIUS.min(
                9.0 + ((3.0 * self.amount) / (4.0 * std::f32::consts::PI))
                    .cbrt(),
            )
        }
    }

    pub fn density(&self) -> f32 {
        let size = self.size();
        let density = self.amount / (std::f32::consts::PI * size * size);
        if density < 1.0 {
            1.0 // minimum to avoid tunneling
        } else {
            density
        }
    }

    pub fn draw(&self, rand: &mut WyRand) -> Image {
        self.r#type.draw(rand)
    }

    fn identifier(&self) -> ItemIdentifier {
        self.r#type.identifier()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemType {
    Abstract(AbstractItem),
    Physical(PhysicalItem),
    Mana(ManaItem),
    Energy(EnergyItem),
    Minigame(MinigameItem),
}

impl ItemType {
    pub fn to_item(self, amount: f32) -> Item {
        Item::new(self, amount)
    }

    pub fn uid(&self) -> String {
        self.identifier().uid()
    }

    pub fn name(&self) -> String {
        self.identifier().adjective
    }

    pub fn identifier(&self) -> ItemIdentifier {
        match self {
            ItemType::Abstract(a) => a.identifier(),
            ItemType::Physical(a) => a.identifier(),
            ItemType::Mana(a) => a.identifier(),
            ItemType::Energy(a) => a.identifier(),
            ItemType::Minigame(a) => a.identifier(),
        }
    }

    pub fn draw(&self, rand: &mut WyRand) -> Image {
        match self {
            ItemType::Abstract(a) => a.draw(rand),
            ItemType::Physical(a) => a.draw(rand),
            ItemType::Mana(a) => a.draw(rand),
            ItemType::Energy(a) => a.draw(rand),
            ItemType::Minigame(a) => a.draw(rand),
        }
    }

    //
    // Taxonomic helpers
    //

    // A discrete object (any species) counts as solid, as does a Bulk solid.
    pub fn is_solid(&self) -> bool {
        match self {
            ItemType::Physical(PhysicalItem::Bulk(b)) => {
                b.structure == BulkStructure::Solid
            }
            ItemType::Physical(PhysicalItem::Discrete(_)) => true,
            _ => false,
        }
    }

    pub fn is_fruit(&self) -> bool {
        match self {
            ItemType::Physical(PhysicalItem::Discrete(d)) => {
                d.species.class() == DiscreteClass::Fruit
            }
            _ => false,
        }
    }

    pub fn freshness(&self) -> Option<u8> {
        match self {
            ItemType::Physical(PhysicalItem::Discrete(DiscreteItem {
                state: State::Freshness(f),
                ..
            })) => Some(*f),
            _ => None,
        }
    }

    //
    // Packed identity (see references/item-model.md)
    //

    pub fn pack(&self) -> u64 {
        match self {
            ItemType::Physical(p) => p.pack(),
            ItemType::Mana(m) => m.pack(),
            ItemType::Energy(e) => e.pack(),
            ItemType::Abstract(a) => a.pack(),
            ItemType::Minigame(m) => m.pack(),
        }
    }

    pub fn unpack(packed: u64) -> Option<ItemType> {
        let domain = packed >> 61;
        match domain {
            DOMAIN_PHYSICAL => PhysicalItem::unpack(packed).map(ItemType::Physical),
            DOMAIN_MANA => ManaItem::unpack(packed).map(ItemType::Mana),
            DOMAIN_ENERGY => EnergyItem::unpack(packed).map(ItemType::Energy),
            DOMAIN_ABSTRACT => AbstractItem::unpack(packed).map(ItemType::Abstract),
            DOMAIN_MINIGAME => {
                MinigameItem::unpack(packed).map(ItemType::Minigame)
            }
            _ => None,
        }
    }
}

//
// Packed-id domain tags and bit helpers (see references/item-model.md).
//

const DOMAIN_PHYSICAL: u64 = 0b000;
const DOMAIN_MANA: u64 = 0b001;
const DOMAIN_ENERGY: u64 = 0b010;
const DOMAIN_ABSTRACT: u64 = 0b011;
const DOMAIN_MINIGAME: u64 = 0b100;

// Physical kind tag [60:59]: Bulk vs Discrete. For Bulk, the matter-state lives
// in the nested BulkStructure field [58:56].
const PHYS_KIND_BULK: u64 = 0;
const PHYS_KIND_DISCRETE: u64 = 1;

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

    pub fn uid(&self) -> String {
        format!("{}/{}/{}", self.domain, self.noun, self.adjective)
    }

    // Returns the asset path for the texture/material of the item.
    pub fn asset(&self) -> String {
        let filename = match self.domain.as_str() {
            "abstract" => self.noun.clone(),
            // Fruit is textured by its form (Apple/Lemon/Lime), not material.
            _ if self.adjective == "Fruit" => self.noun.clone(),
            _ => self.adjective.clone(),
        };
        format!("{}/{}.png", self.domain, filename)
    }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
#[repr(C)]
pub struct AbstractItem {
    pub kind: AbstractKind,
    pub variant: u8,
}

impl AbstractItem {
    pub fn combine(
        &self,
        other: &AbstractItem,
        self_amount: f32,
        other_amount: f32,
    ) -> Option<(AbstractItem, f32)> {
        if self.kind == other.kind && self.variant == other.variant {
            Some((*self, self_amount + other_amount))
        } else {
            None
        }
    }

    fn pack(&self) -> u64 {
        let mut v = DOMAIN_ABSTRACT << 61;
        let kind = match self.kind {
            AbstractKind::Click => 0u64,
            AbstractKind::XP => 1,
            AbstractKind::Rune => 2,
        };
        v |= kind << 48;
        match self.kind {
            AbstractKind::Click => v |= ((self.variant & 0b11) as u64) << 46,
            AbstractKind::XP => v |= ((self.variant & 0xF) as u64) << 44,
            AbstractKind::Rune => v |= ((self.variant & 0x7F) as u64) << 41,
        }
        v
    }

    fn unpack(packed: u64) -> Option<AbstractItem> {
        let kind_bits = (packed >> 48) & 0x1FFF;
        let (kind, variant) = match kind_bits {
            0 => (AbstractKind::Click, ((packed >> 46) & 0b11) as u8),
            1 => (AbstractKind::XP, ((packed >> 44) & 0xF) as u8),
            2 => (AbstractKind::Rune, ((packed >> 41) & 0x7F) as u8),
            _ => return None,
        };
        Some(AbstractItem { kind, variant })
    }

    pub fn draw(&self, _rand: &mut WyRand) -> Image {
        match self.kind {
            AbstractKind::Click => {
                let path = format!("assets/abstract/{}.png", self.object());
                load_image(&path)
            }
            AbstractKind::Rune => match rune::Rune::try_from(self.variant) {
                Ok(rune) => image_gen::draw_rune(rune),
                Err(_) => panic!("Invalid rune variant {}", self.variant),
            },
            _ => panic!("Invalid abstract item kind {:?}", self.kind),
        }
    }

    pub fn object(&self) -> &str {
        match self.kind {
            AbstractKind::Click => match self.variant {
                0 => "ShortClick",
                1 => "LongClick",
                _ => panic!(
                    "Invalid abstract item variant {} for click",
                    self.variant
                ),
            },
            AbstractKind::Rune => match rune::Rune::try_from(self.variant) {
                Ok(rune::Rune::InclusiveSelf) => "RuneInclusiveSelf",
                Ok(rune::Rune::Connector) => "RuneConnector",
                Ok(rune::Rune::ExclusiveSelf) => "Exclusive Self",
                Ok(rune::Rune::Shelter) => "Shelter",
                Ok(rune::Rune::InclusiveOther) => "Inclusive Other",
                Ok(rune::Rune::Force) => "Force",
                Ok(rune::Rune::ExclusiveOther) => "Exclusive Other",
                Err(_) => panic!(
                    "Invalid abstract item variant {} for rune",
                    self.variant
                ),
            },
            _ => panic!("Material {:?} not implemented", self),
        }
    }

    pub fn identifier(&self) -> ItemIdentifier {
        let noun: &str;
        let adjective: &str;
        match self.kind {
            AbstractKind::Click => {
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
            AbstractKind::XP => {
                noun = "XP";
                adjective = "";
            }
            AbstractKind::Rune => {
                noun = "rune";
                match rune::Rune::try_from(self.variant) {
                    Ok(rune::Rune::InclusiveSelf) => {
                        adjective = "Inclusive Self"
                    }
                    Ok(rune::Rune::Connector) => adjective = "Connector",
                    Ok(rune::Rune::ExclusiveSelf) => {
                        adjective = "Exclusive Self"
                    }
                    Ok(rune::Rune::Shelter) => adjective = "Shelter",
                    Ok(rune::Rune::InclusiveOther) => {
                        adjective = "Inclusive Other"
                    }
                    Ok(rune::Rune::Force) => adjective = "Force",
                    Ok(rune::Rune::ExclusiveOther) => {
                        adjective = "Exclusive Other"
                    }
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

pub mod rune {
    use int_enum::IntEnum;

    // A Rune is a magical symbol that can be drawn in a Draw minigame.
    // Each rune is a 2D grid of pixels, where each pixel can be on or off.
    // For a Rune, only connected pixels are considered.
    // Orientation also matters - a rune cannot be rotated or flipped.
    #[repr(u8)]
    #[derive(Debug, PartialEq, Copy, Clone, IntEnum)]
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
        // 4x3
        // magically, refers to affecting physical matter
        Force = 5,
        // 4x4, missing middle
        // magically, refers to the EXCLUSIVE other (not-self)
        ExclusiveOther = 6,
        // TODO: add runes until there are at least 100
    }

    pub mod pattern {
        pub const INCLUSIVE_SELF: [[bool; 1]; 1] = [[true]];
        pub const CONNECTOR: [[bool; 2]; 1] = [[true, true]];
        pub const EXCLUSIVE_SELF: [[bool; 2]; 2] = [[true, true], [true, true]];
        pub const SHELTER: [[bool; 3]; 2] = [
            //
            [true, true, true],
            [true, false, true],
        ];
        pub const INCLUSIVE_OTHER: [[bool; 3]; 3] = [
            //
            [true, true, true],
            [true, false, true],
            [true, true, true],
        ];
        pub const FORCE: [[bool; 4]; 3] = [
            [true, true, false, false],
            [true, false, true, true],
            [true, true, true, false],
        ];
        pub const EXCLUSIVE_OTHER: [[bool; 4]; 4] = [
            [true, true, true, true],
            [true, false, false, true],
            [true, false, false, true],
            [true, true, true, true],
        ];
    }

    fn pattern_to_pixels<const W: usize, const H: usize>(
        pattern: &[[bool; W]; H],
    ) -> Vec<Vec<bool>> {
        pattern.iter().map(|col| col.to_vec()).collect()
    }

    pub fn rune_to_pixels(rune: &Rune) -> Vec<Vec<bool>> {
        match rune {
            Rune::InclusiveSelf => pattern_to_pixels(&pattern::INCLUSIVE_SELF),
            Rune::Connector => pattern_to_pixels(&pattern::CONNECTOR),
            Rune::ExclusiveSelf => pattern_to_pixels(&pattern::EXCLUSIVE_SELF),
            Rune::Shelter => pattern_to_pixels(&pattern::SHELTER),
            Rune::InclusiveOther => {
                pattern_to_pixels(&pattern::INCLUSIVE_OTHER)
            }
            Rune::Force => pattern_to_pixels(&pattern::FORCE),
            Rune::ExclusiveOther => {
                pattern_to_pixels(&pattern::EXCLUSIVE_OTHER)
            }
        }
    }

    // Given a 2D grid of pixels, return the corresponding rune, if any.
    pub fn pixels_to_rune(pixels: &Vec<Vec<bool>>) -> Option<Rune> {
        let pixels = strip_empty_rows(&strip_empty_columns(pixels));
        if pixels.is_empty() {
            return None;
        }
        let width = pixels[0].len();
        let height = pixels.len();
        if width == 1 && height == 1 {
            return (pattern_to_pixels(&pattern::INCLUSIVE_SELF) == pixels)
                .then_some(Rune::InclusiveSelf);
        }
        if width == 2 && height == 1 {
            return (pattern_to_pixels(&pattern::CONNECTOR) == pixels)
                .then_some(Rune::Connector);
        }
        if width == 2 && height == 2 {
            return (pattern_to_pixels(&pattern::EXCLUSIVE_SELF) == pixels)
                .then_some(Rune::ExclusiveSelf);
        }
        if width == 3 && height == 2 {
            return (pattern_to_pixels(&pattern::SHELTER) == pixels)
                .then_some(Rune::Shelter);
        }
        if width == 3 && height == 3 {
            return (pattern_to_pixels(&pattern::INCLUSIVE_OTHER) == pixels)
                .then_some(Rune::InclusiveOther);
        }
        if width == 4 && height == 3 {
            return (pattern_to_pixels(&pattern::FORCE) == pixels)
                .then_some(Rune::Force);
        }
        if width == 4 && height == 4 {
            return (pattern_to_pixels(&pattern::EXCLUSIVE_OTHER) == pixels)
                .then_some(Rune::ExclusiveOther);
        }
        None
    }

    pub fn strip_empty_rows(pixels: &[Vec<bool>]) -> Vec<Vec<bool>> {
        if pixels.is_empty() {
            return pixels.to_owned();
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

    pub fn strip_empty_columns(pixels: &Vec<Vec<bool>>) -> Vec<Vec<bool>> {
        if pixels.is_empty() || pixels[0].is_empty() {
            return pixels.clone();
        }

        let width = pixels[0].len();
        let mut first_col = 0;
        let mut last_col = width;

        // Find first non-empty column
        'outer: while first_col < last_col {
            for row in pixels {
                if row[first_col] {
                    break 'outer;
                }
            }
            first_col += 1;
        }

        // Find last non-empty column
        'outer: while last_col > first_col {
            for row in pixels {
                if row[last_col - 1] {
                    break 'outer;
                }
            }
            last_col -= 1;
        }

        pixels
            .iter()
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
            assert_eq!(strip_empty_rows(&input), expected);
        }

        #[test]
        fn test_strip_empty_columns() {
            let input = vec![
                vec![false, false, true, false],
                vec![false, true, false, false],
            ];
            let expected = vec![vec![false, true], vec![true, false]];
            assert_eq!(strip_empty_columns(&input), expected);
        }

        #[test]
        fn test_empty_input() {
            let empty: Vec<Vec<bool>> = vec![];
            assert_eq!(strip_empty_rows(&empty), empty.clone());
            assert_eq!(strip_empty_columns(&empty), empty);
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
#[repr(u8)]
pub enum AbstractKind {
    Click,
    XP,
    Rune,
}

const ITEM_SIZE: u32 = 256; // pixels

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum PhysicalItem {
    Bulk(BulkItem),
    Discrete(DiscreteItem),
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct BulkItem {
    pub structure: BulkStructure,
    pub substance: Substance,
    pub processing: Processing,
    pub shape: BulkShape,
    pub quality: u8, // 0..=15
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct DiscreteItem {
    pub species: Species,
    pub state: State,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum State {
    Stage(LifeStage),
    Freshness(u8), // 0..=127
    None,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, IntEnum)]
#[repr(u8)]
pub enum BulkStructure {
    Gas = 0,
    Liquid = 1,
    Powder = 2,
    Solid = 3,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, IntEnum)]
#[repr(u8)]
pub enum Substance {
    Mud = 0,
    Dirt = 1,
    Sandstone = 2,
    Granite = 3,
    Marble = 4,
    Obsidian = 5,
    Moss = 6,
    Copper = 7,
    Tin = 8,
    Bronze = 9,
    Iron = 10,
    Silver = 11,
    Gold = 12,
    Diamond = 13,
    Amethyst = 14,
    Unobtainium = 15,
    SaltWater = 16,
    FreshWater = 17,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, IntEnum)]
#[repr(u8)]
pub enum SubstanceClass {
    Earthen = 0,
    Metal = 1,
    Gem = 2,
    Organic = 3,
    Water = 4,
    Exotic = 5,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, IntEnum)]
#[repr(u8)]
pub enum Processing {
    Raw = 0,
    Refined = 1,
    Worked = 2,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, IntEnum)]
#[repr(u8)]
pub enum BulkShape {
    Lump = 0,
    Block = 1,
    Ball = 2,
    Gravel = 3,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, IntEnum)]
#[repr(u8)]
pub enum LifeStage {
    Seed = 0,
    Baby = 1,
    Youth = 2,
    Adult = 3,
    Elder = 4,
    Corpse = 5,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, IntEnum)]
#[repr(u8)]
pub enum Species {
    Apple = 0,
    Lemon = 1,
    Lime = 2,
    Archaea = 3,
    Bacterium = 4,
    Algae = 5,
    Grass = 6,
    Fern = 7,
    Bush = 8,
    Tree = 9,
    Insect = 10,
    Fish = 11,
    Amphibian = 12,
    Reptile = 13,
    Mammal = 14,
    Bird = 15,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, IntEnum)]
#[repr(u8)]
pub enum DiscreteClass {
    Microbe = 0,
    Plant = 1,
    Animal = 2,
    Fruit = 3,
    Tool = 4,
    Weapon = 5,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, IntEnum)]
#[repr(u8)]
pub enum Animacy {
    Alive = 0,
    Inanimate = 1,
}

impl Substance {
    pub fn class(&self) -> SubstanceClass {
        match self {
            Substance::Mud
            | Substance::Dirt
            | Substance::Sandstone
            | Substance::Granite
            | Substance::Marble
            | Substance::Obsidian
            | Substance::Moss => SubstanceClass::Earthen,
            Substance::Copper
            | Substance::Tin
            | Substance::Bronze
            | Substance::Iron
            | Substance::Silver
            | Substance::Gold => SubstanceClass::Metal,
            Substance::Diamond | Substance::Amethyst => SubstanceClass::Gem,
            Substance::SaltWater | Substance::FreshWater => {
                SubstanceClass::Water
            }
            Substance::Unobtainium => SubstanceClass::Exotic,
        }
    }

    pub fn is_water(&self) -> bool {
        matches!(self, Substance::SaltWater | Substance::FreshWater)
    }

    pub fn is_goo(&self) -> bool {
        matches!(self, Substance::Mud)
    }

    pub fn is_metal(&self) -> bool {
        self.class() == SubstanceClass::Metal
    }

    pub fn name(&self) -> &'static str {
        match self {
            Substance::Mud => "Mud",
            Substance::Dirt => "Dirt",
            Substance::Sandstone => "Sandstone",
            Substance::Granite => "Granite",
            Substance::Marble => "Marble",
            Substance::Obsidian => "Obsidian",
            Substance::Moss => "Moss",
            Substance::Copper => "Copper",
            Substance::Tin => "Tin",
            Substance::Bronze => "Bronze",
            Substance::Iron => "Iron",
            Substance::Silver => "Silver",
            Substance::Gold => "Gold",
            Substance::Diamond => "Diamond",
            Substance::Amethyst => "Amethyst",
            Substance::Unobtainium => "Unobtainium",
            Substance::SaltWater => "Salt Water",
            Substance::FreshWater => "Fresh Water",
        }
    }

    pub fn palette(&self) -> image_gen::ColorPalette {
        match self {
            Substance::Mud => Self::mud_palette(),
            Substance::Dirt => Self::dirt_palette(),
            Substance::Sandstone => Self::sandstone_palette(),
            Substance::SaltWater => Self::salt_water_palette(),
            Substance::FreshWater => Self::fresh_water_palette(),
            _ => panic!("palette not implemented for {:?}", self),
        }
    }

    fn mud_palette() -> image_gen::ColorPalette {
        let mut palette = image_gen::ColorPalette::new();
        palette.add_colorant(image_gen::Colorant::new_loose(87, 39, 12, 10, 1));
        palette
    }

    fn dirt_palette() -> image_gen::ColorPalette {
        let mut palette = image_gen::ColorPalette::new();
        palette.add_colorant(image_gen::Colorant::new_loose(70, 60, 40, 10, 1));
        palette
    }

    fn sandstone_palette() -> image_gen::ColorPalette {
        let mut palette = image_gen::ColorPalette::new();
        palette
            .add_colorant(image_gen::Colorant::new_loose(255, 174, 76, 15, 2));
        palette
            .add_colorant(image_gen::Colorant::new_loose(220, 114, 41, 15, 3));
        palette
    }

    fn salt_water_palette() -> image_gen::ColorPalette {
        let mut palette = image_gen::ColorPalette::new();
        palette.add_colorant(image_gen::Colorant::new_loose(0, 21, 125, 2, 5));
        palette
            .add_colorant(image_gen::Colorant::new_loose(52, 71, 180, 2, 10));
        palette
            .add_colorant(image_gen::Colorant::new_loose(152, 162, 200, 4, 2));
        palette
    }

    fn fresh_water_palette() -> image_gen::ColorPalette {
        let mut palette = image_gen::ColorPalette::new();
        palette.add_colorant(image_gen::Colorant::new_loose(0, 21, 125, 2, 5));
        palette
            .add_colorant(image_gen::Colorant::new_loose(52, 71, 180, 2, 10));
        palette
    }
}

impl Species {
    pub fn class(&self) -> DiscreteClass {
        match self {
            Species::Apple | Species::Lemon | Species::Lime => {
                DiscreteClass::Fruit
            }
            Species::Archaea | Species::Bacterium => DiscreteClass::Microbe,
            Species::Algae
            | Species::Grass
            | Species::Fern
            | Species::Bush
            | Species::Tree => DiscreteClass::Plant,
            Species::Insect
            | Species::Fish
            | Species::Amphibian
            | Species::Reptile
            | Species::Mammal
            | Species::Bird => DiscreteClass::Animal,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Species::Apple => "Apple",
            Species::Lemon => "Lemon",
            Species::Lime => "Lime",
            Species::Archaea => "Archaea",
            Species::Bacterium => "Bacterium",
            Species::Algae => "Algae",
            Species::Grass => "Grass",
            Species::Fern => "Fern",
            Species::Bush => "Bush",
            Species::Tree => "Tree",
            Species::Insect => "Insect",
            Species::Fish => "Fish",
            Species::Amphibian => "Amphibian",
            Species::Reptile => "Reptile",
            Species::Mammal => "Mammal",
            Species::Bird => "Bird",
        }
    }

    fn archaea_palette() -> image_gen::ColorPalette {
        let mut palette = image_gen::ColorPalette::new();
        palette.add_colorant(image_gen::Colorant {
            red: 0,
            green: 10,
            blue: 0,
            alpha: 200,
            weight: 1,
            looseness: 10,
            alpha_looseness: 10,
        });
        palette
    }
}

impl DiscreteClass {
    pub fn animacy(&self) -> Animacy {
        match self {
            DiscreteClass::Microbe
            | DiscreteClass::Plant
            | DiscreteClass::Animal => Animacy::Alive,
            DiscreteClass::Fruit
            | DiscreteClass::Tool
            | DiscreteClass::Weapon => Animacy::Inanimate,
        }
    }
}

impl BulkShape {
    pub fn name(&self) -> &'static str {
        match self {
            BulkShape::Lump => "Lump",
            BulkShape::Block => "Block",
            BulkShape::Ball => "Ball",
            BulkShape::Gravel => "Gravel",
        }
    }
}

impl BulkStructure {
    pub fn name(&self) -> &'static str {
        match self {
            BulkStructure::Gas => "Gas",
            BulkStructure::Liquid => "Liquid",
            BulkStructure::Powder => "Powder",
            BulkStructure::Solid => "Solid",
        }
    }
}

impl PhysicalItem {
    pub fn combine(
        &self,
        other: &PhysicalItem,
        self_amount: f32,
        other_amount: f32,
    ) -> Option<(PhysicalItem, f32)> {
        match (self, other) {
            (PhysicalItem::Bulk(a), PhysicalItem::Bulk(b)) => {
                if a.substance != b.substance {
                    return None;
                }
                // Goo (mud) combines regardless of structure/shape.
                if a.substance.is_goo() {
                    return Some((*self, self_amount + other_amount));
                }
                if a != b {
                    return None;
                }
                if matches!(
                    a.structure,
                    BulkStructure::Gas | BulkStructure::Liquid | BulkStructure::Powder
                ) {
                    Some((*self, self_amount + other_amount))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn draw(&self, rand: &mut WyRand) -> Image {
        match self {
            PhysicalItem::Bulk(b) => {
                let palette = b.substance.palette();
                match b.structure {
                    BulkStructure::Gas => palette
                        .adjust_alpha_looseness(128)
                        .draw_ball(rand, ITEM_SIZE),
                    BulkStructure::Liquid => palette
                        .adjust_alpha_looseness(32)
                        .draw_ball(rand, ITEM_SIZE),
                    BulkStructure::Powder => palette.draw_powder(rand, ITEM_SIZE),
                    BulkStructure::Solid => match b.shape {
                        BulkShape::Lump => palette.draw_lump(rand, ITEM_SIZE),
                        BulkShape::Block => palette.draw_block(rand, ITEM_SIZE),
                        BulkShape::Ball => palette.draw_ball(rand, ITEM_SIZE),
                        BulkShape::Gravel => palette.draw_powder(rand, ITEM_SIZE),
                    },
                }
            }
            PhysicalItem::Discrete(d) => match d.species.class() {
                DiscreteClass::Fruit => load_image(&format!(
                    "assets/physical/{}.png",
                    d.species.name()
                )),
                _ => match d.species {
                    Species::Archaea => {
                        Species::archaea_palette().draw_lump(rand, ITEM_SIZE)
                    }
                    _ => panic!("Invalid species {:?}", d.species),
                },
            },
        }
    }

    pub fn identifier(&self) -> ItemIdentifier {
        let (noun, adjective) = match self {
            PhysicalItem::Bulk(b) => {
                let noun = if b.structure == BulkStructure::Solid {
                    b.shape.name()
                } else {
                    b.structure.name()
                };
                (noun.to_string(), b.substance.name().to_string())
            }
            PhysicalItem::Discrete(d) => match d.species.class() {
                DiscreteClass::Fruit => {
                    (d.species.name().to_string(), "Fruit".to_string())
                }
                _ => {
                    // Alive organisms: noun=species, adjective=life-stage.
                    let stage = match d.state {
                        State::Stage(s) => match s {
                            LifeStage::Seed => "Seed",
                            LifeStage::Baby => "Baby",
                            LifeStage::Youth => "Youth",
                            LifeStage::Adult => "Adult",
                            LifeStage::Elder => "Elder",
                            LifeStage::Corpse => "Corpse",
                        },
                        _ => "",
                    };
                    (d.species.name().to_string(), stage.to_string())
                }
            },
        };
        ItemIdentifier {
            domain: "physical".to_string(),
            noun,
            adjective,
        }
    }

    //
    // Packed identity
    //

    fn pack(&self) -> u64 {
        let mut v = DOMAIN_PHYSICAL << 61;
        match self {
            PhysicalItem::Bulk(b) => {
                v |= PHYS_KIND_BULK << 59;
                v |= (b.structure as u64) << 56;
                v |= (b.substance.class() as u64) << 52;
                v |= (b.substance as u64) << 44;
                v |= (b.processing as u64) << 41;
                v |= (b.shape as u64) << 38;
                v |= ((b.quality & 0xF) as u64) << 34;
            }
            PhysicalItem::Discrete(d) => {
                v |= PHYS_KIND_DISCRETE << 59;
                let class = d.species.class();
                v |= (class.animacy() as u64) << 53;
                v |= (class as u64) << 49;
                let state: u64 = match d.state {
                    State::Stage(s) => s as u64,
                    State::Freshness(f) => (f & 0x7F) as u64,
                    State::None => 0,
                };
                v |= state << 42;
                v |= (d.species as u64) << 22;
            }
        }
        v
    }

    fn unpack(packed: u64) -> Option<PhysicalItem> {
        let kind = (packed >> 59) & 0b11;
        if kind == PHYS_KIND_DISCRETE {
            let class_bits = (packed >> 49) & 0b1111;
            let class = DiscreteClass::try_from(class_bits as u8).ok()?;
            let species_bits = (packed >> 22) & 0xFFFFF;
            // Guard the 20-bit field against `as u8` truncation: a value
            // above 255 must fail, not wrap into a valid species.
            let species = u8::try_from(species_bits)
                .ok()
                .and_then(|n| Species::try_from(n).ok())?;
            // Validate derived class matches the species' true class.
            if species.class() != class {
                return None;
            }
            let state_bits = (packed >> 42) & 0x7F;
            let state = match class {
                DiscreteClass::Fruit => State::Freshness(state_bits as u8),
                DiscreteClass::Microbe
                | DiscreteClass::Plant
                | DiscreteClass::Animal => {
                    State::Stage(LifeStage::try_from(state_bits as u8).ok()?)
                }
                _ => State::None,
            };
            Some(PhysicalItem::Discrete(DiscreteItem { species, state }))
        } else {
            let structure =
                BulkStructure::try_from(((packed >> 56) & 0b111) as u8).ok()?;
            let substance =
                Substance::try_from(((packed >> 44) & 0xFF) as u8).ok()?;
            let processing =
                Processing::try_from(((packed >> 41) & 0b111) as u8).ok()?;
            let shape =
                BulkShape::try_from(((packed >> 38) & 0b111) as u8).ok()?;
            let quality = ((packed >> 34) & 0xF) as u8;
            Some(PhysicalItem::Bulk(BulkItem {
                structure,
                substance,
                processing,
                shape,
                quality,
            }))
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
#[repr(C)]
pub struct ManaItem {
    pub kind: ManaKind,
    pub subkind: u8,
    pub intent: ManaIntent,
}

impl ManaItem {
    pub fn combine(
        &self,
        other: &ManaItem,
        self_amount: f32,
        other_amount: f32,
    ) -> Option<(ManaItem, f32)> {
        // TODO mana combining has weird rules - can actually change the mana type
        if self.kind == other.kind
            && self.subkind == other.subkind
            && self.intent == other.intent
        {
            Some((*self, self_amount + other_amount))
        } else {
            None
        }
    }

    pub fn draw(&self, _rand: &mut WyRand) -> Image {
        panic!("ManaItem::draw not implemented");
    }

    pub fn identifier(&self) -> ItemIdentifier {
        panic!("ManaItem::identifier not implemented");
    }

    fn pack(&self) -> u64 {
        let mut v = DOMAIN_MANA << 61;
        let element = match self.kind {
            ManaKind::Fire => 0u64,
            ManaKind::Water => 1,
            ManaKind::Earth => 2,
            ManaKind::Air => 3,
            ManaKind::Light => 4,
            ManaKind::Dark => 5,
        };
        let intent = match self.intent {
            ManaIntent::Attack => 0u64,
            ManaIntent::Defense => 1,
            ManaIntent::Support => 2,
        };
        v |= element << 58;
        v |= intent << 56;
        v |= (self.subkind as u64) << 48;
        v
    }

    fn unpack(packed: u64) -> Option<ManaItem> {
        let kind = match (packed >> 58) & 0b111 {
            0 => ManaKind::Fire,
            1 => ManaKind::Water,
            2 => ManaKind::Earth,
            3 => ManaKind::Air,
            4 => ManaKind::Light,
            5 => ManaKind::Dark,
            _ => return None,
        };
        let intent = match (packed >> 56) & 0b11 {
            0 => ManaIntent::Attack,
            1 => ManaIntent::Defense,
            2 => ManaIntent::Support,
            _ => return None,
        };
        let subkind = ((packed >> 48) & 0xFF) as u8;
        Some(ManaItem {
            kind,
            subkind,
            intent,
        })
    }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
#[repr(u8)]
pub enum ManaKind {
    Fire,
    Water,
    Earth,
    Air,
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
#[repr(u8)]
pub enum ManaIntent {
    Attack,
    Defense,
    Support,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
#[repr(C)]
pub struct EnergyItem {
    pub kind: EnergyKind,
}

impl EnergyItem {
    pub fn combine(
        &self,
        other: &EnergyItem,
        self_amount: f32,
        other_amount: f32,
    ) -> Option<(EnergyItem, f32)> {
        if self.kind == other.kind {
            Some((*self, self_amount + other_amount))
        } else {
            None
        }
    }

    pub fn draw(&self, _rand: &mut WyRand) -> Image {
        load_image(&format!("assets/energy/{}.png", self.identifier().noun))
    }

    pub fn identifier(&self) -> ItemIdentifier {
        let noun = match self.kind {
            EnergyKind::Kinetic => "kinetic",
            EnergyKind::Potential => "potential",
            EnergyKind::Thermal => "thermal",
            EnergyKind::Electric => "electric",
            EnergyKind::Magnetic => "magnetic",
            EnergyKind::Radiant => "radiant",
        };
        ItemIdentifier {
            domain: "energy".to_string(),
            noun: noun.to_string(),
            adjective: "".to_string(),
        }
    }

    // Energy is a bitmask of kinds it works for; a single-kind item sets one
    // bit. Bit index by kind: Kinetic=0 … Radiant=5, placed at [60:55].
    fn energy_bit(kind: EnergyKind) -> u64 {
        match kind {
            EnergyKind::Kinetic => 0,
            EnergyKind::Potential => 1,
            EnergyKind::Thermal => 2,
            EnergyKind::Electric => 3,
            EnergyKind::Magnetic => 4,
            EnergyKind::Radiant => 5,
        }
    }

    fn pack(&self) -> u64 {
        let mut v = DOMAIN_ENERGY << 61;
        let mask = 1u64 << Self::energy_bit(self.kind);
        v |= mask << 55;
        v
    }

    fn unpack(packed: u64) -> Option<EnergyItem> {
        let mask = (packed >> 55) & 0b111111;
        let kind = match mask.trailing_zeros() {
            0 => EnergyKind::Kinetic,
            1 => EnergyKind::Potential,
            2 => EnergyKind::Thermal,
            3 => EnergyKind::Electric,
            4 => EnergyKind::Magnetic,
            5 => EnergyKind::Radiant,
            _ => return None,
        };
        Some(EnergyItem { kind })
    }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
#[repr(u8)]
pub enum EnergyKind {
    Kinetic,
    Potential,
    Thermal,
    Electric,
    Magnetic,
    Radiant,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
#[repr(C)]
pub struct MinigameItem {
    pub kind: MinigameItemKind,
    pub variant: u32,
}

impl MinigameItem {
    // Minigame items can't combine
    pub fn combine(
        &self,
        _other: &MinigameItem,
        _self_amount: f32,
        _other_amount: f32,
    ) -> Option<(MinigameItem, f32)> {
        None
    }

    pub fn draw(&self, _rand: &mut WyRand) -> Image {
        panic!("MinigameItem::draw not implemented");
    }

    pub fn identifier(&self) -> ItemIdentifier {
        panic!("MinigameItem::identifier not implemented");
    }

    fn pack(&self) -> u64 {
        let mut v = DOMAIN_MINIGAME << 61;
        let which = match self.kind {
            MinigameItemKind::Button => 0u64,
            MinigameItemKind::PrimordialOcean => 1,
            MinigameItemKind::Draw => 2,
            MinigameItemKind::BlockBreaker => 3,
            MinigameItemKind::Tree => 4,
        };
        v |= which << 48;
        v |= (self.variant as u64) & 0xFFFF_FFFF;
        v
    }

    fn unpack(packed: u64) -> Option<MinigameItem> {
        let kind = match (packed >> 48) & 0x1FFF {
            0 => MinigameItemKind::Button,
            1 => MinigameItemKind::PrimordialOcean,
            2 => MinigameItemKind::Draw,
            3 => MinigameItemKind::BlockBreaker,
            4 => MinigameItemKind::Tree,
            _ => return None,
        };
        let variant = (packed & 0xFFFF_FFFF) as u32;
        Some(MinigameItem { kind, variant })
    }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
#[repr(u8)]
pub enum MinigameItemKind {
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

pub fn teleport_distant_loose_items(
    mut query: Query<&mut Transform, (With<Item>, Without<Stuck>)>,
) {
    for mut transform in query.iter_mut() {
        if transform.translation.length() > MAX_ITEM_DISTANCE {
            transform.translation = Vec3::ZERO;
        }
    }
}

pub fn combine_loose_items(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut generated_image_assets: ResMut<image_gen::GeneratedImageAssets>,
    loose_item_query: Query<(&Item, &Transform, &Velocity)>,
    stuck_query: Query<&Stuck>,
    mut collision_events: MessageReader<CollisionEvent>,
) {
    let mut eliminated: HashSet<Entity> = HashSet::new();
    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(entity1, entity2, _) = collision_event {
            // already handled
            if eliminated.contains(entity1) || eliminated.contains(entity2) {
                continue;
            }
            // only loose items handled
            let Ok(items) = loose_item_query.get_many([*entity1, *entity2])
            else {
                continue;
            };
            let (item1, transform1, velocity1) = items[0];
            let (item2, transform2, velocity2) = items[1];

            // combine if possible
            let Some(combined) = item1.combine(item2) else {
                continue;
            };

            // prefer the transform of the stuck item, if any
            let transform = if stuck_query.get(*entity1).is_ok() {
                transform1
            } else {
                transform2
            };

            // despawn both and add a new one
            commands.entity(*entity1).despawn();
            commands.entity(*entity2).despawn();
            eliminated.insert(*entity1);
            eliminated.insert(*entity2);
            commands.spawn(ItemBundle::new(
                &mut images,
                &mut generated_image_assets,
                combined,
                *transform,
                Velocity {
                    linear: velocity1.linear + velocity2.linear,
                    angular: velocity1.angular + velocity2.angular,
                },
            ));
        }
    }
}

pub fn grab_items(
    mut commands: Commands,
    read_rapier_context: ReadRapierContext,
    player_query: Query<(Entity, &CircularArea), (With<Player>, With<Sticky>)>,
    mut loose_item_query: Query<
        (&CircularArea, &mut Velocity),
        (With<Item>, Without<Stuck>),
    >,
    mut collision_events: MessageReader<CollisionEvent>,
) {
    let Ok((player_entity, player_area)) = player_query.single() else {
        return;
    };
    let Ok(rapier_context) = read_rapier_context.single() else {
        return;
    };

    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(entity1, entity2, _) = collision_event {
            let (other, player_is_first) = if *entity1 == player_entity {
                (*entity2, true)
            } else if *entity2 == player_entity {
                (*entity1, false)
            } else {
                continue;
            };

            let Ok((item_area, mut item_velocity)) =
                loose_item_query.get_mut(other)
            else {
                continue;
            };

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

            stick(
                &mut commands,
                player_entity,
                *player_area,
                other,
                *item_area,
                &mut item_velocity,
                direction,
            );
        }
    }
}

pub fn stick(
    commands: &mut Commands,
    player_entity: Entity,
    player_area: CircularArea,
    item_entity: Entity,
    item_area: CircularArea,
    item_velocity: &mut Velocity,
    direction: Vect,
) {
    let distance = player_area.radius + item_area.radius;

    let joint = FixedJointBuilder::new().local_anchor1(direction * distance);
    commands
        .entity(item_entity)
        .insert(ImpulseJoint::new(player_entity, joint))
        .insert(Stuck {
            player: player_entity,
        });
    item_velocity.linear = Vec2::ZERO;
    item_velocity.angular = 0.0;
}

pub fn release_items(
    mut commands: Commands,
    loose_item_query: Query<(Entity, &Stuck), With<Item>>,
    player_query: Query<Entity, (With<Player>, Without<Sticky>)>,
) {
    for (stuck_entity, stuck) in loose_item_query.iter() {
        let player_entity = stuck.player;
        if !player_query.contains(player_entity) {
            continue;
        }
        commands.entity(stuck_entity).remove::<ImpulseJoint>();
        commands.entity(stuck_entity).remove::<Stuck>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The tree minigame produces Apple fruit. Before this fix, identifier()
    // panicked on the Apple form (crashing on fruit spawn), and asset()/draw()
    // pointed at the wrong filename. Guards all three.
    #[test]
    fn apple_fruit_identifier_and_asset_resolve() {
        let apple = Item::fruit(Species::Apple, 1.0);
        let id = apple.r#type.identifier();
        assert_eq!(id.noun, "Apple");
        assert_eq!(id.adjective, "Fruit");
        // Fruit is textured by its species, so the path points at Apple.png
        // (which exists in assets/physical/), not Fruit.png.
        assert_eq!(apple.asset(), "physical/Apple.png");
        assert_eq!(apple.r#type.uid(), "physical/Apple/Fruit");
    }

    fn roundtrip(t: ItemType) {
        let packed = t.pack();
        let unpacked = ItemType::unpack(packed)
            .unwrap_or_else(|| panic!("failed to unpack {:#018x}", packed));
        assert_eq!(unpacked, t, "round-trip mismatch for {:?}", t);
    }

    #[test]
    fn pack_roundtrips_worked_examples() {
        // fresh apple
        roundtrip(Item::fruit(Species::Apple, 1.0).r#type);
        // spoiled apple
        roundtrip(ItemType::Physical(PhysicalItem::Discrete(DiscreteItem {
            species: Species::Apple,
            state: State::Freshness(0),
        })));
        // iron ore (Solid + Raw + Gravel)
        roundtrip(Item::ore(Substance::Iron, 1.0).r#type);
        // iron block (Solid + Refined + Block)
        roundtrip(Item::solid(Substance::Iron, BulkShape::Block, 1.0).r#type);
        // fire-attack mana
        roundtrip(ItemType::Mana(ManaItem {
            kind: ManaKind::Fire,
            intent: ManaIntent::Attack,
            subkind: 0,
        }));
        // rune #12
        roundtrip(ItemType::Abstract(AbstractItem {
            kind: AbstractKind::Rune,
            variant: 12,
        }));
        // a few others
        roundtrip(Item::liquid(Substance::SaltWater, 1.0).r#type);
        roundtrip(Item::powder(Substance::Gold, 1.0).r#type);
        roundtrip(
            Item::organism(Species::Tree, LifeStage::Adult, 1.0).r#type,
        );
        roundtrip(ItemType::Energy(EnergyItem {
            kind: EnergyKind::Thermal,
        }));
        roundtrip(ItemType::Minigame(MinigameItem {
            kind: MinigameItemKind::Tree,
            variant: 7,
        }));
    }

    #[test]
    fn fresh_and_spoiled_apple_have_distinct_ids() {
        let fresh = Item::fruit(Species::Apple, 1.0).r#type;
        let spoiled =
            ItemType::Physical(PhysicalItem::Discrete(DiscreteItem {
                species: Species::Apple,
                state: State::Freshness(0),
            }));
        assert_ne!(fresh.pack(), spoiled.pack());
    }

    #[test]
    fn ore_and_block_have_distinct_ids() {
        let ore = Item::ore(Substance::Iron, 1.0).r#type;
        let block = Item::solid(Substance::Iron, BulkShape::Block, 1.0).r#type;
        assert_ne!(ore.pack(), block.pack());
    }
}
