use std::{
    collections::{
        BTreeMap,
        HashSet,
    },
};

use bevy::{
    prelude::*,
    reflect::TypeRegistration,
};

use crate::{
    entity::SaveableEntity,
    CloneReflect,
    RawSnapshot,
    Rollback,
    Rollbacks,
    SaveableRegistry,
    Snapshot,
};

/// A snapshot builder that may extract entities and resources from a [`World`].
pub struct Builder<'w, S, F> {
    world: &'w World,
    filter: F,
    entities: BTreeMap<Entity, SaveableEntity>,
    resources: BTreeMap<String, Box<dyn Reflect>>,
    snapshot: Option<S>,
}

/// A snapshot builder that may extract entities and resources from a [`World`].
///
/// Filters extracted components and resources with the given filter.
///
/// Re-extracting an entity or resource that was already extracted will cause the previously extracted data to be overwritten.
pub trait Build {
    /// The snapshot being built.
    type Output;

    /// Extract all entities and resources from the builder's [`World`].
    fn extract_all(&mut self) -> &mut Self {
        self.extract_all_entities().extract_all_resources()
    }

    /// Extract a single entity from the builder's [`World`].
    fn extract_entity(&mut self, entity: Entity) -> &mut Self {
        self.extract_entities([entity].into_iter())
    }

    /// Extract entities from the builder's [`World`].
    fn extract_entities(&mut self, entities: impl Iterator<Item = Entity>) -> &mut Self;

    /// Extract all entities from the builder's [`World`].
    fn extract_all_entities(&mut self) -> &mut Self;

    /// Extract a single resource with the given type name from the builder's [`World`].
    fn extract_resource<S: Into<String>>(&mut self, resource: S) -> &mut Self {
        self.extract_resources([resource].into_iter())
    }

    /// Extract resources with the given type names from the builder's [`World`].
    fn extract_resources<S: Into<String>>(
        &mut self,
        resources: impl Iterator<Item = S>,
    ) -> &mut Self;

    /// Extract all resources from the builder's [`World`].
    fn extract_all_resources(&mut self) -> &mut Self;

    /// Build the extracted resources into a snapshot.
    fn build(self) -> Self::Output;
}

impl<'w, F> Build for Builder<'w, RawSnapshot, F>
where
    F: Fn(&&TypeRegistration) -> bool,
{
    type Output = RawSnapshot;

    fn extract_entities(&mut self, entities: impl Iterator<Item = Entity>) -> &mut Self {
        let registry_arc = self.world.resource::<AppTypeRegistry>();
        let registry = registry_arc.read();

        let saveables = self.world.resource::<SaveableRegistry>();

        for entity in entities {
            let mut entry = SaveableEntity {
                entity: entity.index(),
                components: Vec::new(),
            };

            let entity = self.world.entity(entity);

            for component_id in entity.archetype().components() {
                let reflect = self
                    .world
                    .components()
                    .get_info(component_id)
                    .filter(|info| saveables.contains(info.name()))
                    .and_then(|info| info.type_id())
                    .and_then(|id| registry.get(id))
                    .filter(&self.filter)
                    .and_then(|reg| reg.data::<ReflectComponent>())
                    .and_then(|reflect| reflect.reflect(entity));

                if let Some(reflect) = reflect {
                    entry.components.push(reflect.clone_value());
                }
            }

            self.entities.insert(entity.id(), entry);
        }

        self
    }

    fn extract_all_entities(&mut self) -> &mut Self {
        self.extract_entities(self.world.iter_entities().map(|e| e.id()))
    }

    fn extract_resources<S: Into<String>>(
        &mut self,
        resources: impl Iterator<Item = S>,
    ) -> &mut Self {
        let names = resources.map(|s| s.into()).collect::<HashSet<String>>();

        let mut builder: Builder<RawSnapshot, _> = Builder {
            world: self.world,
            filter: |reg: &&TypeRegistration| names.contains(reg.type_name()) && (self.filter)(reg),
            entities: BTreeMap::default(),
            resources: BTreeMap::default(),
            snapshot: None,
        };

        builder.extract_all_resources();
        self.resources.append(&mut builder.resources);

        self
    }

    fn extract_all_resources(&mut self) -> &mut Self {
        let registry_arc = self.world.resource::<AppTypeRegistry>();
        let registry = registry_arc.read();

        let saveables = self.world.resource::<SaveableRegistry>();

        saveables
            .types()
            .filter_map(|name| Some((name, registry.get_with_name(name)?)))
            .filter(|(_, reg)| (self.filter)(reg))
            .filter_map(|(name, reg)| Some((name, reg.data::<ReflectResource>()?)))
            .filter_map(|(name, res)| Some((name, res.reflect(self.world)?)))
            .map(|(name, reflect)| (name, reflect.clone_value()))
            .for_each(|(name, reflect)| {
                self.resources.insert(name.clone(), reflect);
            });

        self
    }

    fn build(self) -> Self::Output {
        RawSnapshot {
            resources: self.resources.into_values().collect(),
            entities: self.entities.into_values().collect(),
        }
    }
}

