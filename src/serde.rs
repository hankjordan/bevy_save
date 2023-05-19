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
    entity::SaveableEntity,
    snapshot::RawSnapshot,
    Rollback,
    Rollbacks,
    Snapshot,
};

// Helpers |-----------------------------------------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(transparent)]
struct BorrowableCowStr<'a>(#[serde(borrow)] Cow<'a, str>);

// Vec<dyn Reflect> |--------------------------------------------------------------------------------------------------

struct ReflectsSerializer<'a> {
    types: &'a [Box<dyn Reflect>],
    registry: &'a TypeRegistryArc,
}

impl<'a> ReflectsSerializer<'a> {
    fn new(reflects: &'a [Box<dyn Reflect>], registry: &'a TypeRegistryArc) -> Self {
        Self {
            types: reflects,
            registry,
        }
    }
}

impl<'a> Serialize for ReflectsSerializer<'a> {
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

struct ReflectsDeserializer<'a> {
    registry: &'a TypeRegistryInternal,
}

impl<'a> ReflectsDeserializer<'a> {
    fn new(registry: &'a TypeRegistryInternal) -> Self {
        Self { registry }
    }
}

impl<'a, 'de> DeserializeSeed<'de> for ReflectsDeserializer<'a> {
    type Value = Vec<Box<dyn Reflect>>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(ReflectsVisitor {
            registry: self.registry,
        })
    }
}

struct ReflectsVisitor<'a> {
    registry: &'a TypeRegistryInternal,
}

impl<'a, 'de> Visitor<'de> for ReflectsVisitor<'a> {
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

// SaveableEntity |----------------------------------------------------------------------------------------------------

const ENTITY_STRUCT: &str = "Entity";
const ENTITY_FIELD_COMPONENTS: &str = "components";

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "lowercase")]
enum EntityField {
    Components,
}

struct EntitySerializer<'a> {
    entity: &'a SaveableEntity,
    registry: &'a TypeRegistryArc,
}

impl<'a> EntitySerializer<'a> {
    fn new(entity: &'a SaveableEntity, registry: &'a TypeRegistryArc) -> Self {
        Self { entity, registry }
    }
}

impl<'a> Serialize for EntitySerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct(ENTITY_STRUCT, 1)?;
        state.serialize_field(
            ENTITY_FIELD_COMPONENTS,
            &ReflectsSerializer::new(&self.entity.components, self.registry),
        )?;
        state.end()
    }
}

struct EntityDeserializer<'a> {
    id: u32,
    registry: &'a TypeRegistryInternal,
}

impl<'a> EntityDeserializer<'a> {
    fn new(id: u32, registry: &'a TypeRegistryInternal) -> Self {
        Self { id, registry }
    }
}

impl<'a, 'de> DeserializeSeed<'de> for EntityDeserializer<'a> {
    type Value = SaveableEntity;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_struct(ENTITY_STRUCT, &[ENTITY_FIELD_COMPONENTS], EntityVisitor {
            id: self.id,
            registry: self.registry,
        })
    }
}

struct EntityVisitor<'a> {
    id: u32,
    registry: &'a TypeRegistryInternal,
}

impl<'a, 'de> Visitor<'de> for EntityVisitor<'a> {
    type Value = SaveableEntity;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("entities")
    }

    fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let components = seq
            .next_element_seed(ReflectsDeserializer::new(self.registry))?
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

                    components =
                        Some(map.next_value_seed(ReflectsDeserializer::new(self.registry))?);
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

// Vec<SaveableEntity> |-----------------------------------------------------------------------------------------------

struct EntitiesSerializer<'a> {
    entities: &'a [SaveableEntity],
    registry: &'a TypeRegistryArc,
}

impl<'a> EntitiesSerializer<'a> {
    fn new(entities: &'a [SaveableEntity], registry: &'a TypeRegistryArc) -> Self {
        Self { entities, registry }
    }
}

impl<'a> Serialize for EntitiesSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_map(Some(self.entities.len()))?;

        for entity in self.entities {
            state.serialize_entry(
                &entity.entity,
                &EntitySerializer::new(entity, self.registry),
            )?;
        }

        state.end()
    }
}

struct EntitiesDeserializer<'a> {
    registry: &'a TypeRegistryInternal,
}

impl<'a> EntitiesDeserializer<'a> {
    fn new(registry: &'a TypeRegistryInternal) -> Self {
        Self { registry }
    }
}

impl<'a, 'de> DeserializeSeed<'de> for EntitiesDeserializer<'a> {
    type Value = Vec<SaveableEntity>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(EntitiesVisitor {
            registry: self.registry,
        })
    }
}

struct EntitiesVisitor<'a> {
    registry: &'a TypeRegistryInternal,
}

