use bevy::{
    reflect::TypeRegistry,
    scene::serde::{
        EntitiesSerializer,
        SceneMapSerializer,
    },
};
use serde::{
    ser::{
        SerializeSeq,
        SerializeStruct,
    },
    Serialize,
    Serializer,
};

use crate::prelude::*;

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
        let mut state = serializer.serialize_struct(
            SNAPSHOT_STRUCT,
            if self.snapshot.rollbacks.is_some() {
                3
            } else {
                2
            },
        )?;
        state.serialize_field(SNAPSHOT_ENTITIES, &EntitiesSerializer {
            entities: &self.snapshot.entities,
            registry: self.registry,
        })?;
        state.serialize_field(SNAPSHOT_RESOURCES, &SceneMapSerializer {
            entries: &self.snapshot.resources,
            registry: self.registry,
        })?;

        if let Some(rollbacks) = &self.snapshot.rollbacks {
            state.serialize_field(SNAPSHOT_ROLLBACKS, &RollbacksSerializer {
                rollbacks,
                registry: self.registry,
            })?;
        }

        state.end()
    }
}

struct SnapshotListSerializer<'a> {
    snapshots: &'a [Snapshot],
    registry: &'a TypeRegistry,
}

impl Serialize for SnapshotListSerializer<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.snapshots.len()))?;

        for snapshot in self.snapshots {
            seq.serialize_element(&SnapshotSerializer {
                snapshot,
                registry: self.registry,
            })?;
        }

        seq.end()
    }
}

/// Handles serialization of the global rollbacks store.
pub struct RollbacksSerializer<'a> {
    /// The rollbacks to serialize.
    pub rollbacks: &'a Rollbacks,
    /// Type registry in which the components and resources types used in the rollbacks are registered.
    pub registry: &'a TypeRegistry,
}

impl Serialize for RollbacksSerializer<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct(ROLLBACKS_STRUCT, 2)?;

        state.serialize_field(ROLLBACKS_CHECKPOINTS, &SnapshotListSerializer {
            snapshots: &self.rollbacks.checkpoints,
            registry: self.registry,
        })?;
        state.serialize_field(ROLLBACKS_ACTIVE, &self.rollbacks.active)?;

        state.end()
    }
}
