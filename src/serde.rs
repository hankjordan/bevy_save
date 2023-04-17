use std::{
    borrow::Cow,
    collections::HashSet,
};

use bevy::{
    prelude::*,
    reflect::{
        serde::{
            TypedReflectDeserializer,
            TypedReflectSerializer,
            UntypedReflectDeserializer,
        },
        TypeRegistryArc,
        TypeRegistryInternal,
    },
};
use serde::{
    de::{
        self,
        DeserializeSeed,
        Error,
        MapAccess,
        SeqAccess,
        Visitor,
    },
    ser::{
        SerializeMap,
        SerializeSeq,
        SerializeStruct,
    },
    Deserialize,
    Deserializer,
    Serialize,
    Serializer,
};

use crate::{
    RawSnapshot,
    Rollback,
    Rollbacks,
    SaveableEntity,
    SaveableScene,
    Snapshot,
};

#[derive(Deserialize)]
#[serde(transparent)]
struct BorrowableCowStr<'a>(#[serde(borrow)] Cow<'a, str>);

struct ReflectListSerializer<'a> {
    types: &'a [Box<dyn Reflect>],
    registry: &'a TypeRegistryArc,
}

impl<'a> ReflectListSerializer<'a> {
    fn new(reflects: &'a [Box<dyn Reflect>], registry: &'a TypeRegistryArc) -> Self {
        Self {
            types: reflects,
            registry,
        }
    }
}

impl<'a> Serialize for ReflectListSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_map(Some(self.types.len()))?;

        for reflect in self.types {
            state.serialize_entry(
                reflect.type_name(),
                &TypedReflectSerializer::new(&**reflect, &self.registry.read()),
            )?;
        }

        state.end()
    }
}

struct ReflectListDeserializer<'a> {
    registry: &'a TypeRegistryInternal,
}

impl<'a> ReflectListDeserializer<'a> {
    fn new(registry: &'a TypeRegistryInternal) -> Self {
        Self { registry }
    }
}

impl<'a, 'de> DeserializeSeed<'de> for ReflectListDeserializer<'a> {
    type Value = Vec<Box<dyn Reflect>>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(ReflectListVisitor {
            registry: self.registry,
        })
    }
}

struct ReflectListVisitor<'a> {
    registry: &'a TypeRegistryInternal,
}

impl<'a, 'de> Visitor<'de> for ReflectListVisitor<'a> {
    type Value = Vec<Box<dyn Reflect>>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("map of reflected types")
    }

    fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut added = HashSet::new();
        let mut reflects = Vec::new();

        while let Some(BorrowableCowStr(key)) = map.next_key()? {
            if !added.insert(key.clone()) {
                return Err(de::Error::custom(format!("duplicate key: `{key}`")));
            }

            let registration = self
                .registry
                .get_with_name(&key)
                .ok_or_else(|| de::Error::custom(format!("no registration found for `{key}`")))?;

            reflects.push(
                map.next_value_seed(TypedReflectDeserializer::new(registration, self.registry))?,
            );
        }

        Ok(reflects)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut properties = Vec::new();

        while let Some(entity) =
            seq.next_element_seed(UntypedReflectDeserializer::new(self.registry))?
        {
            properties.push(entity);
        }

        Ok(properties)
    }
}

// SaveableScene |-----------------------------------------------------------------------------------------------------

const SCENE_STRUCT: &str = "Scene";
const SCENE_ENTITIES: &str = "entities";

const ENTITY_STRUCT: &str = "Entity";
const ENTITY_FIELD_COMPONENTS: &str = "components";

/// A serializer for [`SaveableScene`] that uses reflection.
pub struct SceneSerializer<'a> {
    scene: &'a SaveableScene,
    registry: &'a TypeRegistryArc,
}

impl<'a> SceneSerializer<'a> {
    /// Returns a new instance of [`SceneSerializer`].
    pub fn new(scene: &'a SaveableScene, registry: &'a TypeRegistryArc) -> Self {
        SceneSerializer { scene, registry }
    }
}

