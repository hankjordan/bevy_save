use bevy::{
    reflect::{
        TypeRegistry,
        TypeRegistryArc,
    },
    scene::serde::{
        EntitiesSerializer,
        SceneMapSerializer,
    },
};
use serde::{
    Serialize,
    Serializer,
    ser::SerializeStruct,
};

use crate::{
    prelude::*,
    reflect::serde::{
        SNAPSHOT_ENTITIES,
        SNAPSHOT_RESOURCES,
        SNAPSHOT_STRUCT,
    },
};

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
        let mut state = serializer.serialize_struct(
            SNAPSHOT_STRUCT,
            #[cfg(feature = "checkpoints")]
            if self.snapshot.checkpoints.is_some() {
                3
            } else {
                2
            },
            #[cfg(not(feature = "checkpoints"))]
            2,
        )?;
        state.serialize_field(SNAPSHOT_ENTITIES, &EntitiesSerializer {
            entities: &self.snapshot.entities,
            registry: self.registry,
        })?;
        state.serialize_field(SNAPSHOT_RESOURCES, &SceneMapSerializer {
            entries: &self.snapshot.resources,
            registry: self.registry,
        })?;

        #[cfg(feature = "checkpoints")]
        if let Some(checkpoints) = &self.snapshot.checkpoints {
            state.serialize_field(
                super::SNAPSHOT_CHECKPOINTS,
                &checkpoints::CheckpointsSerializer {
                    checkpoints,
                    registry: self.registry,
                },
            )?;
        }

        state.end()
    }
}

#[cfg(feature = "checkpoints")]
mod checkpoints {
    use bevy::reflect::TypeRegistry;
    use serde::{
        Serialize,
        Serializer,
        ser::{
            SerializeSeq,
            SerializeStruct,
        },
    };

    use crate::reflect::{
        Snapshot,
        checkpoint::Checkpoints,
        serde::{
            CHECKPOINTS_ACTIVE,
            CHECKPOINTS_SNAPSHOTS,
            CHECKPOINTS_STRUCT,
        },
    };

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
                seq.serialize_element(&super::SnapshotSerializer {
                    snapshot,
                    registry: self.registry,
                })?;
            }

            seq.end()
        }
    }
}

#[cfg(feature = "checkpoints")]
pub use checkpoints::*;
