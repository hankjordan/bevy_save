use std::{
    borrow::Cow,
    collections::HashSet,
};

use bevy::{
    reflect::{
        serde::{
            TypedReflectDeserializer,
            TypedReflectSerializer,
        },
        Reflect,
        TypeRegistryArc,
        TypeRegistryInternal,
    },
    scene::serde::SceneSerializer,
};
use serde::{
    de::{
        self,
        DeserializeSeed,
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
    Serialize,
};

use crate::{
    scene::{
        SceneDeserializer,
        UntypedReflectDeserializer,
    },
    Capture,
    RollbackSnapshot,
    Rollbacks,
    Snapshot,
};

// Helpers |-----------------------------------------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(transparent)]
struct BorrowableCowStr<'a>(#[serde(borrow)] Cow<'a, str>);

// Resources |---------------------------------------------------------------------------------------------------------

struct ResourcesSerializer<'a> {
    resources: &'a [Box<dyn Reflect>],
    registry: &'a TypeRegistryArc,
}

impl<'a> ResourcesSerializer<'a> {
    fn new(resources: &'a [Box<dyn Reflect>], registry: &'a TypeRegistryArc) -> Self {
        Self {
            resources,
            registry,
        }
    }
}

impl<'a> Serialize for ResourcesSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_map(Some(self.resources.len()))?;

        for resource in self.resources {
            state.serialize_entry(
                resource.type_name(),
                &TypedReflectSerializer::new(&**resource, &self.registry.read()),
            )?;
        }

        state.end()
    }
}

struct ResourcesDeserializer<'a> {
    registry: &'a TypeRegistryInternal,
}

impl<'a> ResourcesDeserializer<'a> {
    fn new(registry: &'a TypeRegistryInternal) -> Self {
        Self { registry }
    }
}

impl<'a, 'de> DeserializeSeed<'de> for ResourcesDeserializer<'a> {
    type Value = Vec<Box<dyn Reflect>>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(ResourcesVisitor {
            registry: self.registry,
        })
    }
}

struct ResourcesVisitor<'a> {
    registry: &'a TypeRegistryInternal,
}

impl<'a, 'de> Visitor<'de> for ResourcesVisitor<'a> {
    type Value = Vec<Box<dyn Reflect>>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("map of resources")
    }

    fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut added = HashSet::new();
        let mut resources = Vec::new();

        while let Some(BorrowableCowStr(key)) = map.next_key()? {
            if !added.insert(key.clone()) {
                return Err(de::Error::custom(format!("duplicate resource: `{key}`")));
            }

            let registration = self
                .registry
                .get_with_name(&key)
                .ok_or_else(|| de::Error::custom(format!("no registration found for `{key}`")))?;

            resources.push(
                map.next_value_seed(TypedReflectDeserializer::new(registration, self.registry))?,
            );
        }

        Ok(resources)
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

// Capture |-----------------------------------------------------------------------------------------------------------

const CAPTURE_STRUCT: &str = "Capture";
const CAPTURE_FIELDS: &[&str] = &["resources", "scene"];

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "lowercase")]
enum CaptureFields {
    Resources,
    Scene,
}

struct CaptureSerializer<'a> {
    capture: &'a Capture,
    registry: &'a TypeRegistryArc,
}

impl<'a> CaptureSerializer<'a> {
    fn new(capture: &'a Capture, registry: &'a TypeRegistryArc) -> Self {
        Self { capture, registry }
    }
}

impl<'a> Serialize for CaptureSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let resources = ResourcesSerializer::new(&self.capture.resources, self.registry);
        let scene = SceneSerializer::new(&self.capture.scene, self.registry);

        let mut state = serializer.serialize_struct(CAPTURE_STRUCT, CAPTURE_FIELDS.len())?;

        state.serialize_field(CAPTURE_FIELDS[0], &resources)?;
        state.serialize_field(CAPTURE_FIELDS[1], &scene)?;

        state.end()
    }
}

struct CaptureDeserializer<'a> {
    registry: &'a TypeRegistryInternal,
}

impl<'a> CaptureDeserializer<'a> {
    fn new(registry: &'a TypeRegistryInternal) -> Self {
        Self { registry }
    }
}

impl<'a, 'de> DeserializeSeed<'de> for CaptureDeserializer<'a> {
    type Value = Capture;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_struct(CAPTURE_STRUCT, CAPTURE_FIELDS, CaptureVisitor {
            registry: self.registry,
        })
    }
}

struct CaptureVisitor<'a> {
    registry: &'a TypeRegistryInternal,
}