impl<'a> Serialize for SceneSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct(SCENE_STRUCT, 1)?;
        state.serialize_field(SCENE_ENTITIES, &EntitiesSerializer {
            entities: &self.scene.entities,
            registry: self.registry,
        })?;
        state.end()
    }
}

struct EntitiesSerializer<'a> {
    entities: &'a [SaveableEntity],
    registry: &'a TypeRegistryArc,
}

impl<'a> Serialize for EntitiesSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_map(Some(self.entities.len()))?;
        for entity in self.entities {
            state.serialize_entry(&entity.entity, &EntitySerializer {
                entity,
                registry: self.registry,
            })?;
        }
        state.end()
    }
}

/// A serializer for [`SaveableEntity`] that uses reflection.
pub struct EntitySerializer<'a> {
    entity: &'a SaveableEntity,
    registry: &'a TypeRegistryArc,
}

impl<'a> EntitySerializer<'a> {
    /// Returns a new instance of [`EntitySerializer`].
    pub fn new(entity: &'a SaveableEntity, registry: &'a TypeRegistryArc) -> Self {
        Self { entity, registry }
    }
}

impl<'a> Serialize for EntitySerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct(ENTITY_STRUCT, 1)?;
        state.serialize_field(ENTITY_FIELD_COMPONENTS, &ReflectListSerializer {
            types: &self.entity.components,
            registry: self.registry,
        })?;
        state.end()
    }
}

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

/// A deserializer for [`SaveableScene`] that uses reflection.
pub struct SceneDeserializer<'a> {
    registry: &'a TypeRegistryInternal,
}

impl<'a> SceneDeserializer<'a> {
    /// Returns a new instance of [`SceneDeserializer`].
    pub fn new(registry: &'a TypeRegistryInternal) -> Self {
        Self { registry }
    }
}

impl<'a, 'de> DeserializeSeed<'de> for SceneDeserializer<'a> {
    type Value = SaveableScene;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_struct(SCENE_STRUCT, &[SCENE_ENTITIES], SceneVisitor {
            registry: self.registry,
        })
    }
}

struct SceneVisitor<'a> {
    registry: &'a TypeRegistryInternal,
}

impl<'a, 'de> Visitor<'de> for SceneVisitor<'a> {
    type Value = SaveableScene;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
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
                        registry: self.registry,
                    })?);
                }
            }
        }

        let entities = entities.ok_or_else(|| Error::missing_field(SCENE_ENTITIES))?;

        Ok(SaveableScene { entities })
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let entities = seq
            .next_element_seed(SceneEntitiesDeserializer {
                registry: self.registry,
            })?
            .ok_or_else(|| Error::missing_field(SCENE_ENTITIES))?;

        Ok(SaveableScene { entities })
    }
}

struct SceneEntitiesDeserializer<'a> {
    registry: &'a TypeRegistryInternal,
}

impl<'a, 'de> DeserializeSeed<'de> for SceneEntitiesDeserializer<'a> {
    type Value = Vec<SaveableEntity>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(SceneEntitiesVisitor {
            registry: self.registry,
        })
    }
}

struct SceneEntitiesVisitor<'a> {
    registry: &'a TypeRegistryInternal,
}

impl<'a, 'de> Visitor<'de> for SceneEntitiesVisitor<'a> {
    type Value = Vec<SaveableEntity>;

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
                registry: self.registry,
            })?;
            entities.push(entity);
        }

        Ok(entities)
    }
}

struct SceneEntityDeserializer<'a> {
    id: u32,
    registry: &'a TypeRegistryInternal,
}

impl<'a, 'de> DeserializeSeed<'de> for SceneEntityDeserializer<'a> {
    type Value = SaveableEntity;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_struct(
            ENTITY_STRUCT,
            &[ENTITY_FIELD_COMPONENTS],
            SceneEntityVisitor {
                id: self.id,
                registry: self.registry,
            },
        )
    }
}