impl<'a, 'de> Visitor<'de> for EntitiesVisitor<'a> {
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
            let entity = map.next_value_seed(EntityDeserializer::new(id, self.registry))?;
            entities.push(entity);
        }

        Ok(entities)
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
        let resources = ReflectsSerializer::new(&self.snapshot.resources, self.registry);
        let entities = EntitiesSerializer::new(&self.snapshot.entities, self.registry);

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
            .next_element_seed(ReflectsDeserializer::new(self.registry))?
            .ok_or_else(|| de::Error::missing_field(RAW_SNAPSHOT_FIELDS[0]))?;

        let entities = seq
            .next_element_seed(EntitiesDeserializer::new(self.registry))?
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
                        Some(map.next_value_seed(ReflectsDeserializer::new(self.registry))?);
                }

                RawSnapshotFields::Entities => {
                    if entities.is_some() {
                        return Err(de::Error::duplicate_field(RAW_SNAPSHOT_FIELDS[1]));
                    }

                    entities = Some(map.next_value_seed(EntitiesDeserializer::new(self.registry))?);
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
            &RawSnapshotSerializer::new(&self.rollback.snapshot, self.registry),
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
        let snapshot = RawSnapshotDeserializer::new(self.registry).deserialize(deserializer)?;
        Ok(Rollback { snapshot })
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

        while let Some(rollback) =
            seq.next_element_seed(RollbackDeserializer::new(self.registry))?
        {
            rollbacks.push(rollback);
        }

        Ok(rollbacks)
    }
}

// Rollbacks |---------------------------------------------------------------------------------------------------------

const ROLLBACKS_STRUCT: &str = "Rollbacks";
const ROLLBACKS_FIELDS: &[&str] = &["checkpoints", "active"];

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "lowercase")]
enum RollbacksFields {
    Checkpoints,
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
        let checkpoints = RollbackListSerializer::new(&self.rollbacks.checkpoints, self.registry);

        let mut state = serializer.serialize_struct(ROLLBACKS_STRUCT, ROLLBACKS_FIELDS.len())?;

        state.serialize_field(ROLLBACKS_FIELDS[0], &checkpoints)?;
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
        let checkpoints = seq
            .next_element_seed(RollbackListDeserializer::new(self.registry))?
            .ok_or_else(|| de::Error::missing_field(ROLLBACKS_FIELDS[0]))?;

        let active = seq
            .next_element()?
            .ok_or_else(|| de::Error::missing_field(ROLLBACKS_FIELDS[1]))?;

        Ok(Self::Value {
            checkpoints,
            active,
        })
    }

    fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
    where
        V: MapAccess<'de>,
    {
        let mut checkpoints = None;
        let mut active = None;

        while let Some(key) = map.next_key()? {
            match key {
                RollbacksFields::Checkpoints => {
                    if checkpoints.is_some() {
                        return Err(de::Error::duplicate_field(ROLLBACKS_FIELDS[0]));
                    }

                    checkpoints =
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

        let checkpoints =
            checkpoints.ok_or_else(|| de::Error::missing_field(ROLLBACKS_FIELDS[0]))?;
        let active = active.ok_or_else(|| de::Error::missing_field(ROLLBACKS_FIELDS[1]))?;

        Ok(Self::Value {
            checkpoints,
            active,
        })
    }
}

// Snapshot |----------------------------------------------------------------------------------------------------------

const SNAPSHOT_STRUCT: &str = "Snapshot";
const SNAPSHOT_FIELDS: &[&str] = &["snapshot", "rollbacks"];

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "lowercase")]
enum SnapshotFields {
    Snapshot,
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
        let snapshot = RawSnapshotSerializer::new(&self.snapshot.snapshot, self.registry);

        let length = if self.snapshot.rollbacks.is_some() {
            2
        } else {
            1
        };

        let mut state = serializer.serialize_struct(SNAPSHOT_STRUCT, length)?;

        state.serialize_field(SNAPSHOT_FIELDS[0], &snapshot)?;

        if let Some(rollbacks) = &self.snapshot.rollbacks {
            let rollbacks = RollbacksSerializer::new(rollbacks, self.registry);
            state.serialize_field(SNAPSHOT_FIELDS[1], &rollbacks)?;
        }

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
        let snapshot = seq
            .next_element_seed(RawSnapshotDeserializer::new(self.registry))?
            .ok_or_else(|| de::Error::missing_field(SNAPSHOT_FIELDS[0]))?;

        let rollbacks = seq.next_element_seed(RollbacksDeserializer::new(self.registry))?;

        Ok(Self::Value {
            snapshot,
            rollbacks,
        })
    }

    fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
    where
        V: MapAccess<'de>,
    {
        let mut snapshot = None;
        let mut rollbacks = None;

        while let Some(key) = map.next_key()? {
            match key {
                SnapshotFields::Snapshot => {
                    if snapshot.is_some() {
                        return Err(de::Error::duplicate_field(SNAPSHOT_FIELDS[0]));
                    }
                    snapshot =
                        Some(map.next_value_seed(RawSnapshotDeserializer::new(self.registry))?);
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

        let snapshot = snapshot.ok_or_else(|| de::Error::missing_field(SNAPSHOT_FIELDS[0]))?;

        Ok(Self::Value {
            snapshot,
            rollbacks,
        })
    }
}