impl<'a, 'de> Visitor<'de> for CaptureVisitor<'a> {
    type Value = Capture;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("struct Capture")
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let resources = seq
            .next_element_seed(ResourcesDeserializer::new(self.registry))?
            .ok_or_else(|| de::Error::missing_field(CAPTURE_FIELDS[0]))?;

        let scene = seq
            .next_element_seed(SceneDeserializer {
                type_registry: self.registry,
            })?
            .ok_or_else(|| de::Error::missing_field(CAPTURE_FIELDS[1]))?;

        Ok(Self::Value { resources, scene })
    }

    fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
    where
        V: MapAccess<'de>,
    {
        let mut resources = None;
        let mut scene = None;

        while let Some(key) = map.next_key()? {
            match key {
                CaptureFields::Resources => {
                    if resources.is_some() {
                        return Err(de::Error::duplicate_field(CAPTURE_FIELDS[0]));
                    }
                    resources =
                        Some(map.next_value_seed(ResourcesDeserializer::new(self.registry))?);
                }

                CaptureFields::Scene => {
                    if scene.is_some() {
                        return Err(de::Error::duplicate_field(CAPTURE_FIELDS[1]));
                    }

                    scene = Some(map.next_value_seed(SceneDeserializer {
                        type_registry: self.registry,
                    })?);
                }
            }
        }

        let resources = resources.ok_or_else(|| de::Error::missing_field(CAPTURE_FIELDS[0]))?;
        let scene = scene.ok_or_else(|| de::Error::missing_field(CAPTURE_FIELDS[1]))?;

        Ok(Self::Value { resources, scene })
    }
}

// RollbackSnapshots |-------------------------------------------------------------------------------------------------

struct RollbackSnapshotsSerializer<'a> {
    rollbacks: &'a [RollbackSnapshot],
    registry: &'a TypeRegistryArc,
}

impl<'a> RollbackSnapshotsSerializer<'a> {
    fn new(rollbacks: &'a [RollbackSnapshot], registry: &'a TypeRegistryArc) -> Self {
        Self {
            rollbacks,
            registry,
        }
    }
}

impl<'a> Serialize for RollbackSnapshotsSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.rollbacks.len()))?;

        for rollback in self.rollbacks {
            seq.serialize_element(&CaptureSerializer::new(&rollback.capture, self.registry))?;
        }

        seq.end()
    }
}

struct RollbackSnapshotsDeserializer<'a> {
    registry: &'a TypeRegistryInternal,
}

impl<'a> RollbackSnapshotsDeserializer<'a> {
    fn new(registry: &'a TypeRegistryInternal) -> Self {
        Self { registry }
    }
}

impl<'a, 'de> DeserializeSeed<'de> for RollbackSnapshotsDeserializer<'a> {
    type Value = Vec<RollbackSnapshot>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(RollbackSnapshotsVisitor {
            registry: self.registry,
        })
    }
}

struct RollbackSnapshotsVisitor<'a> {
    registry: &'a TypeRegistryInternal,
}

impl<'a, 'de> Visitor<'de> for RollbackSnapshotsVisitor<'a> {
    type Value = Vec<RollbackSnapshot>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("map of rollbacksnapshots")
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let mut snapshots = Vec::new();

        while let Some(capture) = seq.next_element_seed(CaptureDeserializer::new(self.registry))? {
            snapshots.push(RollbackSnapshot { capture });
        }

        Ok(snapshots)
    }
}

// Rollbacks |---------------------------------------------------------------------------------------------------------

const ROLLBACKS_STRUCT: &str = "Rollbacks";
const ROLLBACKS_FIELDS: &[&str] = &["snapshots", "active"];

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "lowercase")]
enum RollbacksFields {
    Snapshots,
    Active,
}

struct RollbacksSerializer<'a> {
    rollbacks: &'a Rollbacks,
    registry: &'a TypeRegistryArc,
}

impl<'a> RollbacksSerializer<'a> {
    fn new(rollbacks: &'a Rollbacks, registry: &'a TypeRegistryArc) -> Self {
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
        let snapshots = RollbackSnapshotsSerializer::new(&self.rollbacks.snapshots, self.registry);

        let mut state = serializer.serialize_struct(ROLLBACKS_STRUCT, ROLLBACKS_FIELDS.len())?;

        state.serialize_field(ROLLBACKS_FIELDS[0], &snapshots)?;
        state.serialize_field(ROLLBACKS_FIELDS[1], &self.rollbacks.active)?;

        state.end()
    }
}

struct RollbacksDeserializer<'a> {
    registry: &'a TypeRegistryInternal,
}

