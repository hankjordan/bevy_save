use bevy::{
    ecs::entity::EntityMap,
    prelude::*,
};

use crate::reflect::CloneReflect;

/// A complete snapshot of the game state.
pub struct Snapshot {
    pub(crate) resources: Vec<Box<dyn Reflect>>,
    pub(crate) scene: DynamicScene,
}

impl Clone for Snapshot {
    fn clone(&self) -> Self {
        Self {
            resources: self.resources.clone_value(),
            scene: self.scene.clone_value(),
        }
    }
}

impl Snapshot {
    /// Apply the `Snapshot` to the `World`, restoring it to the saved state.
    pub fn apply(&self, world: &mut World) {
        world.clear_entities();
        world.clear_trackers();

        let _s = self.scene.write_to_world(world, &mut EntityMap::default());

        let registry_arc = world.resource::<AppTypeRegistry>().clone();
        let registry = registry_arc.read();

        for reflect in &self.resources {
            if let Some(reg) = registry.get(reflect.type_id()) {
                if let Some(res) = reg.data::<ReflectResource>() {
                    res.apply(world, reflect.as_reflect());
                }
            }
        }
    }
}