struct SceneEntityVisitor<'a> {
    id: u32,
    registry: &'a TypeRegistryInternal,
}

impl<'a, 'de> Visitor<'de> for SceneEntityVisitor<'a> {
    type Value = SaveableEntity;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("entities")
    }

    fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let components = seq
            .next_element_seed(ReflectListDeserializer {
                registry: self.registry,
            })?
            .ok_or_else(|| Error::missing_field(ENTITY_FIELD_COMPONENTS))?;

        Ok(SaveableEntity {
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

                    components = Some(map.next_value_seed(ReflectListDeserializer {
                        registry: self.registry,
                    })?);
                }
            }
        }

        let components = components
            .take()
            .ok_or_else(|| Error::missing_field(ENTITY_FIELD_COMPONENTS))?;
        Ok(SaveableEntity {
            entity: self.id,
            components,
        })
    }
}

// RawSnapshot |-------------------------------------------------------------------------------------------------------

const RAW_SNAPSHOT_STRUCT: &str = "RawSnapshot";
const RAW_SNAPSHOT_FIELDS: &[&str] = &["resources", "entities"];

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "lowercase")]
enum RawSnapshotFields {
    Resources,
    Entities,
}

struct RawSnapshotSerializer<'a> {
    snapshot: &'a RawSnapshot,
    registry: &'a TypeRegistryArc,
}

impl<'a> RawSnapshotSerializer<'a> {
    fn new(snapshot: &'a RawSnapshot, registry: &'a TypeRegistryArc) -> Self {
        Self { snapshot, registry }
    }
}

impl<'a> Serialize for RawSnapshotSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let resources = ReflectListSerializer::new(&self.snapshot.resources, self.registry);
        let entities = SceneSerializer::new(&self.snapshot.entities, self.registry);

        let mut state =
            serializer.serialize_struct(RAW_SNAPSHOT_STRUCT, RAW_SNAPSHOT_FIELDS.len())?;

        state.serialize_field(RAW_SNAPSHOT_FIELDS[0], &resources)?;
        state.serialize_field(RAW_SNAPSHOT_FIELDS[1], &entities)?;

        state.end()
    }
}

struct RawSnapshotDeserializer<'a> {
    registry: &'a TypeRegistryInternal,
}

impl<'a> RawSnapshotDeserializer<'a> {
    fn new(registry: &'a TypeRegistryInternal) -> Self {
        Self { registry }
    }
}

impl<'a, 'de> DeserializeSeed<'de> for RawSnapshotDeserializer<'a> {
    type Value = RawSnapshot;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_struct(
            RAW_SNAPSHOT_STRUCT,
            RAW_SNAPSHOT_FIELDS,
            RawSnapshotVisitor {
                registry: self.registry,
            },
        )
    }
}

struct RawSnapshotVisitor<'a> {
    registry: &'a TypeRegistryInternal,
}

impl<'a, 'de> Visitor<'de> for RawSnapshotVisitor<'a> {
    type Value = RawSnapshot;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("RawSnapshot")
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let resources = seq
            .next_element_seed(ReflectListDeserializer::new(self.registry))?
            .ok_or_else(|| de::Error::missing_field(RAW_SNAPSHOT_FIELDS[0]))?;

        let entities = seq
            .next_element_seed(SceneDeserializer {
                registry: self.registry,
            })?
            .ok_or_else(|| de::Error::missing_field(RAW_SNAPSHOT_FIELDS[1]))?;

