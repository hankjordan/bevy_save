use std::{
    fmt::Formatter,
    str::FromStr,
};

use bevy::{
    prelude::*,
    reflect::{
        GetTypeRegistration,
        TypeRegistration,
        TypeRegistry,
        TypeRegistryArc,
        serde::{
            ReflectDeserializer,
            TypedReflectDeserializer,
        },
    },
};
use serde::{
    Deserialize,
    Deserializer,
    de::{
        DeserializeSeed,
        Error,
        MapAccess,
        SeqAccess,
        Visitor,
    },
};

use crate::{
    prelude::*,
    reflect::{
        DynamicEntity,
        EntityMap,
        ReflectMap,
        checkpoint::Checkpoints,
        migration::{
            ReflectMigrate,
            SnapshotVersion,
            backcompat::v0_16::SnapshotV0_16,
        },
        serde::{
            ENTITY_FIELD_COMPONENTS,
            ENTITY_STRUCT,
        },
    },
};

/// Owned deserializer that handles snapshot deserialization.
pub struct SnapshotDeserializerArc {
    registry: TypeRegistryArc,
    version: SnapshotVersion,
}

impl SnapshotDeserializerArc {
    /// Creates a new [`SnapshotDeserializerArc`] from the given [`TypeRegistryArc`].
    pub fn new(registry: TypeRegistryArc) -> Self {
        Self {
            registry,
            version: SnapshotVersion::default(),
        }
    }

    /// Sets the [`SnapshotVersion`] for backwards compatibility.
    pub fn version(mut self, version: SnapshotVersion) -> Self {
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
    version: SnapshotVersion,
}

impl<'a> SnapshotDeserializer<'a> {
    /// Creates a new [`SnapshotDeserializerArc`] from the given [`TypeRegistryArc`].
    pub fn new(registry: &'a TypeRegistry) -> Self {
        Self {
            registry,
            version: SnapshotVersion::default(),
        }
    }

    /// Sets the [`SnapshotVersion`] for backwards compatibility.
    pub fn version(mut self, version: SnapshotVersion) -> Self {
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
            SnapshotVersion::V0_16 => SnapshotV0_16::get_type_registration(),
            SnapshotVersion::V1_0 => Snapshot::get_type_registration(),
        };

        TypedReflectDeserializer::new(&reg, self.registry)
            .deserialize(deserializer)
            .and_then(|v| match self.version {
                SnapshotVersion::V0_16 => {
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
                SnapshotVersion::V1_0 => Snapshot::from_reflect(&*v)
                    .ok_or_else(|| Error::custom("FromReflect failed for Snapshot")),
            })
    }
}

/// Handles deserialization for a collection of entities.
pub struct EntityMapDeserializer<'a> {
    registry: &'a TypeRegistry,
}

impl<'a> EntityMapDeserializer<'a> {
    /// Creates a new [`EntityMapDeserializer`] from the given [`TypeRegistry`].
    pub fn new(registry: &'a TypeRegistry) -> Self {
        Self { registry }
    }
}

impl<'de> DeserializeSeed<'de> for EntityMapDeserializer<'_> {
    type Value = EntityMap;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(EntityMapVisitor {
            registry: self.registry,
        })
    }
}

struct EntityMapVisitor<'a> {
    registry: &'a TypeRegistry,
}

impl<'de> Visitor<'de> for EntityMapVisitor<'_> {
    type Value = EntityMap;

    fn expecting(&self, formatter: &mut Formatter) -> core::fmt::Result {
        formatter.write_str("map of entities")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut entities = Vec::new();
        while let Some(entity) = map.next_key::<Entity>()? {
            let entity = map.next_value_seed(EntityDeserializer {
                entity,
                registry: self.registry,
            })?;
            entities.push(entity);
        }

        Ok(EntityMap(entities))
    }
}

/// Handle deserialization of an entity and its components.
struct EntityDeserializer<'a> {
    entity: Entity,
    registry: &'a TypeRegistry,
}

impl<'de> DeserializeSeed<'de> for EntityDeserializer<'_> {
    type Value = DynamicEntity;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct(ENTITY_STRUCT, &[ENTITY_FIELD_COMPONENTS], EntityVisitor {
            entity: self.entity,
            registry: self.registry,
        })
    }
}

struct EntityVisitor<'a> {
    entity: Entity,
    registry: &'a TypeRegistry,
}

impl<'de> Visitor<'de> for EntityVisitor<'_> {
    type Value = DynamicEntity;

    fn expecting(&self, formatter: &mut Formatter) -> core::fmt::Result {
        formatter.write_str("entities")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let components = seq
            .next_element_seed(ReflectMapDeserializer {
                registry: self.registry,
            })?
            .ok_or_else(|| Error::missing_field(ENTITY_FIELD_COMPONENTS))?;

        Ok(DynamicEntity {
            entity: self.entity,
            components,
        })
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum EntityField {
            Components,
        }

        let mut components = None;
        while let Some(key) = map.next_key()? {
            match key {
                EntityField::Components => {
                    if components.is_some() {
                        return Err(Error::duplicate_field(ENTITY_FIELD_COMPONENTS));
                    }

                    components = Some(map.next_value_seed(ReflectMapDeserializer {
                        registry: self.registry,
                    })?);
                }
            }
        }

        let components = components
            .take()
            .ok_or_else(|| Error::missing_field(ENTITY_FIELD_COMPONENTS))?;
        Ok(DynamicEntity {
            entity: self.entity,
            components,
        })
    }
}

