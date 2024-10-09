use bevy::input::Input;
use bevy::prelude::*;

// Components
#[derive(Component)]
struct NebulousSpace;

#[derive(Component)]
struct ButtonMinigame {
    clicks: u32,
}

#[derive(Component)]
struct ResourceWallet {
    clicks: u32,
}

// Resource
#[derive(Resource)]
struct GameState {
    camera_speed: f32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(GameState {
            camera_speed: 500.0,
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (button_click_system, move_camera, update_wallet_display).chain(),
        )
        .run();
}

fn setup(mut commands: Commands) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    // Nebulous Space
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.1, 0.1, 0.1),
                custom_size: Some(Vec2::new(1000.0, 1000.0)),
                ..default()
            },
            ..default()
        },
        NebulousSpace,
    ));

    // Button Minigame
    commands
        .spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(150.0),
                    height: Val::Px(65.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::srgb(0.15, 0.15, 0.15).into(),
                ..default()
            },
            ButtonMinigame { clicks: 0 },
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Click me!",
                TextStyle {
                    font_size: 40.0,
                    color: Color::srgb(0.9, 0.9, 0.9),
                    ..default()
                },
            ));
        });

    // Resource Wallet
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.8, 0.8, 0.8),
                custom_size: Some(Vec2::new(100.0, 100.0)),
                ..default()
            },
            transform: Transform::from_xyz(400.0, 300.0, 0.0),
            ..default()
        },
        ResourceWallet { clicks: 0 },
    ));
}

fn button_click_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut ButtonMinigame),
        (Changed<Interaction>, With<Button>),
    >,
    mut wallet_query: Query<&mut ResourceWallet>,
) {
    for (interaction, mut color, mut button) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = Color::srgb(0.35, 0.75, 0.35).into();
                button.clicks += 1;
                if let Ok(mut wallet) = wallet_query.get_single_mut() {
                    wallet.clicks += 1;
                }
            }
            Interaction::Hovered => {
                *color = Color::srgb(0.25, 0.25, 0.25).into();
            }
            Interaction::None => {
                *color = Color::srgb(0.15, 0.15, 0.15).into();
            }
        }
    }
}

fn move_camera(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Camera>>,
    game_state: Res<GameState>,
) {
    let mut camera_transform = query.single_mut();
    let mut direction = Vec3::ZERO;

    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        direction -= Vec3::new(1.0, 0.0, 0.0);
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) {
        direction += Vec3::new(1.0, 0.0, 0.0);
    }
    if keyboard_input.pressed(KeyCode::ArrowUp) {
        direction += Vec3::new(0.0, 1.0, 0.0);
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) {
        direction -= Vec3::new(0.0, 1.0, 0.0);
    }

    if direction != Vec3::ZERO {
        camera_transform.translation += direction.normalize()
            * game_state.camera_speed
            * time.delta_seconds();
    }
}

fn update_wallet_display(
    wallet_query: Query<&ResourceWallet>,
    button_query: Query<&ButtonMinigame>,
) {
    if let Ok(wallet) = wallet_query.get_single() {
        if let Ok(button) = button_query.get_single() {
            println!(
                "Wallet clicks: {}, Button clicks: {}",
                wallet.clicks, button.clicks
            );
        }
    }
}
