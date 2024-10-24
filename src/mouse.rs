use bevy::prelude::*;

use crate::area::*;

#[derive(Resource, Default)]
pub struct MouseState {
    pub long_click_threshold: f32,
    pub left_button_press_start: Option<f32>,
    pub start_position: Option<Vec2>,
    pub current_position: Option<Vec2>,
}

impl MouseState {
    pub fn new(long_click_threshold: f32) -> Self {
        Self {
            long_click_threshold,
            left_button_press_start: None,
            start_position: None,
            current_position: None,
        }
    }

    pub fn dragging(&self) -> bool {
        self.start_position.is_some()
    }

    pub fn start_press(&mut self, time: f32, position: Vec2) {
        self.left_button_press_start = Some(time);
        self.start_position = Some(position);
        self.current_position = Some(position);
    }

    pub fn update_position(&mut self, position: Vec2) {
        self.current_position = Some(position);
    }

    pub fn end_press(&mut self, current_time: f32) -> ClickType {
        let start_time = self.left_button_press_start.take();
        self.start_position.take();
        self.current_position.take();
        self.evaluate_click_type(current_time, start_time)
    }

    pub fn get_click_type(&self, current_time: f32) -> ClickType {
        self.evaluate_click_type(current_time, self.left_button_press_start)
    }

    fn evaluate_click_type(
        &self,
        current_time: f32,
        start_time: Option<f32>,
    ) -> ClickType {
        if let Some(start_time) = start_time {
            let duration = current_time - start_time;
            if duration >= self.long_click_threshold {
                ClickType::Long
            } else {
                ClickType::Short
            }
        } else {
            ClickType::Invalid
        }
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
        mouse_state.update_position(position);
    }

    if mouse_button_input.just_pressed(MouseButton::Left) {
        if let Some(click_position) =
            get_mouse_position(&camera_query, &window_query)
        {
            mouse_state.start_press(time.elapsed_seconds(), click_position);
        }
    } else if mouse_button_input.just_released(MouseButton::Left) {
        mouse_state.end_press(time.elapsed_seconds());
    }
}

#[derive(Debug, Copy, Clone, Component)]
pub struct FollowsMouse {
    pub bounds: RectangularArea,
    pub bound_center: Vec2,
    pub entity: Entity,
    pub entity_area: RectangularArea,
    // offset from the center of the entity - usually where the user clicked
    pub click_offset: Vec2,
    pub only_while_dragging: bool,
}

pub fn follow_mouse_update(
    mut commands: Commands,
    mouse_state: Res<MouseState>,
    mut query: Query<(Entity, &FollowsMouse, &mut Transform, &GlobalTransform)>,
) {
    let mouse_position = match mouse_state.current_position {
        Some(position) => position,
        None => return, // shouldn't happen
    };

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
