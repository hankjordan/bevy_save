use bevy::{
    prelude::*,
    reflect::{
        GetTypeRegistration,
        TypeRegistry,
        TypeRegistryArc,
        serde::TypedReflectDeserializer,
    },
};
use serde::{
    Deserializer,
    de::{
        DeserializeSeed,
        Error,
    },
};

use crate::{
    prelude::*,
    reflect::{
        backcompat::v0_16::SnapshotV0_16, checkpoint::Checkpoints, Version
    },
};

/// Owned deserializer that handles snapshot deserialization.
pub struct SnapshotDeserializerArc {
    registry: TypeRegistryArc,
    version: Version,
}

impl SnapshotDeserializerArc {
    /// Creates a new [`SnapshotDeserializerArc`] from the given [`TypeRegistryArc`]
    pub fn new(registry: TypeRegistryArc) -> Self {
        Self {
            registry,
            version: Version::default(),
        }
    }

    /// Sets the snapshot [`Version`] for back-compat
    pub fn version(mut self, version: Version) -> Self {
        self.version = version;
        self
    }
}

impl<'de> DeserializeSeed<'de> for SnapshotDeserializerArc {
    type Value = Snapshot;

    fn deserialize<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        SnapshotDeserializer {
            registry: &self.registry.read(),
            version: self.version,
        }
        .deserialize(deserializer)
    }
}

/// Handles snapshot deserialization.
pub struct SnapshotDeserializer<'a> {
    registry: &'a TypeRegistry,
    version: Version,
}

impl<'a> SnapshotDeserializer<'a> {
    /// Creates a new [`SnapshotDeserializerArc`] from the given [`TypeRegistryArc`]
    pub fn new(registry: &'a TypeRegistry) -> Self {
        Self {
            registry,
            version: Version::default(),
        }
    }

    /// Sets the snapshot [`Version`] for back-compat
    pub fn version(mut self, version: Version) -> Self {
        self.version = version;
        self
    }
}

impl<'de> DeserializeSeed<'de> for SnapshotDeserializer<'_> {
    type Value = Snapshot;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let reg = match self.version {
            Version::V0_16 => SnapshotV0_16::get_type_registration(),
            Version::V0_20 => Snapshot::get_type_registration(),
        };

        TypedReflectDeserializer::new(&reg, self.registry)
            .deserialize(deserializer)
            .and_then(|v| match self.version {
                Version::V0_16 => {
                    let old = SnapshotV0_16::from_reflect(&*v)
                        .ok_or_else(|| Error::custom("FromReflect failed for Snapshot (v0.16)"))?;

                    let mut new = Snapshot {
                        entities: old.entities,
                        resources: old.resources,
                    };

                    if let Some(rollbacks) = old.rollbacks {
                        new.resources.0.push(
                            Box::new(Checkpoints {
                                snapshots: rollbacks
                                    .checkpoints
                                    .into_iter()
                                    .map(|c| Snapshot {
                                        entities: c.entities,
                                        resources: c.resources,
                                    })
                                    .collect(),
                                active: rollbacks.active,
                            })
                            .into_partial_reflect()
                            .into(),
                        );
                    }

                    Ok(new)
                }
                Version::V0_20 => Snapshot::from_reflect(&*v)
                    .ok_or_else(|| Error::custom("FromReflect failed for Snapshot")),
            })
    }
}
