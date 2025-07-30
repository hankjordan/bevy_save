//! `serde` serialization and deserialization implementation for snapshots and checkpoints.

mod de;
mod ser;

pub use self::{
    de::{
        EntityMapDeserializer,
        ReflectMapDeserializer,
        SnapshotDeserializer,
        SnapshotDeserializerArc,
    },
    ser::{
        EntityMapSerializer,
        ReflectMapSerializer,
        SnapshotSerializer,
        SnapshotSerializerArc,
    },
};

/// Name of the serialized entity struct type.
pub const ENTITY_STRUCT: &str = "Entity";
/// Name of the serialized component field in an entity struct.
pub const ENTITY_FIELD_COMPONENTS: &str = "components";