/// Handles deserialization of a sequence of values with unique types.
pub struct ReflectMapDeserializer<'a> {
    registry: &'a TypeRegistry,
}

impl<'a> ReflectMapDeserializer<'a> {
    /// Creates a new [`ReflectMapDeserializer`] from the given [`TypeRegistry`].
    ///
    /// Automatically handles registered migrations.
    pub fn new(registry: &'a TypeRegistry) -> Self {
        Self { registry }
    }
}

impl<'de> DeserializeSeed<'de> for ReflectMapDeserializer<'_> {
    type Value = ReflectMap;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(ReflectMapVisitor {
            registry: self.registry,
        })
    }
}

struct ReflectMapVisitor<'a> {
    registry: &'a TypeRegistry,
}

impl<'de> Visitor<'de> for ReflectMapVisitor<'_> {
    type Value = ReflectMap;

    fn expecting(&self, formatter: &mut Formatter) -> core::fmt::Result {
        formatter.write_str("map of reflect types")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut dynamic_properties = Vec::new();
        while let Some(entity) = seq.next_element_seed(ReflectDeserializer::new(self.registry))? {
            dynamic_properties.push(entity);
        }

        Ok(dynamic_properties.into())
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut entries = Vec::new();
        while let Some(registration) =
            map.next_key_seed(TypeRegistrationDeserializer::new(self.registry))?
        {
            let value = map.next_value_seed(TypedReflectDeserializer::new(
                registration.input(),
                self.registry,
            ))?;

            match registration {
                TypeRegistrationVersioned::Unversioned(registration) => {
                    // Attempt to convert using FromReflect.
                    let value = registration
                        .data::<ReflectFromReflect>()
                        .and_then(|fr| fr.from_reflect(value.as_partial_reflect()))
                        .map(PartialReflect::into_partial_reflect)
                        .unwrap_or(value);

                    entries.push(value);
                }
                TypeRegistrationVersioned::Versioned {
                    version, output, ..
                } => {
                    // Attempt to convert using Migrate.
                    let value = output
                        .data::<ReflectMigrate>()
                        .and_then(|m| m.migrate(&*value, version.to_string()))
                        .map(PartialReflect::into_partial_reflect)
                        .unwrap_or(value);

                    entries.push(value);
                }
            }
        }

        Ok(entries.into())
    }
}

struct TypeRegistrationDeserializer<'a> {
    registry: &'a TypeRegistry,
}

impl<'a> TypeRegistrationDeserializer<'a> {
    pub fn new(registry: &'a TypeRegistry) -> Self {
        Self { registry }
    }
}

enum TypeRegistrationVersioned<'a> {
    Unversioned(&'a TypeRegistration),
    Versioned {
        version: semver::Version,
        input: TypeRegistration,
        output: &'a TypeRegistration,
    },
}

impl TypeRegistrationVersioned<'_> {
    pub fn input(&self) -> &TypeRegistration {
        match self {
            TypeRegistrationVersioned::Unversioned(r) => r,
            TypeRegistrationVersioned::Versioned { input, .. } => input,
        }
    }
}

impl<'a, 'de> DeserializeSeed<'de> for TypeRegistrationDeserializer<'a> {
    type Value = TypeRegistrationVersioned<'a>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct TypeRegistrationVisitor<'a>(&'a TypeRegistry);

        impl<'a> Visitor<'_> for TypeRegistrationVisitor<'a> {
            type Value = TypeRegistrationVersioned<'a>;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("string containing `type` entry for the reflected value")
            }

            fn visit_str<E>(self, type_path: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                if let Some((type_path, version)) = type_path.split_once(' ') {
                    let version = semver::Version::from_str(version)
                        .map_err(|_| Error::custom(format_args!("invalid version `{version}`")))?;

                    let output = self
                        .0
                        .get_with_type_path(type_path)
                        .or_else(|| {
                            self.0
                                .iter_with_data::<ReflectMigrate>()
                                .find(|(_, m)| m.matches(type_path))
                                .map(|(r, _)| r)
                        })
                        .ok_or_else(|| {
                            Error::custom(format_args!("no registration found for `{type_path}`"))
                        })?;

                    let migrate = output.data::<ReflectMigrate>().ok_or_else(|| {
                        Error::custom(format_args!(
                            "`ReflectMigrate` not registered for `{type_path}`"
                        ))
                    })?;

                    let input = migrate.registration(version.to_string()).ok_or_else(|| {
                        Error::custom(format_args!(
                            "no migration found for `{type_path}` -> `{}` with version `{version}`",
                            output.type_info().type_path()
                        ))
                    })?;

                    Ok(TypeRegistrationVersioned::Versioned {
                        version,
                        input,
                        output,
                    })
                } else {
                    let registration = self.0.get_with_type_path(type_path).ok_or_else(|| {
                        Error::custom(format_args!("no registration found for `{type_path}`"))
                    })?;

                    Ok(TypeRegistrationVersioned::Unversioned(registration))
                }
            }
        }

        deserializer.deserialize_str(TypeRegistrationVisitor(self.registry))
    }
}
