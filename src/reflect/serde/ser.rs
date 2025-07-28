use bevy::reflect::{
    TypeRegistry,
    TypeRegistryArc,
    serde::TypedReflectSerializer,
};
use serde::{
    Serialize,
    Serializer,
};

use crate::prelude::*;

/// Owned serializer that handles serialization of a snapshot as a struct containing its entities and resources.
pub struct SnapshotSerializerArc<'a> {
    /// The snapshot to serialize.
    pub snapshot: &'a Snapshot,
    /// Type registry in which the components and resources types used in the snapshot are registered.
    pub registry: TypeRegistryArc,
}

impl Serialize for SnapshotSerializerArc<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SnapshotSerializer {
            snapshot: self.snapshot,
            registry: &self.registry.read(),
        }
        .serialize(serializer)
    }
}

/// Handles serialization of a snapshot as a struct containing its entities and resources.
pub struct SnapshotSerializer<'a> {
    /// The snapshot to serialize.
    pub snapshot: &'a Snapshot,
    /// Type registry in which the components and resources types used in the snapshot are registered.
    pub registry: &'a TypeRegistry,
}

impl<'a> SnapshotSerializer<'a> {
    /// Creates a snapshot serializer.
    pub fn new(snapshot: &'a Snapshot, registry: &'a TypeRegistry) -> Self {
        SnapshotSerializer { snapshot, registry }
    }
}

impl Serialize for SnapshotSerializer<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        TypedReflectSerializer::new(self.snapshot, self.registry).serialize(serializer)
    }
}
