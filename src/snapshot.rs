use bevy::{
    ecs::entity::EntityMap,
    prelude::*,
};

use crate::{
    reflect::CloneReflect,
    Rollbacks,
    SaveableRegistry,
};

pub(crate) struct Capture {
    pub(crate) resources: Vec<Box<dyn Reflect>>,
    pub(crate) scene: DynamicScene,
}

impl Capture {
    pub fn apply(&self, world: &mut World) {
        world.clear_trackers();

        let mut map = EntityMap::default();

        for entity in &self.scene.entities {
            let entity = Entity::from_raw(entity.entity);
            map.insert(entity, entity);
            world.get_or_spawn(entity);
        }

        let invalid = world
            .query_filtered::<Entity, Without<Window>>()
            .iter(world)
            .filter(|e| map.get(*e).is_err())
            .collect::<Vec<_>>();

        for entity in invalid {
            world.despawn(entity);
        }

        let _s = self.scene.write_to_world(world, &mut map);

        let registry_arc = world.resource::<AppTypeRegistry>().clone();
        let registry = registry_arc.read();

        for reflect in &self.resources {
            if let Some(reg) = registry.get_with_name(reflect.type_name()) {
                if let Some(res) = reg.data::<ReflectResource>() {
                    res.insert(world, reflect.as_reflect());
                }
            }
        }
    }
}

impl Clone for Capture {
    fn clone(&self) -> Self {
        Self {
            resources: self.resources.clone_value(),
            scene: self.scene.clone_value(),
        }
    }
}

/// A complete snapshot of the game state.
///
/// Can be serialized via [`crate::SnapshotSerializer`] and deserialized via [`crate::SnapshotDeserializer`].
pub struct Snapshot {
    pub(crate) capture: Capture,
    pub(crate) rollbacks: Rollbacks,
}

impl Snapshot {
    /// Retains only the Resources specified by the predicate.
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&Box<dyn Reflect>) -> bool,
    {
        self.capture.resources.retain(f);
    }

    /// Apply the [`Snapshot`] to the [`World`], restoring it to the saved state.
    pub fn apply(&self, world: &mut World) {
        self.capture.apply(world);
        world.insert_resource(self.rollbacks.clone());
    }

    /// Convert the [`Snapshot`] into a [`RollbackSnapshot`] following rollback rules.
    pub fn into_rollback(mut self, world: &mut World) -> RollbackSnapshot {
        let saveables = world.resource::<SaveableRegistry>();

        self.retain(|reg| saveables.can_rollback(reg.type_name()));

        RollbackSnapshot {
            capture: self.capture,
        }
    }
}

/// A rollback snapshot of the game state.
///
/// [`RollbackSnapshot`] excludes resources that opt out of rollback, including the [`Rollbacks`] resource.
#[derive(Clone)]
pub struct RollbackSnapshot {
    pub(crate) capture: Capture,
}

impl RollbackSnapshot {
    /// Apply the [`RollbackSnapshot`] to the [`World`].
    pub fn rollback(&self, world: &mut World) {
        self.capture.apply(world);
    }
}
