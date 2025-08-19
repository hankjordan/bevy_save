use std::any::TypeId;

use bevy::{
    ecs::{
        component::ComponentCloneBehavior,
        entity::{
            EntityHashMap,
            SceneEntityMapper,
        },
        query::QueryFilter,
        reflect::ReflectMapEntities,
        relationship::RelationshipHookMode,
        system::EntityCommands,
        world::{
            CommandQueue,
            EntityRef,
        },
    },
    platform::collections::HashMap,
    prelude::*,
    reflect::TypeRegistry,
    scene::SceneSpawnError,
};

use crate::{
    clone_reflect_value,
    error::Error,
    prelude::*,
    utils::{
        MaybeMut,
        MaybeRef,
    },
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
///         if !entity.contains::<ChildOf>() {
///             cmds.insert(ChildOf(parent));
///         }
///     })
///     .apply();
/// ```
pub trait Hook: for<'a> Fn(&'a EntityRef, &'a mut EntityCommands) + Send + Sync {}

impl<T> Hook for T where T: for<'a> Fn(&'a EntityRef, &'a mut EntityCommands) + Send + Sync {}

/// A boxed [`Hook`].
pub type BoxedHook = Box<dyn Hook>;

type SpawnPrefabFn = fn(Box<dyn PartialReflect>, Entity, &mut World);

/// Input used for applying [`Snapshot`] to the [`World`].
pub struct Applier<'a> {
    pub(crate) snapshot: MaybeRef<'a, Snapshot>,
    entity_map: Option<MaybeMut<'a, EntityHashMap<Entity>>>,
    registry: Option<MaybeRef<'a, TypeRegistry>>,
    despawns: Vec<fn(&mut World)>,
    hooks: Vec<BoxedHook>,
    prefabs: HashMap<TypeId, SpawnPrefabFn>,
}

impl<'a> Applier<'a> {
    /// Create a new [`Applier`] from the given borrowed or owned [`Snapshot`]
    #[must_use]
    pub fn new(snapshot: impl Into<MaybeRef<'a, Snapshot>>) -> Self {
        Self {
            snapshot: snapshot.into(),
            entity_map: None,
            registry: None,
            despawns: Vec::new(),
            hooks: Vec::new(),
            prefabs: HashMap::new(),
        }
    }
}

impl<'i> Applier<'i> {
    /// Creates a temporary, scoped applier from the input.
    #[must_use]
    pub fn scope<'w>(
        self,
        world: &'w mut World,
        scope: impl Fn(ApplierRef<'w, 'i>) -> ApplierRef<'w, 'i>,
    ) -> Self
    where
        'i: 'w,
    {
        scope(ApplierRef::from_parts(world, self)).input
    }
}

/// [`ApplierRef`] lets you configure how a snapshot will be applied to the [`World`].
pub struct ApplierRef<'w, 'i> {
    world: &'w mut World,
    input: Applier<'i>,
}

impl<'w, 'i> ApplierRef<'w, 'i> {
    /// Create a new [`ApplierRef`] from the world and borrowed or owned [`Snapshot`].
    #[must_use]
    pub fn new(snapshot: impl Into<MaybeRef<'i, Snapshot>>, world: &'w mut World) -> Self {
        Self {
            world,
            input: Applier::new(snapshot),
        }
    }

    /// Create a new [`ApplierRef`] from the world and input.
    #[must_use]
    pub fn from_parts(world: &'w mut World, input: Applier<'i>) -> Self {
        Self { world, input }
    }

    /// Reduce the applier into its input
    #[must_use]
    pub fn into_inner(self) -> Applier<'i> {
        self.input
    }
}

