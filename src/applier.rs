use std::{any::TypeId, marker::PhantomData};

use bevy::{
    ecs::{
        entity::EntityHashMap,
        query::QueryFilter,
        reflect::ReflectMapEntities,
        system::EntityCommands,
        world::{CommandQueue, EntityRef},
    },
    prelude::*,
    scene::SceneSpawnError,
    utils::HashMap,
};

use crate::{Error, Snapshot};

/// A [`Hook`] runs on each entity when applying a snapshot.
///
/// # Example
/// This could be used to apply entities as children of another entity.
/// ```
/// # use bevy::prelude::*;
/// # use bevy_save::prelude::*;
/// # let mut app = App::new();
/// # app.add_plugins(MinimalPlugins);
/// # app.add_plugins(SavePlugins);
/// # let world = app.world_mut();
/// # let snapshot = Snapshot::from_world(world);
/// # let parent = world.spawn_empty().id();
/// snapshot
///     .applier(world)
///     .hook(move |entity, cmds| {
///         if !entity.contains::<Parent>() {
///             cmds.set_parent(parent);
///         }
///     })
///     .apply();
/// ```
pub trait Hook: for<'a> Fn(&'a EntityRef, &'a mut EntityCommands) + Send + Sync {}

impl<T> Hook for T where T: for<'a> Fn(&'a EntityRef, &'a mut EntityCommands) + Send + Sync {}

/// A boxed [`Hook`].
pub type BoxedHook = Box<dyn Hook>;

/// [`SnapshotApplier`] lets you configure how a snapshot will be applied to the [`World`].
pub struct SnapshotApplier<'a, F = ()> {
    snapshot: &'a Snapshot,
    world: &'a mut World,
    entity_map: Option<&'a mut EntityHashMap<Entity>>,
    type_registry: Option<&'a AppTypeRegistry>,
    despawn: Option<PhantomData<F>>,
    hook: Option<BoxedHook>,
}

impl<'a> SnapshotApplier<'a> {
    /// Create a new [`SnapshotApplier`] with from the world and snapshot.
    pub fn new(snapshot: &'a Snapshot, world: &'a mut World) -> Self {
        Self {
            snapshot,
            world,
            entity_map: None,
            type_registry: None,
            despawn: None,
            hook: None,
        }
    }
}

impl<'a, A> SnapshotApplier<'a, A> {
    /// Providing an entity map allows you to map ids of spawned entities and see what entities have been spawned.
    pub fn entity_map(mut self, entity_map: &'a mut EntityHashMap<Entity>) -> Self {
        self.entity_map = Some(entity_map);
        self
    }

    /// The [`AppTypeRegistry`] used for reflection information.
    ///
    /// If this is not provided, the [`AppTypeRegistry`] resource is used as a default.
    pub fn type_registry(mut self, type_registry: &'a AppTypeRegistry) -> Self {
        self.type_registry = Some(type_registry);
        self
    }

    /// Change how the snapshot affects existing entities while applying.
    pub fn despawn<F: QueryFilter + 'static>(self) -> SnapshotApplier<'a, F> {
        SnapshotApplier {
            snapshot: self.snapshot,
            world: self.world,
            entity_map: self.entity_map,
            type_registry: self.type_registry,
            despawn: Some(PhantomData),
            hook: self.hook,
        }
    }

    /// Add a [`Hook`] that will run for each entity after applying.
    pub fn hook<F: Hook + 'static>(mut self, hook: F) -> Self {
        self.hook = Some(Box::new(hook));
        self
    }
}

