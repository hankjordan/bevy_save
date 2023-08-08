use bevy::{
    ecs::entity::EntityMap,
    prelude::*,
};

use crate::prelude::*;

/// A reflection-powered serializable representation of an entity and its components.
pub(crate) struct SaveableEntity {
    /// The transiently unique identifier of a corresponding `Entity`.
    pub entity: u32,

    /// A vector of boxed components that belong to the given entity and
    /// implement the `Reflect` trait.
    pub components: Vec<Box<dyn Reflect>>,
}

impl SaveableEntity {
    /// Returns true if there are no saved components for this Entity.
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    /// Attempts to map the stored index with the given [`EntityMap`].
    pub fn map(&self, map: &EntityMap) -> Option<Entity> {
        map.get(Entity::from_raw(self.entity))
    }

    /// Map the stored index with the given [`EntityMap`] or return an Entity with a one-to-one mapping.
    pub fn try_map(&self, map: &EntityMap) -> Entity {
        self.map(map).unwrap_or(Entity::from_raw(self.entity))
    }
}

impl CloneReflect for SaveableEntity {
    fn clone_value(&self) -> Self {
        Self {
            entity: self.entity,
            components: self.components.clone_value(),
        }
    }
}
