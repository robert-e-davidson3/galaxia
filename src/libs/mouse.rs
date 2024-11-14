use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use crate::libs::*;

// MouseState process looks like:
// 0. Position starts at (0,0) until the second frame.
// 1. Unpressed. Position is always tracked.
//    Request for click type returns Invalid.
// 2. Mouse is "just_pressed" -> start tracking time
// 2. Each frame, update time and position
// 4. Request for click type returns Short or Long.
// 5. Mouse is "just_released" -> stop tracking time
// 6. For one more frame, request for click type returns Short or Long
// 7. After one frame, request for click type returns Invalid.
#[derive(Resource, Default)]
pub struct MouseState {
    pub long_click_threshold: f32,
    pub start_time: Option<f32>,
    pub drag_time: f32,
    pub start_position: Option<Vec2>,
    pub current_position: Vec2,
    pub just_pressed: bool,
    pub just_released: bool,
}

impl MouseState {
    pub fn new(long_click_threshold: f32) -> Self {
        Self {
            long_click_threshold,
            start_time: None,
            drag_time: 0.0,
            start_position: None,
            current_position: Vec2::ZERO,
            just_pressed: false,
            just_released: false,
        }
    }

    pub fn get_click_type(&self) -> ClickType {
        if self.start_time.is_none() {
            return ClickType::Invalid;
        }
        if self.drag_time >= self.long_click_threshold {
            ClickType::Long
        } else {
            ClickType::Short
        }
    }

    pub fn dragging(&self) -> bool {
        self.start_position.is_some()
    }

    pub fn update_state(&mut self, position: Vec2, elapsed_seconds: f32) {
        self.current_position = position;
        match self.start_time {
            Some(start_time) => {
                self.drag_time = elapsed_seconds - start_time;
            }
            _ => {}
        }
    }

    pub fn start_press(&mut self, time: f32) {
        self.start_time = Some(time);
        self.start_position = Some(self.current_position);
        self.just_pressed = true;
        self.just_released = false;
    }

    pub fn still_pressed(&mut self) {
        self.just_pressed = false;
        self.just_released = false;
    }

    pub fn end_press(&mut self) {
        self.just_pressed = false;
        self.just_released = true;
    }

    pub fn unpressed(&mut self) {
        self.start_time.take();
        self.start_position.take();
        self.drag_time = 0.0;
        self.just_released = false;
    }
}

#[derive(Debug, PartialEq)]
pub enum ClickType {
    Short,
    Long,
    Invalid,
}

pub fn update_mouse_state(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    window_query: Query<&Window>,
    time: Res<Time>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut mouse_state: ResMut<MouseState>,
) {
    if let Some(position) = get_mouse_position(&camera_query, &window_query) {
        mouse_state.update_state(position, time.elapsed_seconds());
    }

    if mouse_button_input.just_pressed(MouseButton::Left) {
        mouse_state.start_press(time.elapsed_seconds());
    } else if mouse_button_input.just_released(MouseButton::Left) {
        mouse_state.end_press();
    } else if mouse_state.just_released {
        mouse_state.unpressed();
    } else {
        mouse_state.still_pressed();
    }
}

#[derive(Debug, Copy, Clone, Component)]
pub struct FollowsMouse {
    pub bounds: RectangularArea,
    pub bound_center: Vec2,
    pub entity_area: RectangularArea,
    // offset from the center of the entity - usually where the user clicked
    pub click_offset: Vec2,
    pub only_while_dragging: bool,
}

impl FollowsMouse {
    pub fn new(
        bounds: RectangularArea,
        bound_center: Vec2,
        entity_area: RectangularArea,
        click_offset: Vec2,
        only_while_dragging: bool,
    ) -> Self {
        Self {
            bounds,
            bound_center,
            entity_area,
            click_offset,
            only_while_dragging,
        }
    }
}

pub fn follow_mouse_update(
    mut commands: Commands,
    mouse_state: Res<MouseState>,
    mut query: Query<(Entity, &FollowsMouse, &mut Transform, &GlobalTransform)>,
) {
    let mouse_position = mouse_state.current_position;
    let is_dragging = mouse_state.dragging();

    for (entity, follows_mouse, mut transform, global_transform) in
        query.iter_mut()
    {
        if follows_mouse.only_while_dragging && !is_dragging {
            commands.entity(entity).remove::<FollowsMouse>();
            continue;
        }

        let old_global_position = global_transform.translation().truncate();
        let bounds = follows_mouse
            .bounds
            .grow(-follows_mouse.entity_area.width, 0.0);
        let new_global_position = bounds.clamp(
            mouse_position - follows_mouse.click_offset,
            follows_mouse.bound_center,
        );

        // delta needed because GlobalTransform is read-only
        let delta = new_global_position - old_global_position;
        transform.translation.x += delta.x;
        transform.translation.y += delta.y;
    }
}

pub fn get_click_press_position(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    window_query: Query<&Window>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
) -> Option<Vec2> {
    // TODO: https://bevy-cheatbook.github.io/programming/run-conditions.html
    if !mouse_button_input.just_pressed(MouseButton::Left) {
        return None;
    }
    get_mouse_position(&camera_query, &window_query)
}

