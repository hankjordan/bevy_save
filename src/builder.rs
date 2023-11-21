use std::collections::BTreeMap;

use bevy::{
    ecs::component::ComponentId,
    prelude::*,
    scene::DynamicEntity,
};

use crate::{
    CloneReflect,
    Rollbacks,
    Snapshot,
};

/// A snapshot builder that can extract entities, resources, and [`Rollbacks`] from a [`World`].
pub struct SnapshotBuilder<'a> {
    world: &'a World,
    entities: BTreeMap<Entity, DynamicEntity>,
    resources: BTreeMap<ComponentId, Box<dyn Reflect>>,
    rollbacks: Option<Rollbacks>,
}

impl<'a> SnapshotBuilder<'a> {
    /// Create a new [`SnapshotBuilder`] from the [`World`] and snapshot.
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
    /// # let world = &mut app.world;
    /// SnapshotBuilder::new(world)
    ///     // Extract all matching entities and resources
    ///     .extract_all()
    ///     
    ///     // Clear all extracted entities without any components
    ///     .clear_empty()
    ///     
    ///     // Build the `Snapshot`
    ///     .build();
    /// ```
    pub fn new(world: &'a World) -> Self {
        Self {
            world,
            entities: BTreeMap::new(),
            resources: BTreeMap::new(),
            rollbacks: None,
        }
    }
}

impl<'a> SnapshotBuilder<'a> {
    /// Extract a single entity from the builder’s [`World`].
    pub fn extract_entity(self, entity: Entity) -> Self {
        self.extract_entities([entity].into_iter())
    }

    /// Extract the given entities from the builder’s [`World`].
    pub fn extract_entities(mut self, entities: impl Iterator<Item = Entity>) -> Self {
        let registry = self.world.resource::<AppTypeRegistry>().read();

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

    /// Extract all entities from the builder’s [`World`].
    pub fn extract_all_entities(self) -> Self {
        let entites = self.world.iter_entities().map(|e| e.id());
        self.extract_entities(entites)
    }

    /// Extract a single resource with the given type path from the builder's [`World`].
    pub fn extract_resource<T: AsRef<str>>(self, type_path: T) -> Self {
        self.extract_resources([type_path].into_iter())
    }

    /// Extract resources with the given type paths from the builder's [`World`].
    pub fn extract_resources<T: AsRef<str>>(mut self, type_paths: impl Iterator<Item = T>) -> Self {
        let registry = self.world.resource::<AppTypeRegistry>().read();

        type_paths
            .filter_map(|p| registry.get_with_type_path(p.as_ref()))
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

        self.extract_resources(resources)
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

impl<'a> SnapshotBuilder<'a> {
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

impl<'a> SnapshotBuilder<'a> {
    /// Build the extracted entities and resources into a [`Snapshot`].
    pub fn build(self) -> Snapshot {
        Snapshot {
            entities: self.entities.into_values().collect(),
            resources: self.resources.into_values().collect(),
            rollbacks: None,
        }
    }
}
