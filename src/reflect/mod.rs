//! Reflection-based snapshots

mod clone;
pub mod pipeline;
pub mod prefab;
pub mod serde;
pub mod snapshot;

#[cfg(feature = "checkpoints")]
pub mod checkpoint;

pub use self::{
    clone::{
        CloneReflect,
        clone_reflect_value,
    },
    pipeline::Pipeline,
    prefab::{
        CommandsPrefabExt,
        Prefab,
        WithPrefab,
    },
    serde::{
        SnapshotDeserializer,
        SnapshotDeserializerArc,
        SnapshotSerializer,
        SnapshotSerializerArc,
    },
    snapshot::{
        Applier,
        ApplierRef,
        BoxedHook,
        Builder,
        BuilderRef,
        Hook,
        Snapshot,
    },
};
