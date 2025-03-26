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

use crate::{
    checkpoint::Checkpoints,
    prelude::*,
    serde::{
        CHECKPOINTS_ACTIVE,
        CHECKPOINTS_SNAPSHOTS,
        CHECKPOINTS_STRUCT,
        SNAPSHOT_CHECKPOINTS,
        SNAPSHOT_ENTITIES,
        SNAPSHOT_RESOURCES,
        SNAPSHOT_STRUCT,
    },
};

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
            if self.snapshot.checkpoints.is_some() {
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

        if let Some(checkpoints) = &self.snapshot.checkpoints {
            state.serialize_field(SNAPSHOT_CHECKPOINTS, &CheckpointsSerializer {
                checkpoints,
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

/// Handles serialization of the checkpoints store.
pub struct CheckpointsSerializer<'a> {
    /// The checkpoints to serialize.
    pub checkpoints: &'a Checkpoints,
    /// Type registry in which the components and resources types used in the checkpoints are registered.
    pub registry: &'a TypeRegistry,
}

impl Serialize for CheckpointsSerializer<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct(CHECKPOINTS_STRUCT, 2)?;

        state.serialize_field(CHECKPOINTS_SNAPSHOTS, &SnapshotListSerializer {
            snapshots: &self.checkpoints.snapshots,
            registry: self.registry,
        })?;
        state.serialize_field(CHECKPOINTS_ACTIVE, &self.checkpoints.active)?;

        state.end()
    }
}
