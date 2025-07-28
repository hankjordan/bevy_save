//! `serde` serialization and deserialization implementation for snapshots and checkpoints.

mod de;
mod ser;

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
