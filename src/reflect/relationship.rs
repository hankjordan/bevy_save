//! Support for dynamic use of the [`Relationship`] trait

use std::any::TypeId;

use bevy::{
    ecs::relationship::{
        Relationship,
        RelationshipTarget,
    },
    reflect::FromType,
};

/// [`TypeData`](bevy::reflect::TypeData) for [`Relationship`] components
#[derive(Clone)]
pub struct ReflectRelationship {
    target: TypeId,
}

impl ReflectRelationship {
    /// Returns the [`TypeId`] of the associated [`RelationshipTarget`].
    pub fn target(&self) -> TypeId {
        self.target
    }
}

impl<R: Relationship> FromType<R> for ReflectRelationship {
    fn from_type() -> Self {
        Self {
            target: TypeId::of::<R::RelationshipTarget>(),
        }
    }
}

/// [`TypeData`](bevy::reflect::TypeData) for [`RelationshipTarget`] components
#[derive(Clone)]
pub struct ReflectRelationshipTarget {
    relationship: TypeId,
}

impl ReflectRelationshipTarget {
    /// Returns the [`TypeId`] of the associated [`Relationship`].
    pub fn relationship(&self) -> TypeId {
        self.relationship
    }
}

impl<R: RelationshipTarget> FromType<R> for ReflectRelationshipTarget {
    fn from_type() -> Self {
        Self {
            relationship: TypeId::of::<R::Relationship>(),
        }
    }
}
