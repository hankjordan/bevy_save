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
    /// Builder::new::<Snapshot>(world)
    ///     // Exclude `Transform` from this `Snapshot`
    ///     .filter(|reg| reg.type_info().type_path() != "bevy_transform::components::transform::Transform")
    /// 
    ///     // Extract all matching entities and resources
    ///     .extract_all()
    ///     
    ///     // Clear all extracted entities without any components
    ///     .clear_empty()
    ///     
    ///     // Build the `Snapshot`
    ///     .build();
    /// ```
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
pub trait Build: Sized {
    /// The snapshot being built.
    type Output;

    /// Extract all entities and resources from the builder's [`World`].
    fn extract_all(self) -> Self {
        self.extract_all_entities().extract_all_resources()
    }

    /// Extract a single entity from the builder's [`World`].
    fn extract_entity(self, entity: Entity) -> Self {
        self.extract_entities([entity].into_iter())
    }

    /// Extract entities from the builder's [`World`].
    fn extract_entities(self, entities: impl Iterator<Item = Entity>) -> Self;

    /// Extract all entities from the builder's [`World`].
    fn extract_all_entities(self) -> Self;

    /// Extract a single resource with the given type name from the builder's [`World`].
    fn extract_resource<S: Into<String>>(self, resource: S) -> Self {
        self.extract_resources([resource].into_iter())
    }

    /// Extract resources with the given type names from the builder's [`World`].
    fn extract_resources<S: Into<String>>(self, resources: impl Iterator<Item = S>) -> Self;

    /// Extract all resources from the builder's [`World`].
    fn extract_all_resources(self) -> Self;

    /// Clear all extracted entities and resources.
    fn clear(self) -> Self {
        self.clear_entities().clear_resources()
    }

    /// Clear all extracted entities.
    fn clear_entities(self) -> Self;

    /// Clear all extracted resources.
    fn clear_resources(self) -> Self;

    /// Clear all entities without any components.
    fn clear_empty(self) -> Self;

    /// Build the extracted resources into a snapshot.
    fn build(self) -> Self::Output;
}
