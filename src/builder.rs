use std::{
    any::Any,
    collections::BTreeMap,
};

use bevy::{
    ecs::component::ComponentId,
    prelude::*,
    scene::DynamicEntity,
};

use crate::{
    CloneReflect,
    RollbackRegistry,
    Rollbacks,
    Snapshot,
};

/// A snapshot builder that can extract entities, resources, and [`Rollbacks`] from a [`World`].
pub struct SnapshotBuilder<'a> {
    world: &'a World,
    entities: BTreeMap<Entity, DynamicEntity>,
    resources: BTreeMap<ComponentId, Box<dyn Reflect>>,
    filter: SceneFilter,
    rollbacks: Option<Rollbacks>,
    is_rollback: bool,
}

impl<'a> SnapshotBuilder<'a> {
    /// Create a new [`SnapshotBuilder`] from the [`World`].
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
    /// SnapshotBuilder::snapshot(world)
    ///     // Extract all matching entities and resources
    ///     .extract_all()
    ///     
    ///     // Clear all extracted entities without any components
    ///     .clear_empty()
    ///     
    ///     // Build the `Snapshot`
    ///     .build();
    /// ```
    pub fn snapshot(world: &'a World) -> Self {
        Self {
            world,
            entities: BTreeMap::new(),
            resources: BTreeMap::new(),
            filter: SceneFilter::default(),
            rollbacks: None,
            is_rollback: false,
        }
    }

    /// Create a new [`SnapshotBuilder`] from the [`World`].
    ///
    /// Types extracted by this builder will respect the [`RollbackRegistry`](crate::RollbackRegistry).
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
    /// SnapshotBuilder::rollback(world)
    ///     // Extract all matching entities and resources
    ///     .extract_all()
    ///     
    ///     // Clear all extracted entities without any components
    ///     .clear_empty()
    ///     
    ///     // Build the `Snapshot`
    ///     .build();
    /// ```
    pub fn rollback(world: &'a World) -> Self {
        Self {
            world,
            entities: BTreeMap::new(),
            resources: BTreeMap::new(),
            filter: SceneFilter::default(),
            rollbacks: None,
            is_rollback: true,
        }
    }
}

impl<'a> SnapshotBuilder<'a> {
    /// Retrieve the builder's reference to the [`World`].
    pub fn world<'w>(&self) -> &'w World
    where
        'a: 'w,
    {
        self.world
    }
}

impl SnapshotBuilder<'_> {
    /// Specify a custom [`SceneFilter`] to be used with this builder.
    ///
    /// This filter is applied to both components and resources.
    pub fn filter(mut self, filter: SceneFilter) -> Self {
        self.filter = filter;
        self
    }

    /// Allows the given type, `T`, to be included in the generated snapshot.
    ///
    /// This method may be called multiple times for any number of types.
    ///
    /// This is the inverse of [`deny`](Self::deny).
    /// If `T` has already been denied, then it will be removed from the blacklist.
    pub fn allow<T: Any>(mut self) -> Self {
        self.filter = self.filter.allow::<T>();
        self
    }

    /// Denies the given type, `T`, from being included in the generated snapshot.
    ///
    /// This method may be called multiple times for any number of types.
    ///
    /// This is the inverse of [`allow`](Self::allow).
    /// If `T` has already been allowed, then it will be removed from the whitelist.
    pub fn deny<T: Any>(mut self) -> Self {
        self.filter = self.filter.deny::<T>();
        self
    }

    /// Updates the filter to allow all types.
    ///
    /// This is useful for resetting the filter so that types may be selectively [denied].
    ///
    /// [denied]: Self::deny
    pub fn allow_all(mut self) -> Self {
        self.filter = SceneFilter::allow_all();
        self
    }

    /// Updates the filter to deny all types.
    ///
    /// This is useful for resetting the filter so that types may be selectively [allowed].
    ///
    /// [allowed]: Self::allow
    pub fn deny_all(mut self) -> Self {
        self.filter = SceneFilter::deny_all();
        self
    }
}

