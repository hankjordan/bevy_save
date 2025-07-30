//! Reflection-based snapshots

mod clone;
pub mod migration;
pub mod pipeline;
pub mod prefab;
pub mod remote;
pub mod serde;
pub mod snapshot;

#[cfg(feature = "checkpoints")]
pub mod checkpoint;

#[doc(inline)]
pub use self::{
    clone::{
        CloneReflect,
        clone_reflect_value,
    },
    migration::{
        Migrate,
        Migrator,
        ReflectMigrate,
        SnapshotVersion,
    },
    pipeline::Pipeline,
    prefab::{
        CommandsPrefabExt,
        Prefab,
        WithPrefab,
    },
    remote::{
        BoxedPartialReflect,
        DynamicEntity,
        EntityMap,
        ReflectMap,
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
