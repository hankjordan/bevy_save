use std::{
    any::{
        Any,
        TypeId,
    },
    collections::BTreeMap,
};

use bevy::{
    ecs::component::ComponentId,
    prelude::*,
    reflect::TypeRegistry,
    scene::DynamicEntity,
};

#[cfg(feature = "checkpoints")]
use crate::reflect::checkpoint::CheckpointRegistry;
use crate::{
    clone_reflect_value,
    prelude::*,
};

/// A snapshot builder that can extract entities and resources from a [`World`].
pub struct Builder {
    entities: BTreeMap<Entity, DynamicEntity>,
    resources: BTreeMap<ComponentId, Box<dyn PartialReflect>>,
    filter: SceneFilter,
    #[cfg(feature = "checkpoints")]
    is_checkpoint: bool,
}

impl Builder {
    /// Creates input used to build a snapshot.
    #[expect(clippy::new_without_default)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            entities: BTreeMap::new(),
            resources: BTreeMap::new(),
            filter: SceneFilter::default(),
            #[cfg(feature = "checkpoints")]
            is_checkpoint: false,
        }
    }

    /// Creates input used to build a checkpoint.
    #[cfg(feature = "checkpoints")]
    #[must_use]
    pub fn checkpoint() -> Self {
        Self::new().into_checkpoint()
    }
}

#[cfg(feature = "checkpoints")]
impl Builder {
    /// Turns this builder into a checkpoint builder.
    #[must_use]
    pub fn into_checkpoint(mut self) -> Self {
        self.is_checkpoint = true;
        self
    }
}

impl Builder {
    /// Creates a temporary, scoped builder from the input.
    #[must_use]
    pub fn scope(self, world: &World, scope: impl Fn(BuilderRef) -> BuilderRef) -> Self {
        scope(BuilderRef::from_parts(world, self)).input
    }
}

impl Builder {
    /// Build the extracted entities and resources into a [`Snapshot`].
    pub fn build(self) -> Snapshot {
        Snapshot {
            entities: self.entities.into_values().collect(),
            resources: self.resources.into_values().collect(),
        }
    }
}

/// A snapshot builder that can extract entities and resources from a [`World`].
pub struct BuilderRef<'a> {
    world: &'a World,
    type_registry: Option<&'a TypeRegistry>,
    input: Builder,
}

impl<'a> BuilderRef<'a> {
    /// Create a new [`Builder`] from the [`World`].
    ///
    /// You must call at least one of the `extract` methods or the built snapshot will be empty.
    ///
    /// # Example
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_save::prelude::*;
    /// # let mut app = App::new();
    /// # app.add_plugins(MinimalPlugins);
    /// # app.add_plugins(SavePlugins);
    /// # let world = app.world_mut();
    /// BuilderRef::new(world)
    ///     // Extract all matching entities and resources
    ///     .extract_all()
    ///
    ///     // Clear all extracted entities without any components
    ///     .clear_empty()
    ///
    ///     // Build the `Snapshot`
    ///     .build();
    /// ```
    #[must_use]
    pub fn new(world: &'a World) -> Self {
        Self {
            world,
            type_registry: None,
            input: Builder::new(),
        }
    }

    /// Create a new [`BuilderRef`] from the [`World`].
    ///
    /// Types extracted by this builder will respect the [`CheckpointRegistry`](crate::reflect::checkpoint::CheckpointRegistry).
    ///
    /// You must call at least one of the `extract` methods or the built snapshot will be empty.
    ///
    /// # Example
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_save::prelude::*;
    /// # let mut app = App::new();
    /// # app.add_plugins(MinimalPlugins);
    /// # app.add_plugins(SavePlugins);
    /// # let world = app.world_mut();
    /// BuilderRef::checkpoint(world)
    ///     // Extract all matching entities and resources
    ///     .extract_all()
    ///
    ///     // Clear all extracted entities without any components
    ///     .clear_empty()
    ///
    ///     // Build the `Snapshot`
    ///     .build();
    /// ```
    #[cfg(feature = "checkpoints")]
    #[must_use]
    pub fn checkpoint(world: &'a World) -> Self {
        Self {
            world,
            type_registry: None,
            input: Builder::checkpoint(),
        }
    }

    /// Create a new [`BuilderRef`] from the [`World`] and input.
    #[must_use]
    pub fn from_parts(world: &'a World, input: Builder) -> Self {
        Self {
            world,
            type_registry: None,
            input,
        }
    }
}

impl<'a> BuilderRef<'a> {
    /// Retrieve the builder's reference to the [`World`].
    #[must_use]
    pub fn world<'w>(&self) -> &'w World
    where
        'a: 'w,
    {
        self.world
    }

    /// Set the [`TypeRegistry`] to be used for reflection.
    ///
    /// If this is not provided, the [`AppTypeRegistry`] resource is used as a default.
    #[must_use]
    pub fn type_registry(mut self, type_registry: &'a TypeRegistry) -> Self {
        self.type_registry = Some(type_registry);
        self
    }
}