impl SnapshotBuilder<'_> {
    /// Extract a single entity from the builder’s [`World`].
    pub fn extract_entity(self, entity: Entity) -> Self {
        self.extract_entities([entity].into_iter())
    }

    /// Extract the given entities from the builder’s [`World`].
    pub fn extract_entities(mut self, entities: impl Iterator<Item = Entity>) -> Self {
        let registry = self.world.resource::<AppTypeRegistry>().read();
        let rollbacks = self.world.resource::<RollbackRegistry>();

        for entity in entities.filter_map(|e| self.world.get_entity(e)) {
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
                    .filter(|id| self.filter.is_allowed_by_id(*id))
                    .filter(|id| {
                        if self.is_rollback {
                            rollbacks.is_allowed_by_id(*id)
                        } else {
                            true
                        }
                    })
                    .and_then(|id| registry.get(id))
                    .and_then(|reg| reg.data::<ReflectComponent>())
                    .and_then(|reflect| reflect.reflect(entity));

                if let Some(reflect) = reflect {
                    entry.components.push(reflect.clone_value());
                }
            }

            self.entities.insert(id, entry);
        }

        self
    }

    /// Extract the entities matching the given filter from the builder’s [`World`].
    pub fn extract_entities_matching<F: Fn(&EntityRef) -> bool>(self, filter: F) -> Self {
        let entities = self.world.iter_entities().filter(filter).map(|e| e.id());
        self.extract_entities(entities)
    }

    /// Extract all entities from the builder’s [`World`].
    pub fn extract_all_entities(self) -> Self {
        let entites = self.world.iter_entities().map(|e| e.id());
        self.extract_entities(entites)
    }

    /// Extract a single resource from the builder's [`World`].
    pub fn extract_resource<T: Resource>(self) -> Self {
        let registry = self.world.resource::<AppTypeRegistry>().read();

        let path = self
            .world
            .components()
            .resource_id::<T>()
            .and_then(|i| self.world.components().get_info(i))
            .and_then(|i| i.type_id())
            .and_then(|i| registry.get(i))
            .map(|i| i.type_info().type_path())
            .into_iter();

        self.extract_resources_by_path(path)
    }

    /// Extract a single resource with the given type path from the builder's [`World`].
    pub fn extract_resource_by_path<T: AsRef<str>>(self, type_path: T) -> Self {
        self.extract_resources_by_path([type_path].into_iter())
    }

    /// Extract resources with the given type paths from the builder's [`World`].
    pub fn extract_resources_by_path<T: AsRef<str>>(
        mut self,
        type_paths: impl Iterator<Item = T>,
    ) -> Self {
        let registry = self.world.resource::<AppTypeRegistry>().read();
        let rollbacks = self.world.resource::<RollbackRegistry>();

        type_paths
            .filter_map(|p| registry.get_with_type_path(p.as_ref()))
            .filter(|r| self.filter.is_allowed_by_id((*r).type_id()))
            .filter(|r| {
                if self.is_rollback {
                    rollbacks.is_allowed_by_id((*r).type_id())
                } else {
                    true
                }
            })
            .filter_map(|r| {
                Some((
                    self.world.components().get_resource_id(r.type_id())?,
                    r.data::<ReflectResource>()?
                        .reflect(self.world)?
                        .clone_value(),
                ))
            })
            .for_each(|(i, r)| {
                self.resources.insert(i, r);
            });

        self
    }

    /// Extract all resources from the builder's [`World`].
    pub fn extract_all_resources(self) -> Self {
        let registry = self.world.resource::<AppTypeRegistry>().read();

        let resources = self
            .world
            .storages()
            .resources
            .iter()
            .map(|(id, _)| id)
            .filter_map(move |id| self.world.components().get_info(id))
            .filter_map(|info| info.type_id())
            .filter_map(|id| registry.get(id))
            .map(|reg| reg.type_info().type_path());

        self.extract_resources_by_path(resources)
    }

    /// Extract [`Rollbacks`] from the builder's [`World`].
    pub fn extract_rollbacks(mut self) -> Self {
        self.rollbacks = self
            .world
            .get_resource::<Rollbacks>()
            .map(|r| r.clone_value());

        self
    }

    /// Extract all entities, and resources from the builder's [`World`].
    pub fn extract_all(self) -> Self {
        self.extract_all_entities().extract_all_resources()
    }

    /// Extract all entities, resources, and [`Rollbacks`] from the builder's [`World`].
    pub fn extract_all_with_rollbacks(self) -> Self {
        self.extract_all().extract_rollbacks()
    }
}

impl SnapshotBuilder<'_> {
    /// Clear all extracted entities.
    pub fn clear_entities(mut self) -> Self {
        self.entities.clear();
        self
    }

    /// Clear all extracted resources.
    pub fn clear_resources(mut self) -> Self {
        self.resources.clear();
        self
    }

    /// Clear all extracted entities without any components.
    pub fn clear_empty(mut self) -> Self {
        self.entities.retain(|_, e| !e.components.is_empty());
        self
    }

    /// Clear [`Rollbacks`] from the snapshot.
    pub fn clear_rollbacks(mut self) -> Self {
        self.rollbacks = None;
        self
    }

    /// Clear all extracted entities and resources.
    pub fn clear(self) -> Self {
        self.clear_entities().clear_resources()
    }
}

impl SnapshotBuilder<'_> {
    /// Build the extracted entities and resources into a [`Snapshot`].
    pub fn build(self) -> Snapshot {
        Snapshot {
            entities: self.entities.into_values().collect(),
            resources: self.resources.into_values().collect(),
            rollbacks: self.rollbacks,
        }
    }
}
