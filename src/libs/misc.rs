use bevy::prelude::*;

pub fn mark_component_changed<T: Component<Mutability = bevy::ecs::component::Mutable>>(
    commands: &mut Commands,
    entity: Entity,
) {
    commands.queue(move |world: &mut World| {
        if let Ok(mut comp) = world.get_entity_mut(entity) {
            if let Some(mut comp) = comp.get_mut::<T>() {
                comp.set_changed();
            }
        }
    });
}
