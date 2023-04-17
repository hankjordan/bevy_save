use bevy::reflect::Reflect;

use crate::prelude::*;

/// A reflection-powered serializable representation of an entity and its components.
pub struct SaveableEntity {
    /// The transiently unique identifier of a corresponding `Entity`.
    pub entity: u32,

    /// A vector of boxed components that belong to the given entity and
    /// implement the `Reflect` trait.
    pub components: Vec<Box<dyn Reflect>>,
}

impl CloneReflect for SaveableEntity {
    fn clone_value(&self) -> Self {
        Self {
            entity: self.entity,
            components: self.components.clone_value(),
        }
    }
}