impl<'a, F: QueryFilter> SnapshotApplier<'a, F> {
    /// Apply the [`Snapshot`] to the [`World`].
    ///
    /// # Panics
    /// If `type_registry` is not set or the [`AppTypeRegistry`] resource does not exist.
    ///
    /// # Errors
    /// If a type included in the [`Snapshot`] has not been registered with the type registry.
    pub fn apply(self) -> Result<(), Error> {
        let default_type_registry = self.world.get_resource::<AppTypeRegistry>().cloned();

        let type_registry = self
            .type_registry
            .or(default_type_registry.as_ref())
            .expect("Must set `type_registry` or insert `AppTypeRegistry` resource to apply.")
            .read();

        let mut default_entity_map = EntityHashMap::default();

        let entity_map = self.entity_map.unwrap_or(&mut default_entity_map);

        for resource in &self.snapshot.resources {
            let type_info = resource.get_represented_type_info().ok_or_else(|| {
                SceneSpawnError::NoRepresentedType {
                    type_path: resource.reflect_type_path().to_string(),
                }
            })?;
            let registration = type_registry.get(type_info.type_id()).ok_or_else(|| {
                SceneSpawnError::UnregisteredButReflectedType {
                    type_path: type_info.type_path().to_string(),
                }
            })?;
            let reflect_resource = registration.data::<ReflectResource>().ok_or_else(|| {
                SceneSpawnError::UnregisteredResource {
                    type_path: type_info.type_path().to_string(),
                }
            })?;

            // If the world already contains an instance of the given resource
            // just apply the (possibly) new value, otherwise insert the resource
            reflect_resource.apply_or_insert(self.world, &**resource, &type_registry);
        }

        // Despawn entities
        if self.despawn.is_some() {
            let invalid = self
                .world
                .query_filtered::<Entity, F>()
                .iter(self.world)
                .collect::<Vec<_>>();

            for entity in invalid {
                self.world.despawn(entity);
            }
        }

        // For each component types that reference other entities, we keep track
        // of which entities in the scene use that component.
        // This is so we can update the scene-internal references to references
        // of the actual entities in the world.
        let mut scene_mappings: HashMap<TypeId, Vec<Entity>> = HashMap::default();

        for scene_entity in &self.snapshot.entities {
            // Fetch the entity with the given entity id from the `entity_map`
            // or spawn a new entity with a transiently unique id if there is
            // no corresponding entry.
            let entity = *entity_map
                .entry(scene_entity.entity)
                .or_insert_with(|| self.world.spawn_empty().id());

            let entity_mut = &mut self.world.entity_mut(entity);

            // Apply/ add each component to the given entity.
            for component in &scene_entity.components {
                let type_info = component.get_represented_type_info().ok_or_else(|| {
                    SceneSpawnError::NoRepresentedType {
                        type_path: component.reflect_type_path().to_string(),
                    }
                })?;
                let registration = type_registry.get(type_info.type_id()).ok_or_else(|| {
                    SceneSpawnError::UnregisteredButReflectedType {
                        type_path: type_info.type_path().to_string(),
                    }
                })?;
                let reflect_component =
                    registration.data::<ReflectComponent>().ok_or_else(|| {
                        SceneSpawnError::UnregisteredComponent {
                            type_path: type_info.type_path().to_string(),
                        }
                    })?;

                // If this component references entities in the scene, track it
                // so we can update it to the entity in the world.
                if registration.data::<ReflectMapEntities>().is_some() {
                    scene_mappings
                        .entry(registration.type_id())
                        .or_insert(Vec::new())
                        .push(entity);
                }

                // If the entity already has the given component attached,
                // just apply the (possibly) new value, otherwise add the
                // component to the entity.
                reflect_component.insert(entity_mut, &**component, &type_registry);
            }
        }

        // Updates references to entities in the scene to entities in the world
        for (type_id, entities) in scene_mappings {
            let registration = type_registry.get(type_id).expect(
                "we should be getting TypeId from this TypeRegistration in the first place",
            );
            if let Some(map_entities_reflect) = registration.data::<ReflectMapEntities>() {
                map_entities_reflect.map_entities(self.world, entity_map, &entities);
            }
        }

        // Entity hook
        if let Some(hook) = &self.hook {
            let mut queue = CommandQueue::default();
            let mut commands = Commands::new(&mut queue, self.world);

            for (_, entity) in entity_map {
                let entity_ref = self.world.entity(*entity);
                let mut entity_mut = commands.entity(*entity);

                hook(&entity_ref, &mut entity_mut);
            }

            queue.apply(self.world);
        }

        Ok(())
    }
}
