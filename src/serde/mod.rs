mod de;
mod ser;

pub use de::{
    RollbacksDeserializer,
    SnapshotDeserializer,
};
pub use ser::{
    RollbacksSerializer,
    SnapshotSerializer,
};

pub(super) const SNAPSHOT_STRUCT: &str = "Snapshot";
pub(super) const SNAPSHOT_ENTITIES: &str = "entities";
pub(super) const SNAPSHOT_RESOURCES: &str = "resources";
pub(super) const SNAPSHOT_ROLLBACKS: &str = "rollbacks";

pub(super) const ROLLBACKS_STRUCT: &str = "Rollbacks";
pub(super) const ROLLBACKS_CHECKPOINTS: &str = "checkpoints";
pub(super) const ROLLBACKS_ACTIVE: &str = "active";

pub(super) const ENTITY_STRUCT: &str = "Entity";
pub(super) const ENTITY_COMPONENTS: &str = "components";
