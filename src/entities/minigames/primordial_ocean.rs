use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use crate::entities::*;
use crate::libs::*;

pub const NAME: &str = "Primordial Ocean";
pub const _DESCRIPTION: &str = "Infinitely deep, the source of water and mud.";

#[derive(Debug, Clone, Bundle)]
pub struct PrimordialOceanMinigameBundle {
    pub minigame: PrimordialOceanMinigame,
    pub area: RectangularArea,
    pub tag: Minigame,
    pub spatial: SpatialBundle,
}

impl PrimordialOceanMinigameBundle {
    pub fn new(
        minigame: PrimordialOceanMinigame,
        radius: f32,
        transform: Transform,
    ) -> Self {
        let area = RectangularArea::new_square(radius * 2.0);
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

#[derive(Debug, Clone, Component)]
pub struct PrimordialOceanMinigame {
    pub size: f32,
}

impl Default for PrimordialOceanMinigame {
    fn default() -> Self {
        Self { size: 120.0 }
    }
}

pub fn spawn(
    commands: &mut Commands,
    transform: Transform,
    frozen: &PrimordialOceanMinigame,
) {
    let radius = frozen.size;
    let area = RectangularArea::new_square(radius * 2.0);
    commands
        .spawn(PrimordialOceanMinigameBundle::new(
            frozen.clone(),
            radius,
            transform,
        ))
        .with_children(|parent| {
            parent.spawn(MinigameAuraBundle::new(parent.parent_entity(), area));
            spawn_minigame_container(parent, area, NAME);
            parent.spawn(OceanBundle::new(parent.parent_entity(), radius));
        });
}

#[derive(Bundle)]
pub struct OceanBundle {
    pub ocean: Ocean,
    pub area: CircularArea,
    pub shape: ShapeBundle,
    pub fill: Fill,
}

impl OceanBundle {
    pub fn new(minigame: Entity, radius: f32) -> Self {
        let area = CircularArea::new(radius);
        Self {
            ocean: Ocean { minigame },
            area,
            shape: ShapeBundle {
                path: GeometryBuilder::build_as(&shapes::Circle {
                    radius,
                    ..default()
                }),
                ..default()
            },
            fill: Fill::color(Color::srgb(0.0, 0.25, 1.0)),
        }
    }
}

#[derive(Debug, Clone, Component)]
pub struct Ocean {
    pub minigame: Entity,
}

pub fn update(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mouse_state: Res<MouseState>,
    time: Res<Time>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    window_query: Query<&Window>,
    primordial_ocean_minigame_query: Query<
        (&GlobalTransform, &RectangularArea),
        With<PrimordialOceanMinigame>,
    >,
    mut ocean_query: Query<(&Ocean, &GlobalTransform, &CircularArea)>,
) {
    let click_position = match get_click_release_position(
        camera_query,
        window_query,
        mouse_button_input,
    ) {
        Some(position) => position,
        None => return,
    };

    for (ocean, ocean_transform, ocean_area) in ocean_query.iter_mut() {
        if ocean_area
            .is_within(click_position, ocean_transform.translation().truncate())
        {
            let (minigame_transform, minigame_area) =
                primordial_ocean_minigame_query.get(ocean.minigame).unwrap();

            let click_type = mouse_state.get_click_type(time.elapsed_seconds());
            let resource = match click_type {
                ClickType::Short => GalaxiaResource::SaltWater,
                ClickType::Long => GalaxiaResource::Mud,
                ClickType::Invalid => {
                    println!("unexpected: invalid click type");
                    continue;
                }
            };
            commands.spawn(LooseResourceBundle::new_from_minigame(
                &asset_server,
                resource,
                1.0,
                minigame_transform,
                minigame_area.into(),
            ));
        }
    }
}
