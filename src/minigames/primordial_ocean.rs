use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use crate::area::*;
use crate::common::*;
use crate::mouse::*;
use crate::resource::*;

pub const NAME: &str = "Primordial Ocean";
pub const DESCRIPTION: &str = "Infinitely deep, the source of water and mud.";

#[derive(Debug, Clone, Bundle)]
pub struct PrimordialOceanMinigameBundle {
    pub minigame: Minigame,
    pub primordial_ocean_minigame: PrimordialOceanMinigame,
    pub area: CircularArea,
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
    commands
        .spawn((
            PrimordialOceanMinigameBundle {
                minigame: Minigame,
                primordial_ocean_minigame: frozen.clone(),
                area: CircularArea { radius },
            },
            SpatialBundle {
                transform,
                ..default()
            },
        ))
        .with_children(|parent| {
            spawn_minigame_container(
                parent,
                RectangularArea::new_square(radius * 2.0),
                NAME,
            );
            spawn_ocean(parent, radius);
        });
}

#[derive(Debug, Clone, Component)]
pub struct Ocean {
    pub minigame: Entity,
}

fn spawn_ocean(parent: &mut ChildBuilder, radius: f32) {
    let minigame = parent.parent_entity();
    parent.spawn((
        ShapeBundle {
            path: GeometryBuilder::build_as(&shapes::Circle {
                radius,
                ..default()
            }),
            ..default()
        },
        Fill::color(Color::srgb(0.0, 0.25, 1.0)),
        Ocean { minigame },
        CircularArea { radius },
    ));
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
        (&GlobalTransform, &CircularArea),
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
            spawn_loose_resource(
                &mut commands,
                &asset_server,
                resource,
                1.0,
                Transform::from_translation(
                    minigame_transform.translation()
                        + minigame_area.dimensions3() / 1.8,
                ),
            );
        }
    }
}