        Ok(Self::Value {
            resources,
            entities,
        })
    }

    fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
    where
        V: MapAccess<'de>,
    {
        let mut resources = None;
        let mut entities = None;

        while let Some(key) = map.next_key()? {
            match key {
                RawSnapshotFields::Resources => {
                    if resources.is_some() {
                        return Err(de::Error::duplicate_field(RAW_SNAPSHOT_FIELDS[0]));
                    }
                    resources =
                        Some(map.next_value_seed(ReflectListDeserializer::new(self.registry))?);
                }

                RawSnapshotFields::Entities => {
                    if entities.is_some() {
                        return Err(de::Error::duplicate_field(RAW_SNAPSHOT_FIELDS[1]));
                    }

                    entities = Some(map.next_value_seed(SceneDeserializer {
                        registry: self.registry,
                    })?);
                }
            }
        }

        let resources =
            resources.ok_or_else(|| de::Error::missing_field(RAW_SNAPSHOT_FIELDS[0]))?;
        let entities = entities.ok_or_else(|| de::Error::missing_field(RAW_SNAPSHOT_FIELDS[1]))?;

        Ok(Self::Value {
            resources,
            entities,
        })
    }
}

// Rollback |----------------------------------------------------------------------------------------------------------

const ROLLBACK_STRUCT: &str = "Rollback";

/// A serializer for [`Rollback`] that uses reflection.
pub struct RollbackSerializer<'a> {
    rollback: &'a Rollback,
    registry: &'a TypeRegistryArc,
}

impl<'a> RollbackSerializer<'a> {
    /// Returns a new instance of [`RollbackSerializer`].
    pub fn new(rollback: &'a Rollback, registry: &'a TypeRegistryArc) -> Self {
        Self { rollback, registry }
    }
}

impl<'a> Serialize for RollbackSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_newtype_struct(
            ROLLBACK_STRUCT,
            &RawSnapshotSerializer::new(&self.rollback.inner, self.registry),
        )
    }
}

/// A deserializer for [`Rollback`] that uses reflection.
pub struct RollbackDeserializer<'a> {
    registry: &'a TypeRegistryInternal,
}

impl<'a> RollbackDeserializer<'a> {
    /// Returns a new instance of [`RollbackDeserializer`].
    pub fn new(registry: &'a TypeRegistryInternal) -> Self {
        Self { registry }
    }
}

impl<'a, 'de> DeserializeSeed<'de> for RollbackDeserializer<'a> {
    type Value = Rollback;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_newtype_struct(ROLLBACK_STRUCT, RollbackVisitor {
            registry: self.registry,
        })
    }
}

struct RollbackVisitor<'a> {
    registry: &'a TypeRegistryInternal,
}

impl<'a, 'de> Visitor<'de> for RollbackVisitor<'a> {
    type Value = Rollback;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("rollback newtype")
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let inner =
            deserializer.deserialize_newtype_struct(ROLLBACK_STRUCT, RawSnapshotVisitor {
                registry: self.registry,
            })?;
        Ok(Rollback { inner })
    }
}

// RollbackList |------------------------------------------------------------------------------------------------------

struct RollbackListSerializer<'a> {
    rollbacks: &'a [Rollback],
    registry: &'a TypeRegistryArc,
}

impl<'a> RollbackListSerializer<'a> {
    fn new(rollbacks: &'a [Rollback], registry: &'a TypeRegistryArc) -> Self {
        Self {
            rollbacks,
            registry,
        }
    }
}

impl<'a> Serialize for RollbackListSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.rollbacks.len()))?;

        for rollback in self.rollbacks {
            seq.serialize_element(&RollbackSerializer::new(rollback, self.registry))?;
        }

        seq.end()
    }
}

struct RollbackListDeserializer<'a> {
    registry: &'a TypeRegistryInternal,
}

impl<'a> RollbackListDeserializer<'a> {
    fn new(registry: &'a TypeRegistryInternal) -> Self {
        Self { registry }
    }
}

impl<'a, 'de> DeserializeSeed<'de> for RollbackListDeserializer<'a> {
    type Value = Vec<Rollback>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(RollbackListVisitor {
            registry: self.registry,
        })
    }
}

struct RollbackListVisitor<'a> {
    registry: &'a TypeRegistryInternal,
}

