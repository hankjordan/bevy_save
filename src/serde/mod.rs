//! `serde` serialization and deserialization implementation for snapshots and checkpoints.

mod de;
mod ser;

pub use self::{
    de::{
        CheckpointsDeserializer,
        SnapshotDeserializer,
    },
    ser::{
        CheckpointsSerializer,
        SnapshotSerializer,
    },
};

pub(super) const SNAPSHOT_STRUCT: &str = "Snapshot";
pub(super) const SNAPSHOT_ENTITIES: &str = "entities";
pub(super) const SNAPSHOT_RESOURCES: &str = "resources";
pub(super) const SNAPSHOT_CHECKPOINTS: &str = "rollbacks";

pub(super) const CHECKPOINTS_STRUCT: &str = "Rollbacks";
pub(super) const CHECKPOINTS_SNAPSHOTS: &str = "checkpoints";
pub(super) const CHECKPOINTS_ACTIVE: &str = "active";

pub(super) const ENTITY_STRUCT: &str = "Entity";
pub(super) const ENTITY_COMPONENTS: &str = "components";
