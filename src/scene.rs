#![allow(clippy::pedantic)]
// Backport `bevy_reflect: Fix deserialization with readers (#6894)`

use std::{
    borrow::Cow,
    collections::HashSet,
    fmt::Formatter,
};

use bevy::{
    prelude::*,
    reflect::{
        serde::TypedReflectDeserializer,
        TypeRegistryInternal,
    },
    scene::DynamicEntity,
};
use serde::{
    de::{
        DeserializeSeed,
        Error,
        MapAccess,
        SeqAccess,
        Visitor,
    },
    Deserialize,
    Deserializer,
};

type TypeRegistry = TypeRegistryInternal;

pub const SCENE_STRUCT: &str = "Scene";
pub const SCENE_ENTITIES: &str = "entities";

pub const ENTITY_STRUCT: &str = "Entity";
pub const ENTITY_FIELD_COMPONENTS: &str = "components";

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "lowercase")]
enum SceneField {
    Entities,
}

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "lowercase")]
enum EntityField {
    Components,
}

pub struct SceneDeserializer<'a> {
    pub type_registry: &'a TypeRegistry,
}

impl<'a, 'de> DeserializeSeed<'de> for SceneDeserializer<'a> {
    type Value = DynamicScene;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_struct(SCENE_STRUCT, &[SCENE_ENTITIES], SceneVisitor {
            type_registry: self.type_registry,
        })
    }
}

struct SceneVisitor<'a> {
    pub type_registry: &'a TypeRegistry,
}

impl<'a, 'de> Visitor<'de> for SceneVisitor<'a> {
    type Value = DynamicScene;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("scene struct")
    }

    fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut entities = None;
        while let Some(key) = map.next_key()? {
            match key {
                SceneField::Entities => {
                    if entities.is_some() {
                        return Err(Error::duplicate_field(SCENE_ENTITIES));
                    }
                    entities = Some(map.next_value_seed(SceneEntitiesDeserializer {
                        type_registry: self.type_registry,
                    })?);
                }
            }
        }

        let entities = entities.ok_or_else(|| Error::missing_field(SCENE_ENTITIES))?;

        Ok(DynamicScene { entities })
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let entities = seq
            .next_element_seed(SceneEntitiesDeserializer {
                type_registry: self.type_registry,
            })?
            .ok_or_else(|| Error::missing_field(SCENE_ENTITIES))?;

        Ok(DynamicScene { entities })
    }
}

pub struct SceneEntitiesDeserializer<'a> {
    pub type_registry: &'a TypeRegistry,
}

impl<'a, 'de> DeserializeSeed<'de> for SceneEntitiesDeserializer<'a> {
    type Value = Vec<DynamicEntity>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(SceneEntitiesVisitor {
            type_registry: self.type_registry,
        })
    }
}

struct SceneEntitiesVisitor<'a> {
    pub type_registry: &'a TypeRegistry,
}

impl<'a, 'de> Visitor<'de> for SceneEntitiesVisitor<'a> {
    type Value = Vec<DynamicEntity>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("map of entities")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut entities = Vec::new();
        while let Some(id) = map.next_key::<u32>()? {
            let entity = map.next_value_seed(SceneEntityDeserializer {
                id,
                type_registry: self.type_registry,
            })?;
            entities.push(entity);
        }

        Ok(entities)
    }
}

pub struct SceneEntityDeserializer<'a> {
    pub id: u32,
    pub type_registry: &'a TypeRegistry,
}

impl<'a, 'de> DeserializeSeed<'de> for SceneEntityDeserializer<'a> {
    type Value = DynamicEntity;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_struct(
            ENTITY_STRUCT,
            &[ENTITY_FIELD_COMPONENTS],
            SceneEntityVisitor {
                id: self.id,
                registry: self.type_registry,
            },
        )
    }
}

struct SceneEntityVisitor<'a> {
    pub id: u32,
    pub registry: &'a TypeRegistry,
}

impl<'a, 'de> Visitor<'de> for SceneEntityVisitor<'a> {
    type Value = DynamicEntity;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("entities")
    }

    fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let components = seq
            .next_element_seed(ComponentDeserializer {
                registry: self.registry,
            })?
            .ok_or_else(|| Error::missing_field(ENTITY_FIELD_COMPONENTS))?;

        Ok(DynamicEntity {
            entity: self.id,
            components,
        })
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut components = None;
        while let Some(key) = map.next_key()? {
            match key {
                EntityField::Components => {
                    if components.is_some() {
                        return Err(Error::duplicate_field(ENTITY_FIELD_COMPONENTS));
                    }

                    components = Some(map.next_value_seed(ComponentDeserializer {
                        registry: self.registry,
                    })?);
                }
            }
        }

        let components = components
            .take()
            .ok_or_else(|| Error::missing_field(ENTITY_FIELD_COMPONENTS))?;
        Ok(DynamicEntity {
            entity: self.id,
            components,
        })
    }
}

