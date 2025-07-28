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
    scene::serde::{
        EntitiesSerializer,
        SceneEntitiesDeserializer,
        SceneMapDeserializer,
        SceneMapSerializer,
    },
};
use serde::{
    Serialize,
    de::DeserializeSeed,
};

use crate::reflect::{
    BoxedPartialReflect,
    DynamicEntity,
};

/// Serializable wrapper type for `Vec<DynamicEntity>`
#[derive(Reflect, Debug)]
#[reflect(SerializeWithRegistry, DeserializeWithRegistry)]
#[repr(transparent)]
pub struct EntityMap(pub Vec<DynamicEntity>);

impl SerializeWithRegistry for EntityMap {
    fn serialize<S>(&self, serializer: S, registry: &TypeRegistry) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        EntitiesSerializer {
            // SAFETY: DynamicEntity and bevy::scene::DynamicEntity are equivalent
            entities: unsafe { &*(std::ptr::from_ref(self.0.as_slice()) as *const _) },
            registry,
        }
        .serialize(serializer)
    }
}

impl<'de> DeserializeWithRegistry<'de> for EntityMap {
    fn deserialize<D>(deserializer: D, registry: &TypeRegistry) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        SceneEntitiesDeserializer {
            type_registry: registry,
        }
        .deserialize(deserializer)
        .map(|m| {
            EntityMap(
                // SAFETY: DynamicEntity and bevy::scene::DynamicEntity are equivalent
                unsafe {
                    std::mem::transmute::<Vec<bevy::scene::DynamicEntity>, Vec<DynamicEntity>>(m)
                },
            )
        })
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

/// Serializable wrapper type for `Vec<BoxedPartialReflect>`
#[derive(Reflect, Debug)]
#[reflect(SerializeWithRegistry, DeserializeWithRegistry)]
#[repr(transparent)]
pub struct ReflectMap(pub Vec<BoxedPartialReflect>);

impl SerializeWithRegistry for ReflectMap {
    fn serialize<S>(&self, serializer: S, registry: &TypeRegistry) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        SceneMapSerializer {
            // SAFETY: BoxedPartialReflect and Box<dyn PartialReflect> are equivalent
            entries: unsafe { &*(std::ptr::from_ref(self.0.as_slice()) as *const _) },
            registry,
        }
        .serialize(serializer)
    }
}

impl<'de> DeserializeWithRegistry<'de> for ReflectMap {
    fn deserialize<D>(deserializer: D, registry: &TypeRegistry) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        SceneMapDeserializer { registry }
            .deserialize(deserializer)
            .map(|m| m.into())
    }
}

impl From<Vec<Box<dyn PartialReflect>>> for ReflectMap {
    fn from(value: Vec<Box<dyn PartialReflect>>) -> Self {
        ReflectMap(
            // SAFETY: BoxedPartialReflect and Box<dyn PartialReflect> are equivalent
            unsafe {
                std::mem::transmute::<Vec<Box<dyn PartialReflect>>, Vec<BoxedPartialReflect>>(value)
            },
        )
    }
}

impl FromIterator<Box<dyn PartialReflect>> for ReflectMap {
    fn from_iter<T: IntoIterator<Item = Box<dyn PartialReflect>>>(iter: T) -> Self {
        Self(iter.into_iter().map(|r| r.into()).collect())
    }
}

impl FromIterator<BoxedPartialReflect> for ReflectMap {
    fn from_iter<T: IntoIterator<Item = BoxedPartialReflect>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}
