use std::collections::HashSet;
use std::mem::discriminant;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use wyrand::WyRand;

use crate::entities::*;
use crate::libs::*;

pub const MAX_ITEM_DISTANCE: f32 = 10000.0;
pub const SEED: u64 = 91;

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
        let texture: Handle<Image> =
            match generated_image_assets.get(&item.uid()) {
                Some(texture) => texture,
                None => {
                    let image = item.draw(&mut WyRand::new(SEED));
                    let texture = images.add(image.clone());
                    generated_image_assets.insert(item.uid(), &texture);
                    texture
                }
            };
        Self {
            item,
            area,
            sprite: SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(area.into()),
                    ..default()
                },
                texture,
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

    pub fn new_physical(
        form: PhysicalForm,
        material: PhysicalMaterial,
        amount: f32,
    ) -> Self {
        Self::new(ItemType::Physical(PhysicalItem { form, material }), amount)
    }

    pub fn combine(&self, other: &Self) -> Option<Self> {
        if discriminant(&self.r#type) != discriminant(&other.r#type) {
            return None;
        }

        match match (self.r#type, other.r#type) {
            (ItemType::Abstract(a), ItemType::Abstract(b)) => {
                match a.combine(&b, self.amount, other.amount) {
                    Some((t, a)) => Some((ItemType::Abstract(t), a)),
                    None => None,
                }
            }
            (ItemType::Physical(a), ItemType::Physical(b)) => {
                match a.combine(&b, self.amount, other.amount) {
                    Some((t, a)) => Some((ItemType::Physical(t), a)),
                    None => None,
                }
            }
            (ItemType::Mana(a), ItemType::Mana(b)) => {
                match a.combine(&b, self.amount, other.amount) {
                    Some((t, a)) => Some((ItemType::Mana(t), a)),
                    None => None,
                }
            }
            (ItemType::Energy(a), ItemType::Energy(b)) => {
                match a.combine(&b, self.amount, other.amount) {
                    Some((t, a)) => Some((ItemType::Energy(t), a)),
                    None => None,
                }
            }
            (ItemType::Minigame(a), ItemType::Minigame(b)) => {
                match a.combine(&b, self.amount, other.amount) {
                    Some((t, a)) => Some((ItemType::Minigame(t), a)),
                    None => None,
                }
            }
            _ => None, // mismatched types
        } {
            Some((r#type, amount)) => Some(Self { r#type, amount }),
            None => None,
        }
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
    pub fn to_item(&self, amount: f32) -> Item {
        Item::new(*self, amount)
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

    pub fn uid(&self) -> String {
        format!("{}/{}/{}", self.domain, self.noun, self.adjective)
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
            Some((self.clone(), self_amount + other_amount))
        } else {
            None
        }
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
        let mut pixels: Vec<Vec<bool>> = Vec::with_capacity(H);
        for col in pattern.iter() {
            let mut row: Vec<bool> = Vec::with_capacity(W);
            for &pixel in col.iter() {
                row.push(pixel);
            }
            pixels.push(row);
        }
        pixels
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

    pub fn strip_empty_rows(pixels: &Vec<Vec<bool>>) -> Vec<Vec<bool>> {
        if pixels.is_empty() {
            return pixels.clone();
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
            .into_iter()
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

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
#[repr(C)]
pub struct PhysicalItem {
    pub form: PhysicalForm,
    pub material: PhysicalMaterial,
}

const ITEM_SIZE: u32 = 256; // pixels

impl PhysicalItem {
    pub fn combine(
        &self,
        other: &PhysicalItem,
        self_amount: f32,
        other_amount: f32,
    ) -> Option<(PhysicalItem, f32)> {
        if self.material != other.material {
            return None;
        }
        if self.material.is_goo() {
            return Some((self.clone(), self_amount + other_amount));
        }
        if self.form != other.form {
            return None;
        }
        if match self.form {
            PhysicalForm::Gas => true,
            PhysicalForm::Liquid => true,
            PhysicalForm::Powder => true,
            _ => false,
        } {
            Some((self.clone(), self_amount + other_amount))
        } else {
            None
        }
    }

    pub fn draw(&self, rand: &mut WyRand) -> Image {
        match self.form {
            PhysicalForm::Gas => self
                .material
                .palette()
                .adjust_alpha_looseness(128)
                .draw_ball(rand, ITEM_SIZE),
            PhysicalForm::Liquid => self
                .material
                .palette()
                .adjust_alpha_looseness(32)
                .draw_ball(rand, ITEM_SIZE),
            PhysicalForm::Powder => {
                self.material.palette().draw_powder(rand, ITEM_SIZE)
            }
            PhysicalForm::Lump => {
                self.material.palette().draw_lump(rand, ITEM_SIZE)
            }
            PhysicalForm::Block => {
                self.material.palette().draw_block(rand, ITEM_SIZE)
            }
            PhysicalForm::Ball => {
                self.material.palette().draw_ball(rand, ITEM_SIZE)
            }
            PhysicalForm::Land => {
                self.material.palette().draw_block(rand, ITEM_SIZE)
            }
            PhysicalForm::Sea => {
                self.material.palette().draw_block(rand, ITEM_SIZE)
            }
            PhysicalForm::Archaea => {
                self.form.palette().draw_lump(rand, ITEM_SIZE)
            }
            PhysicalForm::Apple => {
                load_image(&"assets/physical/apple.png".to_string())
            }
            _ => panic!("Invalid form {:?}", self.form),
        }
    }

    pub fn identifier(&self) -> ItemIdentifier {
        let noun: &str;
        let adjective: &str;
        match self.form {
            PhysicalForm::Gas => noun = "Gas",
            PhysicalForm::Liquid => noun = "Liquid",
            PhysicalForm::Powder => noun = "Powder",
            PhysicalForm::Lump => noun = "Lump",
            PhysicalForm::Block => noun = "Block",
            PhysicalForm::Ball => noun = "Ball",
            PhysicalForm::Land => noun = "Land",
            PhysicalForm::Sea => noun = "Sea",
            PhysicalForm::Archaea => noun = "Archaea",
            _ => panic!("Invalid form {:?}", self.form),
        }
        match self.material {
            // life
            PhysicalMaterial::Seed => adjective = "Seed",
            PhysicalMaterial::Baby => adjective = "Baby",
            PhysicalMaterial::Youth => adjective = "Youth",
            PhysicalMaterial::Adult => adjective = "Adult",
            PhysicalMaterial::Elder => adjective = "Elder",
            PhysicalMaterial::Corpse => adjective = "Corpse",
            PhysicalMaterial::Fruit => adjective = "Fruit",
            // minerals
            PhysicalMaterial::Mud => adjective = "Mud",
            PhysicalMaterial::Dirt => adjective = "Dirt",
            PhysicalMaterial::Sandstone => adjective = "Sandstone",
            PhysicalMaterial::Granite => adjective = "Granite",
            PhysicalMaterial::Marble => adjective = "Marble",
            PhysicalMaterial::Obsidian => adjective = "Obsidian",
            PhysicalMaterial::Copper => adjective = "Copper",
            PhysicalMaterial::Tin => adjective = "Tin",
            PhysicalMaterial::Bronze => adjective = "Bronze",
            PhysicalMaterial::Iron => adjective = "Iron",
            PhysicalMaterial::Silver => adjective = "Silver",
            PhysicalMaterial::Gold => adjective = "Gold",
            PhysicalMaterial::Diamond => adjective = "Diamond",
            PhysicalMaterial::Amethyst => adjective = "Amethyst",
            PhysicalMaterial::Moss => adjective = "Moss",
            // liquids
            PhysicalMaterial::SaltWater => adjective = "Salt Water",
            PhysicalMaterial::FreshWater => adjective = "Fresh Water",
            _ => panic!("Invalid material {:?}", self.material),
        }
        ItemIdentifier {
            domain: "physical".to_string(),
            noun: noun.to_string(),
            adjective: adjective.to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
#[repr(u8)]
pub enum PhysicalForm {
    Gas,
    Liquid,
    Powder,
    // solids
    Lump,
    Block,
    Ball,
    Ore,
    // terrain
    Land,
    Sea,
    // generic life
    Archaea,
    Bacterium,
    Algae,
    Grass,
    Fern,
    Bush,
    Tree,
    Insect,
    Fish,
    Amphibian,
    Reptile,
    Mammal,
    Bird,
    // specific life
    Apple,
    Lemon,
    Lime,
}

impl PhysicalForm {
    pub fn palette(&self) -> image_gen::ColorPalette {
        match self {
            PhysicalForm::Archaea => Self::archaea_palette(),
            _ => panic!("Invalid form {:?}", self),
        }
    }

    //
    // Palettes
    //

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

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
#[repr(u64)]
pub enum PhysicalMaterial {
    // life
    Seed, // incl egg
    Baby,
    Youth,
    Adult, // generic alive
    Elder,
    Corpse, // generic dead
    Fruit,
    // minerals
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
    // liquids
    SaltWater,
    FreshWater,
}

impl PhysicalMaterial {
    //
    // Types
    //

    pub fn is_goo(&self) -> bool {
        match self {
            PhysicalMaterial::Mud => true,
            _ => false,
        }
    }

    pub fn is_water(&self) -> bool {
        match self {
            PhysicalMaterial::SaltWater => true,
            PhysicalMaterial::FreshWater => true,
            _ => false,
        }
    }

    pub fn is_metal(&self) -> bool {
        match self {
            PhysicalMaterial::Copper
            | PhysicalMaterial::Tin
            | PhysicalMaterial::Bronze
            | PhysicalMaterial::Iron
            | PhysicalMaterial::Silver
            | PhysicalMaterial::Gold => true,
            _ => false,
        }
    }

    //
    // Palettes
    //

    pub fn palette(&self) -> image_gen::ColorPalette {
        match self {
            PhysicalMaterial::Mud => Self::mud_palette(),
            PhysicalMaterial::Dirt => Self::dirt_palette(),
            PhysicalMaterial::Sandstone => Self::sandstone_palette(),
            PhysicalMaterial::SaltWater => Self::salt_water_palette(),
            PhysicalMaterial::FreshWater => Self::fresh_water_palette(),
            _ => panic!("palette not implemented for {:?}", self),
        }
    }

    fn mud_palette() -> image_gen::ColorPalette {
        let mut palette = image_gen::ColorPalette::new();
        // palette.add_color(image_gen::Colorant::new_tight(100, 40, 200, 1));
        // palette.add_color(image_gen::Colorant::new_loose(16, 16, 4, 0, 1));
        // palette.add_color(image_gen::Colorant::new_loose(61, 32, 0, 0, 1));
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
            Some((self.clone(), self_amount + other_amount))
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
            Some((self.clone(), self_amount + other_amount))
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
                // only loose items handled
                let items =
                    match loose_item_query.get_many([*entity1, *entity2]) {
                        Ok(r) => r,
                        Err(_) => continue,
                    };
                let (item1, transform1, velocity1) = items[0];
                let (item2, transform2, velocity2) = items[1];

                // combine if possible
                let combined = match item1.combine(&item2) {
                    Some(c) => c,
                    None => continue,
                };

                // prefer the transform of the stuck item, if any
                let transform = match stuck_query.get(*entity1) {
                    Ok(_) => transform1,
                    Err(_) => transform2,
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
                        linvel: velocity1.linvel + velocity2.linvel,
                        angvel: velocity1.angvel + velocity2.angvel,
                    },
                ));
            }
            _ => {}
        }
    }
}

pub fn grab_items(
    mut commands: Commands,
    rapier_context: Res<RapierContext>,
    player_query: Query<(Entity, &CircularArea), (With<Player>, With<Sticky>)>,
    mut loose_item_query: Query<
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

                let Ok(item) = loose_item_query.get_mut(other) else {
                    continue;
                };
                let (item_area, mut item_velocity) = item;

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
            _ => {}
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
    item_velocity.linvel = Vec2::ZERO;
    item_velocity.angvel = 0.0;
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
