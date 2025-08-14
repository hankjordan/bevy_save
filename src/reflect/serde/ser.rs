use bevy::reflect::{
    PartialReflect,
    TypeRegistry,
    TypeRegistryArc,
    serde::TypedReflectSerializer,
};
use serde::{
    Serialize,
    Serializer,
    ser::{
        SerializeMap,
        SerializeStruct,
    },
};

use crate::{
    prelude::*,
    reflect::{
        DynamicEntity,
        EntityMap,
        ReflectMap,
        migration::ReflectMigrate,
        serde::{
            ENTITY_FIELD_COMPONENTS,
            ENTITY_STRUCT,
        },
    },
};

/// Owned serializer that handles serialization of a snapshot as a struct
/// containing its entities and resources.
pub struct SnapshotSerializerArc<'a> {
    snapshot: &'a Snapshot,
    registry: TypeRegistryArc,
}

impl<'a> SnapshotSerializerArc<'a> {
    /// Creates a snapshot serializer.
    pub fn new(snapshot: &'a Snapshot, registry: TypeRegistryArc) -> Self {
        Self { snapshot, registry }
    }
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
    snapshot: &'a Snapshot,
    registry: &'a TypeRegistry,
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
        TypedReflectSerializer::new(self.snapshot, self.registry).serialize(serializer)
    }
}

/// Handles serialization of multiple entities as a map of entity id to serialized entity.
pub struct EntityMapSerializer<'a> {
    entities: &'a EntityMap,
    registry: &'a TypeRegistry,
}

impl<'a> EntityMapSerializer<'a> {
    /// Creates a new [`EntityMapSerializer`] from the given [`EntityMap`] and [`TypeRegistry`].
    pub fn new(entities: &'a EntityMap, registry: &'a TypeRegistry) -> Self {
        Self { entities, registry }
    }
}

impl Serialize for EntityMapSerializer<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_map(Some(self.entities.len()))?;
        for entity in self.entities.iter() {
            state.serialize_entry(&entity.entity, &EntitySerializer {
                entity,
                registry: self.registry,
            })?;
        }
        state.end()
    }
}

struct EntitySerializer<'a> {
    entity: &'a DynamicEntity,
    registry: &'a TypeRegistry,
}

impl Serialize for EntitySerializer<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct(ENTITY_STRUCT, 1)?;
        state.serialize_field(ENTITY_FIELD_COMPONENTS, &ReflectMapSerializer {
            entries: &self.entity.components,
            registry: self.registry,
        })?;
        state.end()
    }
}

/// Handles serializing a list of values with a unique type as a map of type to value.
///
/// Note: The entries are sorted by type path before they're serialized.
pub struct ReflectMapSerializer<'a> {
    entries: &'a ReflectMap,
    registry: &'a TypeRegistry,
}

impl<'a> ReflectMapSerializer<'a> {
    /// Creates a new [`ReflectMapSerializer`] from the given entries and [`TypeRegistry`].
    ///
    /// Automatically handles registered migrations.
    pub fn new(entries: &'a ReflectMap, registry: &'a TypeRegistry) -> Self {
        Self { entries, registry }
    }
}

impl Serialize for ReflectMapSerializer<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_map(Some(self.entries.len()))?;
        let sorted = {
            let mut entries = self
                .entries
                .iter()
                .map(|entry| {
                    let info = entry.get_represented_type_info().unwrap();

                    (
                        info.type_path(),
                        self.registry
                            .get(info.type_id())
                            .and_then(|r| r.data::<ReflectMigrate>())
                            .and_then(|m| m.version()),
                        entry,
                    )
                })
                .collect::<Vec<_>>();
            entries.sort_by_key(|(type_path, _, _)| *type_path);
            entries
        };

        for (type_path, version, value) in sorted {
            state.serialize_entry(
                &if let Some(version) = version {
                    format!("{type_path} {version}")
                } else {
                    type_path.to_string()
                },
                &TypedReflectSerializer::new(value, self.registry),
            )?;
        }
        state.end()
    }
}
