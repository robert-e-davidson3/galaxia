use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use crate::entities::*;
use crate::libs::*;

pub const ID: &str = "button";

pub const NAME: &str = "Button";
pub const DESCRIPTION: &str = "Click the button, get clicks!";
const AREA: RectangularArea = RectangularArea {
    width: 200.0,
    height: 220.0,
};

#[derive(Debug, Default, Clone, Component)]
pub struct ButtonMinigame {
    pub count: u64,
    pub level: u8,
}

impl ButtonMinigame {
    pub fn new(clicks: u64) -> Self {
        Self {
            count: clicks,
            level: Self::level_by_clicks(clicks),
        }
    }

    //
    // COMMON
    //

    pub fn name(&self) -> &str {
        NAME
    }

    pub fn description(&self) -> &str {
        DESCRIPTION
    }

    pub fn area(&self) -> RectangularArea {
        AREA
    }

    pub fn level(&self) -> u8 {
        self.level
    }

    pub fn levelup(&self) -> Self {
        Self::new(self.count)
    }

    pub fn spawn(&self, parent: &mut ChildBuilder) {
        spawn_background(parent);
        let text = spawn_text(parent, self.count);
        spawn_button(parent, text);
    }

    //
    // SPECIFIC
    //

    pub fn level_by_clicks(clicks: u64) -> u8 {
        if clicks == 0 {
            0
        } else {
            (((clicks as f32).log2() + 1.0) as u8).min(99)
        }
    }

    pub fn should_level_up(&self) -> bool {
        if self.count == 0 {
            false
        } else {
            Self::level_by_clicks(self.count) > self.level
        }
    }
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
    mut images: ResMut<Assets<Image>>,
    mut generated_image_assets: ResMut<image_gen::GeneratedImageAssets>,
    clickable_query: Query<(&ClickMeButton, &GlobalTransform, &CircularArea)>,
    mouse_state: Res<MouseState>,
    mut minigame_query: Query<(
        &mut Minigame,
        &GlobalTransform,
        &RectangularArea,
    )>,
    mut text_query: Query<&mut Text>,
    leveling_up_query: Query<&LevelingUp>,
) {
    if !mouse_state.just_released {
        return;
    }
    let click_position = mouse_state.current_position;

    for (button, global_transform, area) in clickable_query.iter() {
        if area.is_within(
            click_position,
            global_transform.translation().truncate(),
        ) {
            // Skip if already leveling up
            if leveling_up_query.get(button.game).is_ok() {
                continue;
            }

            let (minigame, minigame_transform, minigame_area) =
                match minigame_query.get_mut(button.game) {
                    Ok(x) => x,
                    Err(_) => continue,
                };
            let minigame = match minigame.into_inner() {
                Minigame::Button(minigame) => minigame,
                _ => continue,
            };
            minigame.count += 1;
            let mut text = text_query.get_mut(button.text).unwrap();
            text.sections[0].value = format!("Clicks: {}", minigame.count);

            // Check for level up condition
            if minigame.should_level_up() {
                commands.entity(button.game).insert(LevelingUp);
            }

            let click_type = mouse_state.get_click_type();
            let variant = match click_type {
                ClickType::Short => 0,
                ClickType::Long => 1,
                ClickType::Invalid => {
                    println!("unexpected: invalid click type");
                    continue;
                }
            };
            commands.spawn(ItemBundle::new_from_minigame(
                &mut images,
                &mut generated_image_assets,
                Item::new_abstract(AbstractItemKind::Click, variant, 1.0),
                minigame_transform,
                minigame_area,
            ));
        }
    }
}
