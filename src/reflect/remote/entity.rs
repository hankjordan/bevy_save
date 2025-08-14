use bevy::{
    ecs::entity::Entity,
    reflect::Reflect,
};

use crate::reflect::ReflectMap;

/// A reflection-powered serializable representation of an entity and its components.
#[derive(Reflect, Clone, Debug)]
#[reflect(Clone)]
#[type_path = "bevy_save"]
pub struct DynamicEntity {
    /// The identifier of the entity, unique within a scene
    /// (and the world it may have been generated from).
    ///
    /// Components that reference this entity must consistently use this identifier.
    pub entity: Entity,
    /// A map of boxed components that belong to the given entity and
    /// implement the [`PartialReflect`](bevy::reflect::PartialReflect) trait.
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

#[cfg(test)]
mod test {
    use bevy::prelude::*;

    #[test]
    fn assert_dynamic_entity_matches() {
        let a = bevy::scene::DynamicEntity {
            entity: Entity::PLACEHOLDER,
            components: Vec::new(),
        };

        let b = super::DynamicEntity {
            entity: Entity::PLACEHOLDER,
            components: Vec::new().into(),
        };

        assert_eq!(std::mem::size_of_val(&a), std::mem::size_of_val(&b));
        assert_eq!(
            std::mem::size_of_val(&a.entity),
            std::mem::size_of_val(&b.entity)
        );
        assert_eq!(
            std::mem::size_of_val(&a.components),
            std::mem::size_of_val(&b.components)
        );
    }
}
