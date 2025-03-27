use std::{
    any::TypeId,
    marker::PhantomData,
};

use bevy::{
    ecs::{
        entity::{
            EntityHashMap,
            SceneEntityMapper,
        },
        query::QueryFilter,
        reflect::ReflectMapEntities,
        system::EntityCommands,
        world::{
            CommandQueue,
            EntityRef,
        },
    },
    prelude::*,
    reflect::TypeRegistry,
    scene::SceneSpawnError,
    utils::HashMap,
};

use crate::{
    error::Error,
    prelude::*,
};

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

type SpawnPrefabFn = fn(Box<dyn PartialReflect>, Entity, &mut World);

/// [`SnapshotApplier`] lets you configure how a snapshot will be applied to the [`World`].
pub struct SnapshotApplier<'a, F = ()> {
    snapshot: &'a Snapshot,
    world: &'a mut World,
    entity_map: Option<&'a mut EntityHashMap<Entity>>,
    type_registry: Option<&'a TypeRegistry>,
    despawn: Option<PhantomData<F>>,
    hook: Option<BoxedHook>,
    prefabs: HashMap<TypeId, SpawnPrefabFn>,
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
            prefabs: HashMap::new(),
        }
    }
}

impl<'a, A> SnapshotApplier<'a, A> {
    /// Providing an entity map allows you to map ids of spawned entities and see what entities have been spawned.
    pub fn entity_map(mut self, entity_map: &'a mut EntityHashMap<Entity>) -> Self {
        self.entity_map = Some(entity_map);
        self
    }

    /// Set the [`TypeRegistry`] to be used for reflection.
    ///
    /// If this is not provided, the [`AppTypeRegistry`] resource is used as a default.
    pub fn type_registry(mut self, type_registry: &'a TypeRegistry) -> Self {
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
            prefabs: self.prefabs,
        }
    }

    /// Add a [`Hook`] that will run for each entity after applying.
    pub fn hook<F: Hook + 'static>(mut self, hook: F) -> Self {
        self.hook = Some(Box::new(hook));
        self
    }

    /// Handle loading for a [`Prefab`].
    #[allow(clippy::missing_panics_doc)]
    pub fn prefab<P: Prefab + FromReflect>(mut self) -> Self {
        self.prefabs.insert(
            std::any::TypeId::of::<P>(),
            |this: Box<dyn PartialReflect>, target: Entity, world: &mut World| {
                world.entity_mut(target).insert(P::Marker::default());

                P::spawn(
                    <P as FromReflect>::from_reflect(&*this).unwrap(),
                    target,
                    world,
                );
            },
        );
        self
    }
}

impl<F: QueryFilter> SnapshotApplier<'_, F> {
    /// Apply the [`Snapshot`] to the [`World`].
    ///
    /// # Panics
    /// If `type_registry` is not set or the [`AppTypeRegistry`] resource does not exist.
    ///
    /// # Errors
    /// If a type included in the [`Snapshot`] has not been registered with the type registry.
    pub fn apply(self) -> Result<(), Error> {
        let app_type_registry_arc = self.world.get_resource::<AppTypeRegistry>().cloned();

        let app_type_registry = app_type_registry_arc.as_ref().map(|r| r.read());

        let type_registry = self
            .type_registry
            .or(app_type_registry.as_deref())
            .expect("Must set `type_registry` or insert `AppTypeRegistry` resource to apply.");

        let mut default_entity_map = EntityHashMap::default();

        let entity_map = self.entity_map.unwrap_or(&mut default_entity_map);

        let mut prefab_entities = HashMap::new();

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

        // First ensure that every entity in the snapshot has a corresponding world
        // entity in the entity map.
        for scene_entity in &self.snapshot.entities {
            // Fetch the entity with the given entity id from the `entity_map`
            // or spawn a new entity with a transiently unique id if there is
            // no corresponding entry.
            entity_map
                .entry(scene_entity.entity)
                .or_insert_with(|| self.world.spawn_empty().id());
        }

        for scene_entity in &self.snapshot.entities {
            // Fetch the entity with the given entity id from the `entity_map`.
            let entity = *entity_map
                .get(&scene_entity.entity)
                .expect("should have previously spawned an empty entity");

            // Apply/ add each component to the given entity.
            for component in &scene_entity.components {
                let mut component = component.clone_value();
                let type_info = component.get_represented_type_info().ok_or_else(|| {
                    SceneSpawnError::NoRepresentedType {
                        type_path: component.reflect_type_path().to_string(),
                    }
                })?;

                let type_id = type_info.type_id();
                if self.prefabs.contains_key(&type_id) {
                    prefab_entities
                        .entry(type_id)
                        .or_insert_with(Vec::new)
                        .push((entity, component));

                    continue;
                }

                let registration = type_registry.get(type_id).ok_or_else(|| {
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

                if let Some(map_entities) = registration.data::<ReflectMapEntities>() {
                    SceneEntityMapper::world_scope(entity_map, self.world, |_, mapper| {
                        map_entities.map_entities(component.as_partial_reflect_mut(), mapper);
                    });
                }

                // If the entity already has the given component attached,
                // just apply the (possibly) new value, otherwise add the
                // component to the entity.
                reflect_component.insert(
                    &mut self.world.entity_mut(entity),
                    component.as_partial_reflect(),
                    type_registry,
                );
            }
        }

        // Insert resources after all entities have been added to the world.
        // This ensures the entities are available for the resources to reference during mapping.
        for resource in &self.snapshot.resources {
            let mut resource = resource.clone_value();
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

            // If this resource references entities in the scene, update
            // them to the entities in the world.
            if let Some(map_entities) = registration.data::<ReflectMapEntities>() {
                SceneEntityMapper::world_scope(entity_map, self.world, |_, mapper| {
                    map_entities.map_entities(resource.as_partial_reflect_mut(), mapper);
                });
            }

            // If the world already contains an instance of the given resource
            // just apply the (possibly) new value, otherwise insert the resource
            reflect_resource.apply_or_insert(
                self.world,
                resource.as_partial_reflect(),
                type_registry,
            );
        }

        // Prefab hooks
        for (type_id, entities) in prefab_entities {
            let Some(hook) = self.prefabs.get(&type_id) else {
                continue;
            };

            for (entity, component) in entities {
                hook(component, entity, self.world);
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