impl BuilderRef<'_> {
    /// Specify a custom [`SceneFilter`] to be used with this builder.
    ///
    /// This filter is applied to both components and resources.
    #[must_use]
    pub fn filter(mut self, filter: SceneFilter) -> Self {
        self.input.filter = filter;
        self
    }

    /// Allows the given type, `T`, to be included in the generated snapshot.
    ///
    /// This method may be called multiple times for any number of types.
    ///
    /// This is the inverse of [`deny`](Self::deny).
    /// If `T` has already been denied, then it will be removed from the blacklist.
    #[must_use]
    pub fn allow<T: Any>(mut self) -> Self {
        self.input.filter = self.input.filter.allow::<T>();
        self
    }

    /// Denies the given type, `T`, from being included in the generated snapshot.
    ///
    /// This method may be called multiple times for any number of types.
    ///
    /// This is the inverse of [`allow`](Self::allow).
    /// If `T` has already been allowed, then it will be removed from the whitelist.
    #[must_use]
    pub fn deny<T: Any>(mut self) -> Self {
        self.input.filter = self.input.filter.deny::<T>();
        self
    }

    /// Updates the filter to allow all types.
    ///
    /// This is useful for resetting the filter so that types may be selectively [denied].
    ///
    /// [denied]: Self::deny
    #[must_use]
    pub fn allow_all(mut self) -> Self {
        self.input.filter = SceneFilter::allow_all();
        self
    }

    /// Updates the filter to deny all types.
    ///
    /// This is useful for resetting the filter so that types may be selectively [allowed].
    ///
    /// [allowed]: Self::allow
    #[must_use]
    pub fn deny_all(mut self) -> Self {
        self.input.filter = SceneFilter::deny_all();
        self
    }
}

