use bevy::{
    prelude::*,
    reflect::{
        PartialReflect,
        Reflect,
        TypeRegistry,
        serde::{
            DeserializeWithRegistry,
            ReflectDeserializeWithRegistry,
            ReflectSerializeWithRegistry,
            SerializeWithRegistry,
        },
    },
};
use serde::{
    Serialize,
    de::DeserializeSeed,
};

use crate::reflect::{
    DynamicEntity,
    DynamicValue,
    serde::{
        EntityMapDeserializer,
        EntityMapSerializer,
        ReflectMapDeserializer,
        ReflectMapSerializer,
    },
};

/// Serializable wrapper type for `Vec<DynamicEntity>`
#[derive(Reflect, Clone)]
#[reflect(Clone, SerializeWithRegistry, DeserializeWithRegistry)]
#[type_path = "bevy_save"]
#[repr(transparent)]
pub struct EntityMap(pub Vec<DynamicEntity>);

impl SerializeWithRegistry for EntityMap {
    fn serialize<S>(&self, serializer: S, registry: &TypeRegistry) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        EntityMapSerializer::new(self, registry).serialize(serializer)
    }
}

impl<'de> DeserializeWithRegistry<'de> for EntityMap {
    fn deserialize<D>(deserializer: D, registry: &TypeRegistry) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        EntityMapDeserializer::new(registry).deserialize(deserializer)
    }
}

impl std::fmt::Debug for EntityMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        struct DebugEntity<'a>(&'a DynamicEntity);

        impl std::fmt::Debug for DebugEntity<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_map()
                    .entry(&"components", &self.0.components)
                    .finish()
            }
        }

        f.debug_map()
            .entries(self.0.iter().map(|e| (e.entity, DebugEntity(e))))
            .finish()
    }
}

impl std::ops::Deref for EntityMap {
    type Target = Vec<DynamicEntity>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for EntityMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<bevy::scene::DynamicEntity>> for EntityMap {
    fn from(value: Vec<bevy::scene::DynamicEntity>) -> Self {
        EntityMap(
            // SAFETY: DynamicEntity and bevy::scene::DynamicEntity are equivalent
            unsafe {
                std::mem::transmute::<Vec<bevy::scene::DynamicEntity>, Vec<DynamicEntity>>(value)
            },
        )
    }
}

impl FromIterator<bevy::scene::DynamicEntity> for EntityMap {
    fn from_iter<T: IntoIterator<Item = bevy::scene::DynamicEntity>>(iter: T) -> Self {
        Self(iter.into_iter().map(|r| r.into()).collect())
    }
}

impl FromIterator<DynamicEntity> for EntityMap {
    fn from_iter<T: IntoIterator<Item = DynamicEntity>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

/// Serializable wrapper type for `Vec<DynamicValue>`
#[derive(Reflect)]
#[reflect(Clone, SerializeWithRegistry, DeserializeWithRegistry)]
#[type_path = "bevy_save"]
#[repr(transparent)]
pub struct ReflectMap(pub Vec<DynamicValue>);

impl SerializeWithRegistry for ReflectMap {
    fn serialize<S>(&self, serializer: S, registry: &TypeRegistry) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        ReflectMapSerializer::new(self, registry).serialize(serializer)
    }
}

impl<'de> DeserializeWithRegistry<'de> for ReflectMap {
    fn deserialize<D>(deserializer: D, registry: &TypeRegistry) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        ReflectMapDeserializer::new(registry).deserialize(deserializer)
    }
}

impl Clone for ReflectMap {
    fn clone(&self) -> Self {
        Self(
            self.0
                .iter()
                .filter_map(|r| Some(r.reflect_clone().ok()?.into_partial_reflect().into()))
                .collect(),
        )
    }
}

impl std::fmt::Debug for ReflectMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.0.iter()).finish()
    }
}

impl std::ops::Deref for ReflectMap {
    type Target = Vec<DynamicValue>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for ReflectMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<Box<dyn PartialReflect>>> for ReflectMap {
    fn from(value: Vec<Box<dyn PartialReflect>>) -> Self {
        ReflectMap(
            // SAFETY: DynamicValue and Box<dyn PartialReflect> are equivalent
            unsafe {
                std::mem::transmute::<Vec<Box<dyn PartialReflect>>, Vec<DynamicValue>>(value)
            },
        )
    }
}

impl FromIterator<Box<dyn PartialReflect>> for ReflectMap {
    fn from_iter<T: IntoIterator<Item = Box<dyn PartialReflect>>>(iter: T) -> Self {
        Self(iter.into_iter().map(|r| r.into()).collect())
    }
}

impl FromIterator<DynamicValue> for ReflectMap {
    fn from_iter<T: IntoIterator<Item = DynamicValue>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

#[cfg(test)]
mod test {
    use bevy::prelude::*;

    use super::{
        EntityMap,
        ReflectMap,
    };

    #[test]
    fn assert_entity_map_matches() {
        let a: Vec<bevy::scene::DynamicEntity> = Vec::new();
        let b: EntityMap = Vec::new().into();

        assert_eq!(std::mem::size_of_val(&a), std::mem::size_of_val(&b));
        assert_eq!(
            std::mem::size_of_val(&a.first()),
            std::mem::size_of_val(&b.first())
        );
    }

    #[test]
    fn assert_reflect_map_matches() {
        let a: Vec<Box<dyn PartialReflect>> = Vec::new();
        let b: ReflectMap = Vec::new().into();

        assert_eq!(std::mem::size_of_val(&a), std::mem::size_of_val(&b));
        assert_eq!(
            std::mem::size_of_val(&a.first()),
            std::mem::size_of_val(&b.first())
        );
    }
}
