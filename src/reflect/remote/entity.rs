use bevy::{
    ecs::entity::Entity,
    reflect::Reflect,
};

use crate::reflect::ReflectMap;

/// A reflection-powered serializable representation of an entity and its components.
#[derive(Reflect, Debug)]
pub struct DynamicEntity {
    /// The identifier of the entity, unique within a scene (and the world it may have been generated from).
    ///
    /// Components that reference this entity must consistently use this identifier.
    pub entity: Entity,
    /// A vector of boxed components that belong to the given entity and
    /// implement the [`PartialReflect`] trait.
    pub components: ReflectMap,
}

impl From<bevy::scene::DynamicEntity> for DynamicEntity {
    fn from(value: bevy::scene::DynamicEntity) -> Self {
        // SAFETY: DynamicEntity and bevy::scene::DynamicEntity are equivalent
        unsafe { std::mem::transmute(value) }
    }
}

impl From<DynamicEntity> for bevy::scene::DynamicEntity {
    fn from(value: DynamicEntity) -> Self {
        // SAFETY: DynamicEntity and bevy::scene::DynamicEntity are equivalent
        unsafe { std::mem::transmute(value) }
    }
}

impl From<&bevy::scene::DynamicEntity> for &DynamicEntity {
    fn from(value: &bevy::scene::DynamicEntity) -> Self {
        // SAFETY: DynamicEntity and bevy::scene::DynamicEntity are equivalent
        unsafe { &*std::ptr::from_ref(value).cast() }
    }
}

impl From<&DynamicEntity> for &bevy::scene::DynamicEntity {
    fn from(value: &DynamicEntity) -> Self {
        // SAFETY: DynamicEntity and bevy::scene::DynamicEntity are equivalent
        unsafe { &*std::ptr::from_ref(value).cast() }
    }
}

impl From<&mut bevy::scene::DynamicEntity> for &mut DynamicEntity {
    fn from(value: &mut bevy::scene::DynamicEntity) -> Self {
        // SAFETY: DynamicEntity and bevy::scene::DynamicEntity are equivalent
        unsafe { &mut *std::ptr::from_mut(value).cast() }
    }
}

impl From<&mut DynamicEntity> for &mut bevy::scene::DynamicEntity {
    fn from(value: &mut DynamicEntity) -> Self {
        // SAFETY: DynamicEntity and bevy::scene::DynamicEntity are equivalent
        unsafe { &mut *std::ptr::from_mut(value).cast() }
    }
}