impl<'w, F> Build for Builder<'w, Snapshot, F>
where
    F: Fn(&&TypeRegistration) -> bool,
{
    type Output = Snapshot;

    fn extract_entities(&mut self, entities: impl Iterator<Item = Entity>) -> &mut Self {
        let mut builder: Builder<RawSnapshot, _> = Builder {
            world: self.world,
            filter: &self.filter,
            entities: BTreeMap::default(),
            resources: BTreeMap::default(),
            snapshot: None,
        };

        builder.extract_entities(entities);
        self.entities.append(&mut builder.entities);

        self
    }

    fn extract_all_entities(&mut self) -> &mut Self {
        self.extract_entities(self.world.iter_entities().map(|e| e.id()))
    }

    fn extract_resources<S: Into<String>>(
        &mut self,
        resources: impl Iterator<Item = S>,
    ) -> &mut Self {
        let mut builder: Builder<RawSnapshot, _> = Builder {
            world: self.world,
            filter: &self.filter,
            entities: BTreeMap::default(),
            resources: BTreeMap::default(),
            snapshot: None,
        };

        builder.extract_resources(resources);
        self.resources.append(&mut builder.resources);

        if self.snapshot.is_none() {
            self.snapshot = Some(Snapshot::default());
        }

        self.snapshot.as_mut().unwrap().rollbacks =
            self.world.resource::<Rollbacks>().clone_value();

        self
    }

    fn extract_all_resources(&mut self) -> &mut Self {
        let mut builder: Builder<RawSnapshot, _> = Builder {
            world: self.world,
            filter: &self.filter,
            entities: BTreeMap::default(),
            resources: BTreeMap::default(),
            snapshot: None,
        };

        builder.extract_all_resources();
        self.resources.append(&mut builder.resources);

        self
    }

    fn build(self) -> Self::Output {
        todo!()
    }
}

impl<'w, F> Build for Builder<'w, Rollback, F>
where
    F: Fn(&&TypeRegistration) -> bool,
{
    type Output = Rollback;

    fn extract_entities(&mut self, entities: impl Iterator<Item = Entity>) -> &mut Self {
        let registry = self.world.resource::<SaveableRegistry>();

        let mut builder: Builder<RawSnapshot, _> = Builder {
            world: self.world,
            filter: |reg: &&TypeRegistration| {
                registry.can_rollback(reg.type_name()) && (self.filter)(reg)
            },
            entities: BTreeMap::default(),
            resources: BTreeMap::default(),
            snapshot: None,
        };

        builder.extract_entities(entities);
        self.entities.append(&mut builder.entities);

        self
    }

    fn extract_all_entities(&mut self) -> &mut Self {
        self.extract_entities(self.world.iter_entities().map(|e| e.id()))
    }

    fn extract_resources<S: Into<String>>(
        &mut self,
        resources: impl Iterator<Item = S>,
    ) -> &mut Self {
        let registry = self.world.resource::<SaveableRegistry>();

        let mut builder: Builder<RawSnapshot, _> = Builder {
            world: self.world,
            filter: |reg: &&TypeRegistration| {
                registry.can_rollback(reg.type_name()) && (self.filter)(reg)
            },
            entities: BTreeMap::default(),
            resources: BTreeMap::default(),
            snapshot: None,
        };

        builder.extract_resources(resources);
        self.resources.append(&mut builder.resources);

        self
    }

    fn extract_all_resources(&mut self) -> &mut Self {
        let registry = self.world.resource::<SaveableRegistry>();

        let mut builder: Builder<RawSnapshot, _> = Builder {
            world: self.world,
            filter: |reg: &&TypeRegistration| {
                registry.can_rollback(reg.type_name()) && (self.filter)(reg)
            },
            entities: BTreeMap::default(),
            resources: BTreeMap::default(),
            snapshot: None,
        };

        builder.extract_all_resources();
        self.resources.append(&mut builder.resources);

        self
    }

    fn build(self) -> Self::Output {
        Rollback {
            snapshot: RawSnapshot {
                entities: self.entities.into_values().collect(),
                resources: self.resources.into_values().collect(),
            },
        }
    }
}

