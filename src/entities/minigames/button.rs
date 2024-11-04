use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use crate::entities::*;
use crate::libs::*;

pub const NAME: &str = "Button";
pub const _DESCRIPTION: &str = "Click the button, get clicks!";
const AREA: RectangularArea = RectangularArea {
    width: 200.0,
    height: 220.0,
};

#[derive(Debug, Default, Bundle)]
pub struct ButtonMinigameBundle {
    pub minigame: ButtonMinigame,
    pub area: RectangularArea,
    pub tag: Minigame,
    pub spatial: SpatialBundle,
}

impl ButtonMinigameBundle {
    pub fn new(minigame: ButtonMinigame, transform: Transform) -> Self {
        Self {
            minigame,
            area: AREA,
            tag: Minigame,
            spatial: SpatialBundle {
                transform,
                ..default()
            },
        }
    }
}

#[derive(Debug, Default, Clone, Component)]
pub struct ButtonMinigame {
    pub count: u64,
}

pub fn spawn(
    commands: &mut Commands,
    transform: Transform,
    frozen: &ButtonMinigame,
) {
    commands
        .spawn(ButtonMinigameBundle::new(frozen.clone(), transform))
        .with_children(|parent| {
            parent.spawn(MinigameAuraBundle::new(parent.parent_entity(), AREA));
            spawn_minigame_container(parent, AREA, NAME);
            spawn_background(parent);
            let text = spawn_text(parent, frozen.count);
            spawn_button(parent, text);
        });
}

fn spawn_background(parent: &mut ChildBuilder) {
    parent.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::srgb(0.9, 0.9, 0.9),
            custom_size: Some(Vec2::new(AREA.width, AREA.height)),
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, -1.0),
        ..default()
    });
}

fn spawn_text(parent: &mut ChildBuilder, initial_clicks: u64) -> Entity {
    parent
        .spawn(Text2dBundle {
            text: Text::from_section(
                format!("Clicks: {}", initial_clicks),
                TextStyle {
                    font_size: 24.0,
                    color: Color::BLACK,
                    ..default()
                },
            ),
            transform: Transform::from_xyz(0.0, 100.0, 0.0),
            ..default()
        })
        .id()
}

fn spawn_button(parent: &mut ChildBuilder, text: Entity) {
    parent.spawn((
        ClickMeButton {
            game: parent.parent_entity(),
            text,
        },
        CircularArea { radius: 90.0 },
        ShapeBundle {
            path: GeometryBuilder::build_as(&shapes::Circle {
                radius: 90.0,
                ..default()
            }),
            spatial: SpatialBundle {
                transform: Transform::from_xyz(0.0, -18.0, 0.0),
                ..default()
            },
            ..default()
        },
        Fill::color(Color::srgb(0.8, 0.1, 0.1)),
        Stroke::new(Color::BLACK, 2.0),
    ));
}

#[derive(Debug, Component)]
pub struct ClickMeButton {
    pub game: Entity,
    pub text: Entity,
}

pub fn update(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    clickable_query: Query<(&ClickMeButton, &GlobalTransform, &CircularArea)>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mouse_state: Res<MouseState>,
    time: Res<Time>,
    mut button_minigames_query: Query<(
        &mut ButtonMinigame,
        &GlobalTransform,
        &RectangularArea,
    )>,
    mut text_query: Query<&mut Text>,
) {
    let click_position = match get_click_release_position(
        camera_query,
        windows,
        mouse_button_input,
    ) {
        Some(world_position) => world_position,
        None => return,
    };

    for (button, global_transform, area) in clickable_query.iter() {
        if area.is_within(
            click_position,
            global_transform.translation().truncate(),
        ) {
            let (mut minigame, minigame_transform, minigame_area) =
                button_minigames_query.get_mut(button.game).unwrap();
            minigame.count += 1;
            let mut text = text_query.get_mut(button.text).unwrap();
            text.sections[0].value = format!("Clicks: {}", minigame.count);

            let click_type = mouse_state.get_click_type(time.elapsed_seconds());
            let variant = match click_type {
                ClickType::Short => 0,
                ClickType::Long => 1,
                ClickType::Invalid => {
                    println!("unexpected: invalid click type");
                    continue;
                }
            };
            commands.spawn(ItemBundle::new_from_minigame(
                &asset_server,
                Item::new_abstract(AbstractItemKind::Click, variant, 1.0),
                minigame_transform,
                minigame_area,
            ));
        }
    }
}
