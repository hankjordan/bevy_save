use bevy::{
    ecs::entity::EntityMap,
    prelude::*,
    reflect::TypeRegistration,
};

use crate::prelude::*;

pub(crate) struct RawSnapshot {
    pub(crate) resources: Vec<Box<dyn Reflect>>,
    pub(crate) entities: SaveableScene,
}

impl RawSnapshot {
    fn from_world_with_filter<F>(world: &World, filter: F) -> Self
    where
        F: Fn(&&TypeRegistration) -> bool,
    {
        let saveables = world.resource::<SaveableRegistry>();

        let resources = saveables
            .types()
            .filter(&filter)
            .filter_map(|reg| reg.data::<ReflectResource>())
            .filter_map(|res| res.reflect(world))
            .map(|reflect| reflect.clone_value())
            .collect::<Vec<_>>();

        let entities = SaveableScene::from_world_with_filter(world, filter);

        Self {
            resources,
            entities,
        }
    }

    fn apply_with_map(&self, world: &mut World, map: &mut EntityMap) -> Result<(), SaveableError> {
        let registry_arc = world.resource::<AppTypeRegistry>().clone();
        let registry = registry_arc.read();

        for resource in &self.resources {
            let reg = registry
                .get_with_name(resource.type_name())
                .ok_or_else(|| SaveableError::UnregisteredType {
                    type_name: resource.type_name().to_string(),
                })?;

            let data = reg.data::<ReflectResource>().ok_or_else(|| {
                SaveableError::UnregisteredResource {
                    type_name: resource.type_name().to_string(),
                }
            })?;

            data.insert(world, resource.as_reflect());
        }

        self.entities.apply_with_map(world, map)
    }
}

impl CloneReflect for RawSnapshot {
    fn clone_value(&self) -> Self {
        Self {
            resources: self.resources.clone_value(),
            entities: self.entities.clone_value(),
        }
    }
}

impl CloneReflect for Snapshot {
    fn clone_value(&self) -> Self {
        Self {
            inner: self.inner.clone_value(),
            rollbacks: self.rollbacks.clone_value(),
        }
    }
}

/// A rollback snapshot of the game state.
///
/// [`Rollback`] excludes types that opt out of rollback.
pub struct Rollback {
    pub(crate) inner: RawSnapshot,
}

impl Rollback {
    /// Returns a [`Rollback`] of the current [`World`] state.
    /// This excludes [`Rollbacks`] and any saveable that ignores rollbacking.
    pub fn from_world(world: &World) -> Self {
        Self::from_world_with_filter(world, |_| true)
    }

    /// Returns a [`Rollback`] of the current [`World`] state, filtered by `filter`.
    /// This excludes [`Rollbacks`] and any saveable that ignores rollbacking.
    pub fn from_world_with_filter<F>(world: &World, filter: F) -> Self
    where
        F: Fn(&&TypeRegistration) -> bool,
    {
        let registry = world.resource::<SaveableRegistry>();

        let inner = RawSnapshot::from_world_with_filter(world, |reg| {
            registry.can_rollback(reg.type_name()) && filter(reg)
        });

        Self { inner }
    }

    /// Apply the [`Rollback`] to the [`World`].
    /// 
    /// # Errors
    /// - See [`SaveableError`]
    pub fn apply(&self, world: &mut World) -> Result<(), SaveableError> {
        self.apply_with_map(world, &mut EntityMap::default())
    }

    /// Apply the [`Rollback`] to the [`World`], mapping entities to new ids with the [`EntityMap`].
    /// 
    /// # Errors
    /// - See [`SaveableError`]
    pub fn apply_with_map(
        &self,
        world: &mut World,
        map: &mut EntityMap,
    ) -> Result<(), SaveableError> {
        self.inner.apply_with_map(world, map)
    }
}

impl CloneReflect for Rollback {
    fn clone_value(&self) -> Self {
        Self {
            inner: self.inner.clone_value(),
        }
    }
}

/// A complete snapshot of the game state.
///
/// Can be serialized via [`crate::SnapshotSerializer`] and deserialized via [`crate::SnapshotDeserializer`].
pub struct Snapshot {
    pub(crate) inner: RawSnapshot,
    pub(crate) rollbacks: Rollbacks,
}

impl Snapshot {
    /// Returns a [`Snapshot`] of the current [`World`] state.
    /// Includes [`Rollbacks`].
    pub fn from_world(world: &World) -> Self {
        Self::from_world_with_filter(world, |_| true)
    }

    /// Returns a [`Snapshot`] of the current [`World`] state filtered by `filter`.
    pub fn from_world_with_filter<F>(world: &World, filter: F) -> Self
    where
        F: Fn(&&TypeRegistration) -> bool,
    {
        let inner = RawSnapshot::from_world_with_filter(world, filter);
        let rollbacks = world.resource::<Rollbacks>().clone_value();

        Self { inner, rollbacks }
    }

    /// Apply the [`Snapshot`] to the [`World`], restoring it to the saved state.
    /// 
    /// # Errors
    /// - See [`SaveableError`]
    pub fn apply(&self, world: &mut World) -> Result<(), SaveableError> {
        self.apply_with_map(world, &mut EntityMap::default())
    }

    /// Apply the [`Snapshot`] to the [`World`], restoring it to the saved state, mapping entities to new ids with the [`EntityMap`].
    /// 
    /// # Errors
    /// - See [`SaveableError`]
    pub fn apply_with_map(
        &self,
        world: &mut World,
        map: &mut EntityMap,
    ) -> Result<(), SaveableError> {
        self.inner.apply_with_map(world, map)?;
        world.insert_resource(self.rollbacks.clone_value());
        Ok(())
    }
}