pub fn get_click_release_position(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    window_query: Query<&Window>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
) -> Option<Vec2> {
    // TODO: https://bevy-cheatbook.github.io/programming/run-conditions.html
    if !mouse_button_input.just_released(MouseButton::Left) {
        return None;
    }
    get_mouse_position(&camera_query, &window_query)
}

fn get_mouse_position(
    camera_query: &Query<(&Camera, &GlobalTransform)>,
    window_query: &Query<&Window>,
) -> Option<Vec2> {
    let (camera, camera_transform) = camera_query.single();
    let window = window_query.single();
    let world_position =
        translate_to_world_position(window, camera, camera_transform);
    return world_position;
}

fn translate_to_world_position(
    window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Vec2> {
    window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
}

#[derive(Component)]
pub struct ClickIndicator {}

#[derive(Resource)]
pub struct ClickIndicatorConfig {
    pub radius: f32,
    pub color: Color,
    pub long_color: Color,
    pub stroke_width: f32,
}

impl Default for ClickIndicatorConfig {
    fn default() -> Self {
        Self {
            radius: 10.0,
            color: Color::srgba(1.0, 0.5, 0.0, 1.0),
            long_color: Color::srgba(1.0, 0.0, 0.0, 1.0),
            stroke_width: 2.0,
        }
    }
}

fn setup_click_indicator(mut commands: Commands) {
    commands.insert_resource(ClickIndicatorConfig::default());
}

fn manage_click_indicator(
    mut commands: Commands,
    mouse_state: Res<MouseState>,
    indicator_config: Res<ClickIndicatorConfig>,
    indicator_query: Query<Entity, With<ClickIndicator>>,
    time: Res<Time>,
) {
    if !mouse_state.dragging() {
        // Remove the indicator when mouse is not dragging
        for entity in indicator_query.iter() {
            commands.entity(entity).despawn();
        }
        return;
    }

    let elapsed =
        time.elapsed_seconds() - mouse_state.start_time.unwrap_or(0.0);
    if elapsed < mouse_state.long_click_threshold / 5.0 {
        return; // not pressed long enough to show indicator
    }
    let progress = (elapsed / mouse_state.long_click_threshold).min(1.0);
    let position = mouse_state.current_position;

    if indicator_query.iter().count() == 0 {
        // Create the indicator
        let shape = shapes::Circle {
            radius: indicator_config.radius,
            center: Vec2::ZERO,
        };
        commands.spawn((
            ShapeBundle {
                path: GeometryBuilder::build_as(&shape),
                spatial: SpatialBundle {
                    transform: Transform::from_xyz(
                        position.x, position.y, 100.0,
                    ),
                    ..default()
                },
                ..default()
            },
            Fill::color(Color::NONE),
            Stroke::new(indicator_config.color, indicator_config.stroke_width),
            ClickIndicator {},
        ));
    } else {
        // Update the indicator
        for entity in indicator_query.iter() {
            // Update position
            commands
                .entity(entity)
                .insert(Transform::from_xyz(position.x, position.y, 100.0));
            // Update color
            if progress >= 1.0 {
                commands
                    .entity(entity)
                    .insert(Fill::color(indicator_config.long_color));
            } else {
                commands.entity(entity).insert(Fill::color(
                    indicator_config.color.with_alpha(progress),
                ));
            }
        }
    }
}

pub struct ClickIndicatorPlugin;

impl Plugin for ClickIndicatorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ClickIndicatorConfig>()
            .add_systems(Startup, setup_click_indicator)
            .add_systems(Update, manage_click_indicator);
    }
}

#[derive(Component)]
pub struct HoverText {
    pub text: String,
    pub text_entity: Option<Entity>,
}

impl HoverText {
    pub fn new(text: String) -> Self {
        Self {
            text,
            text_entity: None,
        }
    }
}

pub fn update_hover_text(
    mut commands: Commands,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    window_query: Query<&Window>,
    mut hover_text_query: Query<(Entity, &mut HoverText, &GlobalTransform)>,
) {
    let mouse_position = match get_mouse_position(&camera_query, &window_query)
    {
        Some(pos) => pos,
        None => return,
    };

    for (entity, mut hover_text, transform) in hover_text_query.iter_mut() {
        let is_hovering = transform
            .compute_transform()
            .translation
            .truncate()
            .distance(mouse_position)
            < 20.0;

        match (is_hovering, hover_text.text_entity) {
            (true, None) => {
                // Spawn text entity when starting to hover
                let text_entity = commands
                    .spawn(Text2dBundle {
                        text: Text::from_section(
                            hover_text.text.clone(),
                            TextStyle {
                                font_size: 20.0,
                                color: Color::BLACK,
                                ..default()
                            },
                        ),
                        transform: Transform::from_xyz(0.0, 30.0, 2.0),
                        ..default()
                    })
                    .id();
                commands.entity(entity).add_child(text_entity);
                hover_text.text_entity = Some(text_entity);
            }
            (false, Some(text_entity)) => {
                // Remove text entity when no longer hovering
                commands.entity(text_entity).despawn();
                hover_text.text_entity = None;
            }
            _ => {}
        }
    }
}
