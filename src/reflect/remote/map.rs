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
    BoxedPartialReflect,
    DynamicEntity,
    serde::{
        EntityMapDeserializer,
        EntityMapSerializer,
        ReflectMapDeserializer,
        ReflectMapSerializer,
    },
};

/// Serializable wrapper type for `Vec<DynamicEntity>`
#[derive(Reflect, Debug)]
#[reflect(SerializeWithRegistry, DeserializeWithRegistry)]
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

/// Serializable wrapper type for `Vec<BoxedPartialReflect>`
#[derive(Reflect, Debug)]
#[reflect(SerializeWithRegistry, DeserializeWithRegistry)]
#[type_path = "bevy_save"]
#[repr(transparent)]
pub struct ReflectMap(pub Vec<BoxedPartialReflect>);

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
