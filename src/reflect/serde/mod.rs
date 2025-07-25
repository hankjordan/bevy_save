//! `serde` serialization and deserialization implementation for snapshots and checkpoints.

mod de;
mod ser;

#[cfg(feature = "checkpoints")]
pub use self::{
    de::CheckpointsDeserializer,
    ser::CheckpointsSerializer,
};
pub use self::{
    de::{
        SnapshotDeserializer,
        SnapshotDeserializerArc,
    },
    ser::{
        SnapshotSerializer,
        SnapshotSerializerArc,
    },
};

pub(super) const SNAPSHOT_STRUCT: &str = "Snapshot";
pub(super) const SNAPSHOT_ENTITIES: &str = "entities";
pub(super) const SNAPSHOT_RESOURCES: &str = "resources";
pub(super) const SNAPSHOT_CHECKPOINTS: &str = "rollbacks";

#[cfg(feature = "checkpoints")]
mod checkpoints {
    pub(super) const CHECKPOINTS_STRUCT: &str = "Rollbacks";
    pub(super) const CHECKPOINTS_SNAPSHOTS: &str = "checkpoints";
    pub(super) const CHECKPOINTS_ACTIVE: &str = "active";
}

#[cfg(feature = "checkpoints")]
use checkpoints::{
    CHECKPOINTS_ACTIVE,
    CHECKPOINTS_SNAPSHOTS,
    CHECKPOINTS_STRUCT,
};

pub(super) const ENTITY_STRUCT: &str = "Entity";
pub(super) const ENTITY_COMPONENTS: &str = "components";