impl<'a, 'de> Visitor<'de> for RollbackListVisitor<'a> {
    type Value = Vec<Rollback>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("map of rollbacks")
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let mut rollbacks = Vec::new();

        while let Some(rollback) = seq.next_element_seed(RollbackDeserializer {
            registry: self.registry,
        })? {
            rollbacks.push(rollback);
        }

        Ok(rollbacks)
    }
}

// Rollbacks |---------------------------------------------------------------------------------------------------------

const ROLLBACKS_STRUCT: &str = "Rollbacks";
const ROLLBACKS_FIELDS: &[&str] = &["rollbacks", "active"];

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "lowercase")]
enum RollbacksFields {
    Rollbacks,
    Active,
}

/// A serializer for [`Rollbacks`] that uses reflection.
pub struct RollbacksSerializer<'a> {
    rollbacks: &'a Rollbacks,
    registry: &'a TypeRegistryArc,
}

impl<'a> RollbacksSerializer<'a> {
    /// Returns a new instance of [`RollbacksSerializer`]
    pub fn new(rollbacks: &'a Rollbacks, registry: &'a TypeRegistryArc) -> Self {
        Self {
            rollbacks,
            registry,
        }
    }
}

impl<'a> Serialize for RollbacksSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let snapshots = RollbackListSerializer::new(&self.rollbacks.rollbacks, self.registry);

        let mut state = serializer.serialize_struct(ROLLBACKS_STRUCT, ROLLBACKS_FIELDS.len())?;

        state.serialize_field(ROLLBACKS_FIELDS[0], &snapshots)?;
        state.serialize_field(ROLLBACKS_FIELDS[1], &self.rollbacks.active)?;

        state.end()
    }
}

/// A deserializer for [`Rollbacks`] that uses reflection.
pub struct RollbacksDeserializer<'a> {
    registry: &'a TypeRegistryInternal,
}

impl<'a> RollbacksDeserializer<'a> {
    /// Returns a new instance of [`RollbacksDeserializer`].
    pub fn new(registry: &'a TypeRegistryInternal) -> Self {
        Self { registry }
    }
}

impl<'a, 'de> DeserializeSeed<'de> for RollbacksDeserializer<'a> {
    type Value = Rollbacks;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_struct(ROLLBACKS_STRUCT, ROLLBACKS_FIELDS, RollbacksVisitor {
            registry: self.registry,
        })
    }
}

struct RollbacksVisitor<'a> {
    registry: &'a TypeRegistryInternal,
}

impl<'a, 'de> Visitor<'de> for RollbacksVisitor<'a> {
    type Value = Rollbacks;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("struct Rollbacks")
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let rollbacks = seq
            .next_element_seed(RollbackListDeserializer::new(self.registry))?
            .ok_or_else(|| de::Error::missing_field(ROLLBACKS_FIELDS[0]))?;

        let active = seq
            .next_element()?
            .ok_or_else(|| de::Error::missing_field(ROLLBACKS_FIELDS[1]))?;

        Ok(Self::Value { rollbacks, active })
    }

    fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
    where
        V: MapAccess<'de>,
    {
        let mut rollbacks = None;
        let mut active = None;

        while let Some(key) = map.next_key()? {
            match key {
                RollbacksFields::Rollbacks => {
                    if rollbacks.is_some() {
                        return Err(de::Error::duplicate_field(ROLLBACKS_FIELDS[0]));
                    }

                    rollbacks =
                        Some(map.next_value_seed(RollbackListDeserializer::new(self.registry))?);
                }

                RollbacksFields::Active => {
                    if active.is_some() {
                        return Err(de::Error::duplicate_field(ROLLBACKS_FIELDS[1]));
                    }

                    active = Some(map.next_value()?);
                }
            }
        }

        let rollbacks = rollbacks.ok_or_else(|| de::Error::missing_field(ROLLBACKS_FIELDS[0]))?;
        let active = active.ok_or_else(|| de::Error::missing_field(ROLLBACKS_FIELDS[1]))?;

        Ok(Self::Value { rollbacks, active })
    }
}