pub struct ComponentDeserializer<'a> {
    pub registry: &'a TypeRegistry,
}

impl<'a, 'de> DeserializeSeed<'de> for ComponentDeserializer<'a> {
    type Value = Vec<Box<dyn Reflect>>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(ComponentVisitor {
            registry: self.registry,
        })
    }
}

struct ComponentVisitor<'a> {
    pub registry: &'a TypeRegistry,
}

impl<'a, 'de> Visitor<'de> for ComponentVisitor<'a> {
    type Value = Vec<Box<dyn Reflect>>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("map of components")
    }

    fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut added = HashSet::new();
        let mut components = Vec::new();
        while let Some(BorrowableCowStr(key)) = map.next_key()? {
            if !added.insert(key.clone()) {
                return Err(Error::custom(format!("duplicate component: `{key}`")));
            }

            let registration = self
                .registry
                .get_with_name(&key)
                .ok_or_else(|| Error::custom(format!("no registration found for `{key}`")))?;
            components.push(
                map.next_value_seed(TypedReflectDeserializer::new(registration, self.registry))?,
            );
        }

        Ok(components)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut dynamic_properties = Vec::new();
        while let Some(entity) =
            seq.next_element_seed(UntypedReflectDeserializer::new(self.registry))?
        {
            dynamic_properties.push(entity);
        }

        Ok(dynamic_properties)
    }
}

/// Helper struct for deserializing strings without allocating (when possible).
///
/// Based on [this comment](https://github.com/bevyengine/bevy/pull/6894#discussion_r1045069010).
#[derive(Deserialize)]
#[serde(transparent)]
struct BorrowableCowStr<'a>(#[serde(borrow)] Cow<'a, str>);

/// A general purpose deserializer for reflected types.
///
/// This will return a [`Box<dyn Reflect>`] containing the deserialized data.
/// For non-value types, this `Box` will contain the dynamic equivalent. For example, a
/// deserialized struct will return a [`DynamicStruct`] and a `Vec` will return a
/// [`DynamicList`]. For value types, this `Box` will contain the actual value.
/// For example, an `f32` will contain the actual `f32` type.
///
/// This means that converting to any concrete instance will require the use of
/// [`FromReflect`], or downcasting for value types.
///
/// Because the type isn't known ahead of time, the serialized data must take the form of
/// a map containing the following entries (in order):
/// 1. `type`: The _full_ [type name]
/// 2. `value`: The serialized value of the reflected type
///
/// If the type is already known and the [`TypeInfo`] for it can be retrieved,
/// [`TypedReflectDeserializer`] may be used instead to avoid requiring these entries.
///
/// [`Box<dyn Reflect>`]: crate::Reflect
/// [`DynamicStruct`]: crate::DynamicStruct
/// [`DynamicList`]: crate::DynamicList
/// [`FromReflect`]: crate::FromReflect
/// [type name]: std::any::type_name
pub struct UntypedReflectDeserializer<'a> {
    registry: &'a TypeRegistry,
}

impl<'a> UntypedReflectDeserializer<'a> {
    pub fn new(registry: &'a TypeRegistry) -> Self {
        Self { registry }
    }
}

impl<'a, 'de> DeserializeSeed<'de> for UntypedReflectDeserializer<'a> {
    type Value = Box<dyn Reflect>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(UntypedReflectDeserializerVisitor {
            registry: self.registry,
        })
    }
}

struct UntypedReflectDeserializerVisitor<'a> {
    registry: &'a TypeRegistry,
}

impl<'a, 'de> Visitor<'de> for UntypedReflectDeserializerVisitor<'a> {
    type Value = Box<dyn Reflect>;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("map containing `type` and `value` entries for the reflected value")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let type_name = map
            .next_key::<BorrowableCowStr>()?
            .ok_or_else(|| Error::invalid_length(0, &"at least one entry"))?
            .0;

        let registration = self.registry.get_with_name(&type_name).ok_or_else(|| {
            Error::custom(format_args!("No registration found for `{type_name}`"))
        })?;
        let value =
            map.next_value_seed(TypedReflectDeserializer::new(registration, self.registry))?;
        Ok(value)
    }
}
