use std::collections::HashSet;

use bevy::{
    ecs::{
        entity::EntityMap,
        reflect::ReflectMapEntities,
    },
    prelude::*,
    reflect::TypeRegistration,
};

use crate::prelude::*;

pub(crate) struct RawSnapshot {
    pub(crate) resources: Vec<Box<dyn Reflect>>,
    pub(crate) entities: Vec<SaveableEntity>,
}

impl RawSnapshot {
    fn from_world_with_filter<F>(world: &World, filter: F) -> Self
    where
        F: Fn(&&TypeRegistration) -> bool,
    {
        let registry_arc = world.resource::<AppTypeRegistry>();
        let registry = registry_arc.read();

        let saveables = world.resource::<SaveableRegistry>();

        // Resources

        let resources = saveables
            .types()
            .filter_map(|name| registry.get_with_name(name))
            .filter(&filter)
            .filter_map(|reg| reg.data::<ReflectResource>())
            .filter_map(|res| res.reflect(world))
            .map(|reflect| reflect.clone_value())
            .collect::<Vec<_>>();

        // Entities

        let mut entities = Vec::new();

        for entity in world.iter_entities().map(|entity| entity.id()) {
            let mut entry = SaveableEntity {
                entity: entity.index(),
                components: Vec::new(),
            };

            let entity = world.entity(entity);

            for component_id in entity.archetype().components() {
                let reflect = world
                    .components()
                    .get_info(component_id)
                    .filter(|info| saveables.contains(info.name()))
                    .and_then(|info| info.type_id())
                    .and_then(|id| registry.get(id))
                    .filter(&filter)
                    .and_then(|reg| reg.data::<ReflectComponent>())
                    .and_then(|reflect| reflect.reflect(entity));

                if let Some(reflect) = reflect {
                    entry.components.push(reflect.clone_value());
                }
            }

            entities.push(entry);
        }

        Self {
            resources,
            entities,
        }
    }

    fn apply_with_map(&self, world: &mut World, map: &mut EntityMap) -> Result<(), SaveableError> {
        let registry_arc = world.resource::<AppTypeRegistry>().clone();
        let registry = registry_arc.read();

        // Resources

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

            if let Some(mapper) = reg.data::<ReflectMapEntities>() {
                mapper
                    .map_entities(world, map)
                    .map_err(SaveableError::MapEntitiesError)?;
            }
        }

        // Entities

        // Apply the EntityMap to the saved entities
        let valid = self
            .entities
            .iter()
            .filter_map(|e| e.map(map))
            .collect::<HashSet<_>>();

        // Despawn any entities not contained in the mapped set
        let invalid = world
            .iter_entities()
            .map(|e| e.id())
            .filter(|e| !valid.contains(e))
            .collect::<Vec<_>>();

        for entity in invalid {
            world.despawn(entity);
        }

        // Apply snapshot entities
        for scene_entity in &self.entities {
            let entity = scene_entity.map(map).unwrap_or(world.spawn_empty().id());
            let entity_mut = &mut world.entity_mut(entity);

            for component in &scene_entity.components {
                let reg = registry
                    .get_with_name(component.type_name())
                    .ok_or_else(|| SaveableError::UnregisteredType {
                        type_name: component.type_name().to_string(),
                    })?;

                let data = reg.data::<ReflectComponent>().ok_or_else(|| {
                    SaveableError::UnregisteredComponent {
                        type_name: component.type_name().to_string(),
                    }
                })?;

                data.apply_or_insert(entity_mut, &**component);
            }
        }

        for reg in registry.iter() {
            if let Some(mapper) = reg.data::<ReflectMapEntities>() {
                mapper
                    .map_entities(world, map)
                    .map_err(SaveableError::MapEntitiesError)?;
            }
        }

        Ok(())
    }
}

impl CloneReflect for RawSnapshot {
    fn clone_value(&self) -> Self {
        Self {
            resources: self.resources.clone_value(),
            entities: self.entities.iter().map(|e| e.clone_value()).collect(),
        }
    }
}

/// A rollback snapshot of the game state.
///
/// [`Rollback`] excludes types that opt out of rollback.
pub struct Rollback {
    pub(crate) snapshot: RawSnapshot,
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

        let snapshot = RawSnapshot::from_world_with_filter(world, |reg| {
            registry.can_rollback(reg.type_name()) && filter(reg)
        });

        Self { snapshot }
    }

    /// Apply the [`Rollback`] to the [`World`].
    ///
    /// # Errors
    /// - See [`SaveableError`]
    pub fn apply(&self, world: &mut World) -> Result<(), SaveableError> {
        self.apply_with_map(world, &mut world.entity_map())
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
        self.snapshot.apply_with_map(world, map)
    }
}

impl CloneReflect for Rollback {
    fn clone_value(&self) -> Self {
        Self {
            snapshot: self.snapshot.clone_value(),
        }
    }
}

/// A complete snapshot of the game state.
///
/// Can be serialized via [`SnapshotSerializer`] and deserialized via [`SnapshotDeserializer`].
pub struct Snapshot {
    pub(crate) snapshot: RawSnapshot,
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
        let snapshot = RawSnapshot::from_world_with_filter(world, filter);
        let rollbacks = world.resource::<Rollbacks>().clone_value();

        Self {
            snapshot,
            rollbacks,
        }
    }

    /// Apply the [`Snapshot`] to the [`World`], restoring it to the saved state.
    ///
    /// # Errors
    /// - See [`SaveableError`]
    pub fn apply(&self, world: &mut World) -> Result<(), SaveableError> {
        self.apply_with_map(world, &mut world.entity_map())
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
        self.snapshot.apply_with_map(world, map)?;
        world.insert_resource(self.rollbacks.clone_value());
        Ok(())
    }
}

impl CloneReflect for Snapshot {
    fn clone_value(&self) -> Self {
        Self {
            snapshot: self.snapshot.clone_value(),
            rollbacks: self.rollbacks.clone_value(),
        }
    }
}