impl BuilderRef<'_> {
    /// Extract a single entity from the builder’s [`World`].
    #[must_use]
    pub fn extract_entity(self, entity: Entity) -> Self {
        self.extract_entities([entity].into_iter())
    }

    /// Extract the given entities from the builder’s [`World`].
    ///
    /// # Panics
    /// If `type_registry` is not set or the [`AppTypeRegistry`] resource does not exist.
    #[must_use]
    pub fn extract_entities(mut self, entities: impl Iterator<Item = Entity>) -> Self {
        let app_type_registry = self
            .world
            .get_resource::<AppTypeRegistry>()
            .map(|r| r.read());

        let type_registry = self
            .type_registry
            .or(app_type_registry.as_deref())
            .expect("Must set `type_registry` or insert `AppTypeRegistry` resource to extract.");

        #[cfg(feature = "checkpoints")]
        let checkpoints = self.world.get_resource::<CheckpointRegistry>();

        for entity in entities.filter_map(|e| self.world.get_entity(e).ok()) {
            let id = entity.id();
            let mut entry = DynamicEntity {
                entity: id,
                components: Vec::new(),
            };

            for component in entity.archetype().components() {
                let reflect = self
                    .world
                    .components()
                    .get_info(component)
                    .and_then(|info| info.type_id())
                    .filter(|id| self.input.filter.is_allowed_by_id(*id));

                #[cfg(feature = "checkpoints")]
                let reflect = reflect.filter(|id| {
                    if self.input.is_checkpoint {
                        checkpoints.is_none_or(|rb| rb.is_allowed_by_id(*id))
                    } else {
                        true
                    }
                });

                let reflect = reflect.and_then(|id| type_registry.get(id)).and_then(|r| {
                    Some(clone_reflect_value(
                        r.data::<ReflectComponent>()?.reflect(entity)?,
                        type_registry,
                    ))
                });

                if let Some(reflect) = reflect {
                    entry.components.push(reflect);
                }
            }

            self.input.entities.insert(id, entry);
        }

        self
    }

    /// Extract the entities matching the given filter from the builder’s [`World`].
    #[must_use]
    pub fn extract_entities_matching<F: Fn(&EntityRef) -> bool>(self, filter: F) -> Self {
        // TODO: We should be using Query and caching the lookup
        let entities = self.world.iter_entities().filter(filter).map(|e| e.id());
        self.extract_entities(entities)
    }

    /// Extract all entities from the builder’s [`World`].
    #[must_use]
    pub fn extract_all_entities(self) -> Self {
        let entites = self.world.iter_entities().map(|e| e.id());
        self.extract_entities(entites)
    }

    /// Extract all entities with a custom extraction function.
    #[must_use]
    pub fn extract_entities_manual<F, B>(mut self, func: F) -> Self
    where
        F: Fn(&EntityRef) -> Option<Vec<Box<dyn PartialReflect>>>,
    {
        for entity in self.world.iter_entities() {
            let Some(components) = func(&entity) else {
                continue;
            };

            self.input.entities.insert(entity.id(), DynamicEntity {
                entity: entity.id(),
                components,
            });
        }

        self
    }

    /// Extract all [`Prefab`] entities with a custom extraction function.
    #[must_use]
    pub fn extract_prefab<F, P>(mut self, func: F) -> Self
    where
        F: Fn(&EntityRef) -> Option<P>,
        P: Prefab + PartialReflect,
    {
        for entity in self.world.iter_entities() {
            if !entity.contains::<P::Marker>() {
                continue;
            }

            let Some(prefab) = func(&entity) else {
                continue;
            };

            self.input.entities.insert(entity.id(), DynamicEntity {
                entity: entity.id(),
                components: vec![Box::new(prefab).into_partial_reflect()],
            });
        }

        self
    }

    /// Extract all spawned instances of [`Prefab`] from the builder’s [`World`].
    #[must_use]
    pub fn extract_all_prefabs<P: Prefab>(self) -> Self {
        P::extract(self)
    }

    /// Extract a single resource from the builder's [`World`].
    #[must_use]
    pub fn extract_resource<T: Resource>(self) -> Self {
        let type_id = self
            .world
            .components()
            .resource_id::<T>()
            .and_then(|i| self.world.components().get_info(i))
            .and_then(|i| i.type_id())
            .into_iter();

        self.extract_resources_by_type_id(type_id)
    }

    /// Extract a single resource with the given type path from the builder's [`World`].
    #[must_use]
    pub fn extract_resource_by_path<T: AsRef<str>>(self, type_path: T) -> Self {
        self.extract_resources_by_path([type_path].into_iter())
    }

    /// Extract resources with the given type paths from the builder's [`World`].
    ///
    /// # Panics
    /// If `type_registry` is not set or the [`AppTypeRegistry`] resource does not exist.
    #[must_use]
    pub fn extract_resources_by_path<T: AsRef<str>>(
        self,
        type_paths: impl Iterator<Item = T>,
    ) -> Self {
        let app_type_registry = self
            .world
            .get_resource::<AppTypeRegistry>()
            .map(|r| r.read());

        let type_registry = self
            .type_registry
            .or(app_type_registry.as_deref())
            .expect("Must set `type_registry` or insert `AppTypeRegistry` resource to extract.");

        self.extract_resources_by_type_id(
            type_paths
                .filter_map(|p| type_registry.get_with_type_path(p.as_ref()))
                .map(|r| r.type_id()),
        )
    }

    /// Extract resources with the given [`TypeId`]'s from the builder's [`World`].
    ///
    /// # Panics
    /// If `type_registry` is not set or the [`AppTypeRegistry`] resource does not exist.
    #[must_use]
    pub fn extract_resources_by_type_id(mut self, type_ids: impl Iterator<Item = TypeId>) -> Self {
        let app_type_registry = self
            .world
            .get_resource::<AppTypeRegistry>()
            .map(|r| r.read());

        let type_registry = self
            .type_registry
            .or(app_type_registry.as_deref())
            .expect("Must set `type_registry` or insert `AppTypeRegistry` resource to extract.");

        #[cfg(feature = "checkpoints")]
        let checkpoints = self.world.get_resource::<CheckpointRegistry>();

        let f = type_ids
            .filter_map(|id| type_registry.get(id))
            .filter(|r| self.input.filter.is_allowed_by_id((*r).type_id()));

        #[cfg(feature = "checkpoints")]
        let f = f.filter(|r| {
            if self.input.is_checkpoint {
                checkpoints.is_none_or(|rb| rb.is_allowed_by_id((*r).type_id()))
            } else {
                true
            }
        });

        f.filter_map(|r| {
            Some((
                self.world.components().get_resource_id(r.type_id())?,
                clone_reflect_value(
                    r.data::<ReflectResource>()?.reflect(self.world).ok()?,
                    type_registry,
                ),
            ))
        })
        .for_each(|(i, r)| {
            self.input.resources.insert(i, r);
        });

        self
    }

    /// Extract all resources from the builder's [`World`].
    #[must_use]
    pub fn extract_all_resources(self) -> Self {
        let resources = self
            .world
            .storages()
            .resources
            .iter()
            .map(|(id, _)| id)
            .filter_map(move |id| self.world.components().get_info(id))
            .filter_map(|info| info.type_id());

        self.extract_resources_by_type_id(resources)
    }

    /// Extract all entities, and resources from the builder's [`World`].
    #[must_use]
    pub fn extract_all(self) -> Self {
        self.extract_all_entities().extract_all_resources()
    }
}

impl BuilderRef<'_> {
    /// Clear all extracted entities.
    #[must_use]
    pub fn clear_entities(mut self) -> Self {
        self.input.entities.clear();
        self
    }

    /// Clear all extracted resources.
    #[must_use]
    pub fn clear_resources(mut self) -> Self {
        self.input.resources.clear();
        self
    }

    /// Clear all extracted entities without any components.
    #[must_use]
    pub fn clear_empty(mut self) -> Self {
        self.input.entities.retain(|_, e| !e.components.is_empty());
        self
    }

    /// Clear all extracted entities and resources.
    #[must_use]
    pub fn clear(self) -> Self {
        self.clear_entities().clear_resources()
    }
}

impl BuilderRef<'_> {
    /// Build the extracted entities and resources into a [`Snapshot`].
    pub fn build(self) -> Snapshot {
        self.input.build()
    }
}
