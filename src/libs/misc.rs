use bevy::prelude::*;

pub fn mark_component_changed<T: Component>(
    commands: &mut Commands,
    entity: Entity,
) {
    commands.add(move |world: &mut World| {
        if let Some(mut comp) = world.get_entity_mut(entity) {
            if let Some(mut comp) = comp.get_mut::<T>() {
                comp.set_changed();
            }
        }
    });
}
