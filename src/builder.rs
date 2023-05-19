use std::collections::BTreeMap;

use bevy::{
    prelude::*,
    reflect::TypeRegistration,
};

use crate::entity::SaveableEntity;

/// A snapshot builder that may extract entities and resources from a [`World`].
pub struct Builder<'w, S = (), F = fn(&&TypeRegistration) -> bool> {
    pub(crate) world: &'w World,
    pub(crate) filter: F,
    pub(crate) entities: BTreeMap<Entity, SaveableEntity>,
    pub(crate) resources: BTreeMap<String, Box<dyn Reflect>>,
    pub(crate) snapshot: Option<S>,
}

impl<'w> Builder<'w> {
    /// Create a new [`Builder`] from the [`World`] and snapshot.
    pub fn new<S>(world: &'w World) -> Builder<'w, S> {
        Builder {
            world,
            filter: |_| true,
            entities: BTreeMap::default(),
            resources: BTreeMap::default(),
            snapshot: None,
        }
    }
}

impl<'w, S> Builder<'w, S> {
    /// Change the type filter of the builder.
    ///
    /// Only matching types are included in the snapshot.
    pub fn filter<F>(self, filter: F) -> Builder<'w, S, F>
    where
        F: Fn(&&TypeRegistration) -> bool,
    {
        Builder {
            world: self.world,
            filter,
            entities: self.entities,
            resources: self.resources,
            snapshot: self.snapshot,
        }
    }
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