/// Snapshot trait
pub trait Capture: Sized + Default {
    /// Create a snapshot with all entities and resources from the [`World`].
    fn extract(world: &World) -> Self {
        Self::extract_with_filter(world, |_| true)
    }

    /// Create a snapshot with all entities and resources from the [`World`].
    ///
    /// Filters extracted components and resources with the given filter.
    fn extract_with_filter<F>(world: &World, filter: F) -> Self
    where
        F: Fn(&&TypeRegistration) -> bool,
    {
        let mut snapshot = Self::default();

        snapshot.extract_all_entities_with_filter(world, &filter);
        snapshot.extract_resources_with_filter(world, filter);

        snapshot
    }

    /// Extract a single entity from the [`World`].
    ///
    /// Re-extracting an entity that was already extracted will create a duplicate.
    fn extract_entity(&mut self, world: &World, entity: Entity) -> &mut Self {
        self.extract_entities(world, [entity].into_iter())
    }

    /// Extract a single entity from the [`World`].
    ///
    /// Filters extracted components with the given filter.
    ///
    /// Re-extracting an entity that was already extracted will create a duplicate.
    fn extract_entity_with_filter<F>(
        &mut self,
        world: &World,
        entity: Entity,
        filter: F,
    ) -> &mut Self
    where
        F: Fn(&&TypeRegistration) -> bool,
    {
        self.extract_entities_with_filter(world, [entity].into_iter(), filter)
    }

    /// Extract all entities from the [`World`].
    ///
    /// Re-extracting an entity that was already extracted will create a duplicate.
    fn extract_all_entities(&mut self, world: &World) -> &mut Self {
        self.extract_all_entities_with_filter(world, |_| true)
    }

    /// Extract all entities from the [`World`].
    ///
    /// Filters extracted components with the given filter.
    ///
    /// Re-extracting an entity that was already extracted will create a duplicate.
    fn extract_all_entities_with_filter<F>(&mut self, world: &World, filter: F) -> &mut Self
    where
        F: Fn(&&TypeRegistration) -> bool,
    {
        self.extract_entities_with_filter(world, world.iter_entities().map(|e| e.id()), &filter)
    }

    /// Extract entities from the [`World`].
    ///
    /// Re-extracting an entity that was already extracted will create a duplicate.
    fn extract_entities(
        &mut self,
        world: &World,
        entities: impl Iterator<Item = Entity>,
    ) -> &mut Self {
        self.extract_entities_with_filter(world, entities, |_| true)
    }

    /// Extract entities from the [`World`].
    ///
    /// Filters extracted components with the given filter.
    ///
    /// Re-extracting an entity that was already extracted will create a duplicate.
    fn extract_entities_with_filter<F>(
        &mut self,
        world: &World,
        entities: impl Iterator<Item = Entity>,
        filter: F,
    ) -> &mut Self
    where
        F: Fn(&&TypeRegistration) -> bool;

    /// Extract resources from the [`World`].
    ///
    /// Re-extracting a resource that was already extracted will create a duplicate.
    fn extract_resources(&mut self, world: &World) -> &mut Self {
        self.extract_resources_with_filter(world, |_| true)
    }

    /// Extract resources from the [`World`].
    ///
    /// Filters extracted resources with the given filter.
    ///
    /// Re-extracting a resource that was already extracted will create a duplicate.
    fn extract_resources_with_filter<F>(&mut self, world: &World, filter: F) -> &mut Self
    where
        F: Fn(&&TypeRegistration) -> bool;

    /// Clears all extracted entities and resources.
    fn clear(&mut self) -> &mut Self;

    /// Clears all extracted entities.
    fn clear_entities(&mut self) -> &mut Self;

    /// Clears all extracted resources.
    fn clear_resources(&mut self) -> &mut Self;

    /// Clears all entities that do not have any components.
    fn remove_empty(&mut self) -> &mut Self;
}