// Snapshot |----------------------------------------------------------------------------------------------------------

const SNAPSHOT_STRUCT: &str = "Snapshot";
const SNAPSHOT_FIELDS: &[&str] = &["inner", "rollbacks"];

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "lowercase")]
enum SnapshotFields {
    Inner,
    Rollbacks,
}

/// A serializer for [`Snapshot`] that uses reflection.
pub struct SnapshotSerializer<'a> {
    snapshot: &'a Snapshot,
    registry: &'a TypeRegistryArc,
}

impl<'a> SnapshotSerializer<'a> {
    /// Returns a new instance of [`SnapshotSerializer`].
    pub fn new(snapshot: &'a Snapshot, registry: &'a TypeRegistryArc) -> Self {
        Self { snapshot, registry }
    }
}

impl<'a> Serialize for SnapshotSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let inner = RawSnapshotSerializer::new(&self.snapshot.inner, self.registry);
        let rollbacks = RollbacksSerializer::new(&self.snapshot.rollbacks, self.registry);

        let mut state = serializer.serialize_struct(SNAPSHOT_STRUCT, SNAPSHOT_FIELDS.len())?;

        state.serialize_field(SNAPSHOT_FIELDS[0], &inner)?;
        state.serialize_field(SNAPSHOT_FIELDS[1], &rollbacks)?;

        state.end()
    }
}

/// A deserializer for [`Snapshot`] that uses reflection.
pub struct SnapshotDeserializer<'a> {
    registry: &'a TypeRegistryInternal,
}

impl<'a> SnapshotDeserializer<'a> {
    /// Returns a new instance of [`SnapshotDeserializer`].
    pub fn new(registry: &'a TypeRegistryInternal) -> Self {
        Self { registry }
    }
}

impl<'a, 'de> DeserializeSeed<'de> for SnapshotDeserializer<'a> {
    type Value = Snapshot;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_struct(SNAPSHOT_STRUCT, SNAPSHOT_FIELDS, SnapshotVisitor {
            registry: self.registry,
        })
    }
}

struct SnapshotVisitor<'a> {
    registry: &'a TypeRegistryInternal,
}

impl<'a, 'de> Visitor<'de> for SnapshotVisitor<'a> {
    type Value = Snapshot;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("struct Snapshot")
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let inner = seq
            .next_element_seed(RawSnapshotDeserializer::new(self.registry))?
            .ok_or_else(|| de::Error::missing_field(SNAPSHOT_FIELDS[0]))?;

        let rollbacks = seq
            .next_element_seed(RollbacksDeserializer::new(self.registry))?
            .ok_or_else(|| de::Error::missing_field(SNAPSHOT_FIELDS[1]))?;

        Ok(Self::Value { inner, rollbacks })
    }

    fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
    where
        V: MapAccess<'de>,
    {
        let mut inner = None;
        let mut rollbacks = None;

        while let Some(key) = map.next_key()? {
            match key {
                SnapshotFields::Inner => {
                    if inner.is_some() {
                        return Err(de::Error::duplicate_field(SNAPSHOT_FIELDS[0]));
                    }
                    inner = Some(map.next_value_seed(RawSnapshotDeserializer::new(self.registry))?);
                }

                SnapshotFields::Rollbacks => {
                    if rollbacks.is_some() {
                        return Err(de::Error::duplicate_field(SNAPSHOT_FIELDS[1]));
                    }

                    rollbacks =
                        Some(map.next_value_seed(RollbacksDeserializer::new(self.registry))?);
                }
            }
        }

        let inner = inner.ok_or_else(|| de::Error::missing_field(SNAPSHOT_FIELDS[0]))?;
        let rollbacks = rollbacks.ok_or_else(|| de::Error::missing_field(SNAPSHOT_FIELDS[1]))?;

        Ok(Self::Value { inner, rollbacks })
    }
}
