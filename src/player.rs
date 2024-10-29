use std::collections::HashMap;

use bevy::ecs::prelude::Component;

use crate::resource::GalaxiaResource;

#[derive(Debug, Default, Component)]
pub struct Player {
    pub resources: HashMap<GalaxiaResource, f32>,
}