impl<'i> ApplierRef<'_, 'i> {
    /// Providing an entity map allows you to map ids of spawned entities and
    /// see what entities have been spawned.
    ///
    /// Most applications will not need to build an entity map - instead,
    /// prefer to [despawn existing entities](Self::despawn).
    #[must_use]
    pub fn entity_map(
        mut self,
        entity_map: impl Into<MaybeMut<'i, EntityHashMap<Entity>>>,
    ) -> Self {
        self.input.entity_map = Some(entity_map.into());
        self
    }

    /// Set the [`TypeRegistry`] to be used for reflection.
    ///
    /// If this is not provided, the [`AppTypeRegistry`] resource is used as a default.
    #[must_use]
    pub fn type_registry(mut self, registry: &'i TypeRegistry) -> Self {
        self.input.registry = Some(registry.into());
        self
    }

    /// Despawn existing entities matching the filter while applying.
    #[must_use]
    pub fn despawn<F: QueryFilter + 'static>(mut self) -> Self {
        self.input.despawns.push(|w| {
            for entity in w.query_filtered::<Entity, F>().iter(w).collect::<Vec<_>>() {
                w.despawn(entity);
            }
        });
        self
    }

    /// Add a [`Hook`] that will run for each entity after applying.
    #[must_use]
    pub fn hook(mut self, hook: impl Hook + 'static) -> Self {
        self.input.hooks.push(Box::new(hook));
        self
    }

    /// Handle loading for a [`Prefab`].
    #[expect(clippy::missing_panics_doc)]
    #[must_use]
    pub fn prefab<P: Prefab + FromReflect>(mut self) -> Self {
        self.input.prefabs.insert(
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

struct MapEntitiesMapper<'m, 'w> {
    map: &'m mut EntityHashMap<Entity>,
    world: &'w mut World,
}

impl<'m, 'w> MapEntitiesMapper<'m, 'w> {
    fn new(map: &'m mut EntityHashMap<Entity>, world: &'w mut World) -> Self {
        Self { map, world }
    }
}

impl EntityMapper for MapEntitiesMapper<'_, '_> {
    fn get_mapped(&mut self, source: Entity) -> Entity {
        *self
            .map
            .entry(source)
            .or_insert_with(|| self.world.spawn_empty().id())
    }

    fn set_mapped(&mut self, source: Entity, target: Entity) {
        self.map.insert(source, target);
    }
}

impl Drop for MapEntitiesMapper<'_, '_> {
    fn drop(&mut self) {
        // WORKAROUND: We've already mapped, don't do it again.
        for mapped in self.map.values().copied().collect::<Vec<_>>() {
            self.map.insert(mapped, mapped);
        }
    }
}

impl ApplierRef<'_, '_> {
    /// Apply the [`Snapshot`] to the [`World`].
    ///
    /// # Panics
    /// If `type_registry` is not set or the [`AppTypeRegistry`] resource does not exist.
    ///
    /// # Errors
    /// If a type included in the [`Snapshot`] has not been registered with the type registry.
    pub fn apply(&mut self) -> Result<(), Error> {
        let app_registry_arc = self.world.get_resource::<AppTypeRegistry>().cloned();

        let app_registry = app_registry_arc.as_ref().map(|r| r.read());

        let registry = self
            .input
            .registry
            .as_deref()
            .or(app_registry.as_deref())
            .expect("Must set `type_registry` or insert `AppTypeRegistry` resource to apply.");

        let entity_map = self.input.entity_map.get_or_insert_default();

        let mut prefab_entities = HashMap::new();

        // Despawn entities
        for despawn in &self.input.despawns {
            despawn(self.world);
        }

        // First ensure that every entity in the snapshot has a corresponding world
        // entity in the entity map.
        for scene_entity in self.input.snapshot.entities() {
            // Fetch the entity with the given entity id from the `entity_map`
            // or spawn a new entity with a transiently unique id if there is
            // no corresponding entry.
            entity_map
                .entry(scene_entity.entity)
                .or_insert_with(|| self.world.spawn_empty().id());
        }

        for scene_entity in self.input.snapshot.entities() {
            // Fetch the entity with the given entity id from the `entity_map`.
            let entity = *entity_map
                .get(&scene_entity.entity)
                .expect("should have previously spawned an empty entity");

            // Apply/ add each component to the given entity.
            for component in &scene_entity.components {
                let type_info = component.get_represented_type_info().ok_or_else(|| {
                    SceneSpawnError::NoRepresentedType {
                        type_path: component.reflect_type_path().to_string(),
                    }
                })?;
                let type_id = type_info.type_id();
                let registration = registry.get(type_id).ok_or_else(|| {
                    SceneSpawnError::UnregisteredButReflectedType {
                        type_path: type_info.type_path().to_string(),
                    }
                })?;

                if registration.contains::<ReflectIgnore>()
                    || registration.contains::<ReflectRelationshipTarget>()
                {
                    continue;
                }

                if self.input.prefabs.contains_key(&type_id) {
                    let mut prefab = clone_reflect_value(&**component, registry);

                    if let Some(map_entities) = registration.data::<ReflectMapEntities>() {
                        map_entities.map_entities(
                            &mut *prefab,
                            &mut MapEntitiesMapper::new(entity_map, self.world),
                        );
                    }

                    prefab_entities
                        .entry(type_id)
                        .or_insert_with(Vec::new)
                        .push((entity, prefab));

                    continue;
                }

                let reflect = registration.data::<ReflectComponent>().ok_or_else(|| {
                    SceneSpawnError::UnregisteredComponent {
                        type_path: type_info.type_path().to_string(),
                    }
                })?;

                {
                    let component_id = reflect.register_component(self.world);
                    // SAFETY: we registered the component above. the info exists
                    let component_info =
                        unsafe { self.world.components().get_info_unchecked(component_id) };
                    if *component_info.clone_behavior() == ComponentCloneBehavior::Ignore {
                        continue;
                    }
                }

                let mut cloned = None;

                // If this component references entities in the scene, update
                // them to the entities in the world.
                let component = registration
                    .data::<ReflectMapEntities>()
                    .and_then(|map_entities| {
                        cloned = Some(clone_reflect_value(&**component, registry));

                        map_entities.map_entities(
                            cloned.as_deref_mut()?,
                            &mut MapEntitiesMapper::new(entity_map, self.world),
                        );

                        cloned.as_deref()
                    })
                    .unwrap_or(&**component);

                SceneEntityMapper::world_scope(entity_map, self.world, |world, mapper| {
                    let entity_mut = &mut world.entity_mut(entity);

                    // WORKAROUND: apply_or_insert doesn't actually apply
                    reflect.remove(entity_mut);

                    reflect.apply_or_insert_mapped(
                        entity_mut,
                        component,
                        registry,
                        mapper,
                        RelationshipHookMode::Run,
                    );
                });
            }
        }

        // Insert resources after all entities have been added to the world.
        // This ensures the entities are available for the resources to reference during
        // mapping.
        for resource in self.input.snapshot.resources() {
            let type_info = resource.get_represented_type_info().ok_or_else(|| {
                SceneSpawnError::NoRepresentedType {
                    type_path: resource.reflect_type_path().to_string(),
                }
            })?;
            let registration = registry.get(type_info.type_id()).ok_or_else(|| {
                SceneSpawnError::UnregisteredButReflectedType {
                    type_path: type_info.type_path().to_string(),
                }
            })?;

            if registration.contains::<ReflectIgnore>()
                || registration.contains::<ReflectRelationshipTarget>()
            {
                continue;
            }

            let reflect = registration.data::<ReflectResource>().ok_or_else(|| {
                SceneSpawnError::UnregisteredResource {
                    type_path: type_info.type_path().to_string(),
                }
            })?;

            let mut cloned = None;

            // If this resource references entities in the scene, update
            // them to the entities in the world.
            let resource = registration
                .data::<ReflectMapEntities>()
                .and_then(|map_entities| {
                    cloned = Some(clone_reflect_value(&**resource, registry));

                    map_entities.map_entities(
                        cloned.as_deref_mut()?,
                        &mut MapEntitiesMapper::new(entity_map, self.world),
                    );

                    cloned.as_deref()
                })
                .unwrap_or(&**resource);

            // If the world already contains an instance of the given resource
            // just apply the (possibly) new value, otherwise insert the resource
            reflect.apply_or_insert(self.world, resource, registry);
        }

        // Prefab hooks
        for (type_id, entities) in prefab_entities {
            let Some(hook) = self.input.prefabs.get(&type_id) else {
                continue;
            };

            for (entity, component) in entities {
                hook(component, entity, self.world);
            }
        }

        // Entity hooks
        if !self.input.hooks.is_empty() {
            let mut queue = CommandQueue::default();
            let mut commands = Commands::new(&mut queue, self.world);

            for hook in &self.input.hooks {
                for (_, entity) in entity_map.iter() {
                    let entity_ref = self.world.entity(*entity);
                    let mut entity_mut = commands.entity(*entity);

                    hook(&entity_ref, &mut entity_mut);
                }
            }

            queue.apply(self.world);
        }

        Ok(())
    }
}