impl<'a> RollbacksDeserializer<'a> {
    fn new(registry: &'a TypeRegistryInternal) -> Self {
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
        let snapshots = seq
            .next_element_seed(RollbackSnapshotsDeserializer::new(self.registry))?
            .ok_or_else(|| de::Error::missing_field(ROLLBACKS_FIELDS[0]))?;

        let active = seq
            .next_element()?
            .ok_or_else(|| de::Error::missing_field(ROLLBACKS_FIELDS[1]))?;

        Ok(Self::Value { snapshots, active })
    }

    fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
    where
        V: MapAccess<'de>,
    {
        let mut snapshots = None;
        let mut active = None;

        while let Some(key) = map.next_key()? {
            match key {
                RollbacksFields::Snapshots => {
                    if snapshots.is_some() {
                        return Err(de::Error::duplicate_field(ROLLBACKS_FIELDS[0]));
                    }
                    snapshots = Some(
                        map.next_value_seed(RollbackSnapshotsDeserializer::new(self.registry))?,
                    );
                }

                RollbacksFields::Active => {
                    if active.is_some() {
                        return Err(de::Error::duplicate_field(ROLLBACKS_FIELDS[1]));
                    }

                    active = Some(map.next_value()?);
                }
            }
        }

        let snapshots = snapshots.ok_or_else(|| de::Error::missing_field(ROLLBACKS_FIELDS[0]))?;
        let active = active.ok_or_else(|| de::Error::missing_field(ROLLBACKS_FIELDS[1]))?;

        Ok(Self::Value { snapshots, active })
    }
}

// Snapshot |----------------------------------------------------------------------------------------------------------

const SNAPSHOT_STRUCT: &str = "Rollbacks";
const SNAPSHOT_FIELDS: &[&str] = &["capture", "rollbacks"];

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "lowercase")]
enum SnapshotFields {
    Capture,
    Rollbacks,
}

/// A serializer for `Snapshot` that uses reflection.
pub struct SnapshotSerializer<'a> {
    snapshot: &'a Snapshot,
    registry: &'a TypeRegistryArc,
}

impl<'a> SnapshotSerializer<'a> {
    /// Returns a new instance of `SnapshotSerializer`.
    pub fn new(snapshot: &'a Snapshot, registry: &'a TypeRegistryArc) -> Self {
        Self { snapshot, registry }
    }
}

impl<'a> Serialize for SnapshotSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let capture = CaptureSerializer::new(&self.snapshot.capture, self.registry);
        let rollbacks = RollbacksSerializer::new(&self.snapshot.rollbacks, self.registry);

        let mut state = serializer.serialize_struct(SNAPSHOT_STRUCT, SNAPSHOT_FIELDS.len())?;

        state.serialize_field(SNAPSHOT_FIELDS[0], &capture)?;
        state.serialize_field(SNAPSHOT_FIELDS[1], &rollbacks)?;

        state.end()
    }
}

/// A deserializer for `Snapshot` that uses reflection.
pub struct SnapshotDeserializer<'a> {
    registry: &'a TypeRegistryInternal,
}

impl<'a> SnapshotDeserializer<'a> {
    /// Returns a new instance of `SnapshotDeserializer`.
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
        let capture = seq
            .next_element_seed(CaptureDeserializer::new(self.registry))?
            .ok_or_else(|| de::Error::missing_field(SNAPSHOT_FIELDS[0]))?;

        let rollbacks = seq
            .next_element_seed(RollbacksDeserializer::new(self.registry))?
            .ok_or_else(|| de::Error::missing_field(SNAPSHOT_FIELDS[1]))?;

        Ok(Self::Value { capture, rollbacks })
    }

    fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
    where
        V: MapAccess<'de>,
    {
        let mut capture = None;
        let mut rollbacks = None;

        while let Some(key) = map.next_key()? {
            match key {
                RollbacksFields::Snapshots => {
                    if capture.is_some() {
                        return Err(de::Error::duplicate_field(SNAPSHOT_FIELDS[0]));
                    }
                    capture = Some(map.next_value_seed(CaptureDeserializer::new(self.registry))?);
                }

                RollbacksFields::Active => {
                    if rollbacks.is_some() {
                        return Err(de::Error::duplicate_field(SNAPSHOT_FIELDS[1]));
                    }

                    rollbacks =
                        Some(map.next_value_seed(RollbacksDeserializer::new(self.registry))?);
                }
            }
        }

        let capture = capture.ok_or_else(|| de::Error::missing_field(SNAPSHOT_FIELDS[0]))?;
        let rollbacks = rollbacks.ok_or_else(|| de::Error::missing_field(SNAPSHOT_FIELDS[1]))?;

        Ok(Self::Value { capture, rollbacks })
    }
}
