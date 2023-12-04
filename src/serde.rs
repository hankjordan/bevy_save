use std::{
    fmt::Formatter,
    marker::PhantomData,
};

use bevy::{
    ecs::entity::Entity,
    reflect::{
        serde::{
            TypeRegistrationDeserializer,
            TypedReflectDeserializer,
            TypedReflectSerializer,
            UntypedReflectDeserializer,
        },
        Reflect,
        TypeRegistry,
        TypeRegistryArc,
    },
    scene::DynamicEntity,
    utils::HashSet,
};
use serde::{
    de::{
        DeserializeSeed,
        Error,
        MapAccess,
        SeqAccess,
        Visitor,
    },
    ser::{
        SerializeMap,
        SerializeSeq,
        SerializeStruct,
    },
    Deserialize,
    Deserializer,
    Serialize,
    Serializer,
};

use crate::{
    extract::{
        ExtractComponent,
        ExtractResource,
    },
    Components,
    Entities,
    Resources,
    Rollbacks,
    Snapshot,
    Snapshot2,
};

const SNAPSHOT_STRUCT: &str = "Snapshot";
const SNAPSHOT_ENTITIES: &str = "entities";
const SNAPSHOT_RESOURCES: &str = "resources";
const SNAPSHOT_ROLLBACKS: &str = "rollbacks";

const ROLLBACKS_STRUCT: &str = "Rollbacks";
const ROLLBACKS_CHECKPOINTS: &str = "checkpoints";
const ROLLBACKS_ACTIVE: &str = "active";

const ENTITY_STRUCT: &str = "Entity";
const ENTITY_COMPONENTS: &str = "components";

/// Handles serialization of a snapshot as a struct containing its entities and resources.
pub struct SnapshotSerializer<'a> {
    /// The snapshot to serialize.
    pub snapshot: &'a Snapshot,
    /// Type registry in which the components and resources types used in the snapshot are registered.
    pub registry: &'a TypeRegistryArc,
}

impl<'a> SnapshotSerializer<'a> {
    /// Creates a snapshot serializer.
    pub fn new(snapshot: &'a Snapshot, registry: &'a TypeRegistryArc) -> Self {
        SnapshotSerializer { snapshot, registry }
    }
}

impl<'a> Serialize for SnapshotSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct(
            SNAPSHOT_STRUCT,
            if self.snapshot.rollbacks.is_some() {
                3
            } else {
                2
            },
        )?;
        state.serialize_field(SNAPSHOT_ENTITIES, &EntityMapSerializer {
            entities: &self.snapshot.entities,
            registry: self.registry,
        })?;
        state.serialize_field(SNAPSHOT_RESOURCES, &ReflectMapSerializer {
            entries: &self.snapshot.resources,
            registry: self.registry,
        })?;

        if let Some(rollbacks) = &self.snapshot.rollbacks {
            state.serialize_field(SNAPSHOT_ROLLBACKS, &RollbacksSerializer {
                rollbacks,
                registry: self.registry,
            })?;
        }

        state.end()
    }
}

struct SnapshotListSerializer<'a> {
    snapshots: &'a [Snapshot],
    registry: &'a TypeRegistryArc,
}

impl<'a> Serialize for SnapshotListSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.snapshots.len()))?;

        for snapshot in self.snapshots {
            seq.serialize_element(&SnapshotSerializer {
                snapshot,
                registry: self.registry,
            })?;
        }

        seq.end()
    }
}

/// Handles serialization of the global rollbacks store.
pub struct RollbacksSerializer<'a> {
    /// The rollbacks to serialize.
    pub rollbacks: &'a Rollbacks,
    /// Type registry in which the components and resources types used in the rollbacks are registered.
    pub registry: &'a TypeRegistryArc,
}

impl<'a> Serialize for RollbacksSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct(ROLLBACKS_STRUCT, 2)?;

        state.serialize_field(ROLLBACKS_CHECKPOINTS, &SnapshotListSerializer {
            snapshots: &self.rollbacks.checkpoints,
            registry: self.registry,
        })?;
        state.serialize_field(ROLLBACKS_ACTIVE, &self.rollbacks.active)?;

        state.end()
    }
}

struct EntityMapSerializer<'a> {
    entities: &'a [DynamicEntity],
    registry: &'a TypeRegistryArc,
}

impl<'a> Serialize for EntityMapSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_map(Some(self.entities.len()))?;
        for entity in self.entities {
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
    registry: &'a TypeRegistryArc,
}

impl<'a> Serialize for EntitySerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct(ENTITY_STRUCT, 1)?;
        state.serialize_field(ENTITY_COMPONENTS, &ReflectMapSerializer {
            entries: &self.entity.components,
            registry: self.registry,
        })?;
        state.end()
    }
}

struct ReflectMapSerializer<'a> {
    entries: &'a [Box<dyn Reflect>],
    registry: &'a TypeRegistryArc,
}

impl<'a> Serialize for ReflectMapSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_map(Some(self.entries.len()))?;
        for reflect in self.entries {
            state.serialize_entry(
                reflect.get_represented_type_info().unwrap().type_path(),
                &TypedReflectSerializer::new(&**reflect, &self.registry.read()),
            )?;
        }
        state.end()
    }
}

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "lowercase")]
enum SnapshotField {
    Entities,
    Resources,
    Rollbacks,
}

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "lowercase")]
enum RollbacksField {
    Checkpoints,
    Active,
}

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "lowercase")]
enum EntityField {
    Components,
}

/// Handles snapshot deserialization.
pub struct SnapshotDeserializer<'a> {
    /// Type registry in which the components and resources types used in the snapshot to deserialize are registered.
    pub registry: &'a TypeRegistry,
}

impl<'a, 'de> DeserializeSeed<'de> for SnapshotDeserializer<'a> {
    type Value = Snapshot;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_struct(
            SNAPSHOT_STRUCT,
            &[SNAPSHOT_ENTITIES, SNAPSHOT_RESOURCES, SNAPSHOT_ROLLBACKS],
            SnapshotVisitor {
                registry: self.registry,
            },
        )
    }
}

struct SnapshotVisitor<'a> {
    pub registry: &'a TypeRegistry,
}

impl<'a, 'de> Visitor<'de> for SnapshotVisitor<'a> {
    type Value = Snapshot;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("snapshot struct")
    }

    fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut entities = None;
        let mut resources = None;
        let mut rollbacks = None;

        while let Some(key) = map.next_key()? {
            match key {
                SnapshotField::Entities => {
                    if entities.is_some() {
                        return Err(Error::duplicate_field(SNAPSHOT_ENTITIES));
                    }
                    entities = Some(map.next_value_seed(EntityMapDeserializer {
                        registry: self.registry,
                    })?);
                }
                SnapshotField::Resources => {
                    if resources.is_some() {
                        return Err(Error::duplicate_field(SNAPSHOT_RESOURCES));
                    }
                    resources = Some(map.next_value_seed(ReflectMapDeserializer {
                        registry: self.registry,
                    })?);
                }
                SnapshotField::Rollbacks => {
                    if rollbacks.is_some() {
                        return Err(Error::duplicate_field(SNAPSHOT_ROLLBACKS));
                    }
                    rollbacks = Some(map.next_value_seed(RollbacksDeserializer {
                        registry: self.registry,
                    })?);
                }
            }
        }

        let entities = entities.ok_or_else(|| Error::missing_field(SNAPSHOT_ENTITIES))?;
        let resources = resources.ok_or_else(|| Error::missing_field(SNAPSHOT_RESOURCES))?;

        Ok(Snapshot {
            entities,
            resources,
            rollbacks,
        })
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let entities = seq
            .next_element_seed(EntityMapDeserializer {
                registry: self.registry,
            })?
            .ok_or_else(|| Error::missing_field(SNAPSHOT_ENTITIES))?;

        let resources = seq
            .next_element_seed(ReflectMapDeserializer {
                registry: self.registry,
            })?
            .ok_or_else(|| Error::missing_field(SNAPSHOT_RESOURCES))?;

        let rollbacks = seq.next_element_seed(RollbacksDeserializer {
            registry: self.registry,
        })?;

        Ok(Snapshot {
            entities,
            resources,
            rollbacks,
        })
    }
}

/// Handles rollbacks deserialization.
pub struct RollbacksDeserializer<'a> {
    /// Type registry in which the components and resources types used to deserialize the rollbacks are registered.
    pub registry: &'a TypeRegistry,
}

impl<'a, 'de> DeserializeSeed<'de> for RollbacksDeserializer<'a> {
    type Value = Rollbacks;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_struct(
            ROLLBACKS_STRUCT,
            &[ROLLBACKS_CHECKPOINTS, ROLLBACKS_ACTIVE],
            RollbacksVisitor {
                registry: self.registry,
            },
        )
    }
}

struct RollbacksVisitor<'a> {
    registry: &'a TypeRegistry,
}

impl<'a, 'de> Visitor<'de> for RollbacksVisitor<'a> {
    type Value = Rollbacks;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("rollbacks struct")
    }

    fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut checkpoints = None;
        let mut active = None;

        while let Some(key) = map.next_key()? {
            match key {
                RollbacksField::Checkpoints => {
                    if checkpoints.is_some() {
                        return Err(Error::duplicate_field(ROLLBACKS_CHECKPOINTS));
                    }
                    checkpoints = Some(map.next_value_seed(SnapshotListDeserializer {
                        registry: self.registry,
                    })?);
                }
                RollbacksField::Active => {
                    if active.is_some() {
                        return Err(Error::duplicate_field(ROLLBACKS_ACTIVE));
                    }
                    active = Some(map.next_value()?);
                }
            }
        }

        let checkpoints = checkpoints.ok_or_else(|| Error::missing_field(ROLLBACKS_CHECKPOINTS))?;
        let active = active.ok_or_else(|| Error::missing_field(ROLLBACKS_ACTIVE))?;

        Ok(Rollbacks {
            checkpoints,
            active,
        })
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let checkpoints = seq
            .next_element_seed(SnapshotListDeserializer {
                registry: self.registry,
            })?
            .ok_or_else(|| Error::missing_field(ROLLBACKS_CHECKPOINTS))?;

        let active = seq
            .next_element()?
            .ok_or_else(|| Error::missing_field(ROLLBACKS_ACTIVE))?;

        Ok(Rollbacks {
            checkpoints,
            active,
        })
    }
}

struct SnapshotListDeserializer<'a> {
    registry: &'a TypeRegistry,
}

impl<'a, 'de> DeserializeSeed<'de> for SnapshotListDeserializer<'a> {
    type Value = Vec<Snapshot>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(SnapshotListVisitor {
            registry: self.registry,
        })
    }
}

struct SnapshotListVisitor<'a> {
    registry: &'a TypeRegistry,
}

impl<'a, 'de> Visitor<'de> for SnapshotListVisitor<'a> {
    type Value = Vec<Snapshot>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("sequence of snapshots")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut result = Vec::new();

        while let Some(next) = seq.next_element_seed(SnapshotDeserializer {
            registry: self.registry,
        })? {
            result.push(next);
        }

        Ok(result)
    }
}

struct EntityMapDeserializer<'a> {
    registry: &'a TypeRegistry,
}

impl<'a, 'de> DeserializeSeed<'de> for EntityMapDeserializer<'a> {
    type Value = Vec<DynamicEntity>;

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

impl<'a, 'de> Visitor<'de> for EntityMapVisitor<'a> {
    type Value = Vec<DynamicEntity>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
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

        Ok(entities)
    }
}

struct EntityDeserializer<'a> {
    entity: Entity,
    registry: &'a TypeRegistry,
}

impl<'a, 'de> DeserializeSeed<'de> for EntityDeserializer<'a> {
    type Value = DynamicEntity;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_struct(ENTITY_STRUCT, &[ENTITY_COMPONENTS], EntityVisitor {
            entity: self.entity,
            registry: self.registry,
        })
    }
}

struct EntityVisitor<'a> {
    entity: Entity,
    registry: &'a TypeRegistry,
}

impl<'a, 'de> Visitor<'de> for EntityVisitor<'a> {
    type Value = DynamicEntity;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("entities")
    }

    fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let components = seq
            .next_element_seed(ReflectMapDeserializer {
                registry: self.registry,
            })?
            .ok_or_else(|| Error::missing_field(ENTITY_COMPONENTS))?;

        Ok(DynamicEntity {
            entity: self.entity,
            components,
        })
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut components = None;
        while let Some(key) = map.next_key()? {
            match key {
                EntityField::Components => {
                    if components.is_some() {
                        return Err(Error::duplicate_field(ENTITY_COMPONENTS));
                    }

                    components = Some(map.next_value_seed(ReflectMapDeserializer {
                        registry: self.registry,
                    })?);
                }
            }
        }

        let components = components
            .take()
            .ok_or_else(|| Error::missing_field(ENTITY_COMPONENTS))?;
        Ok(DynamicEntity {
            entity: self.entity,
            components,
        })
    }
}

struct ReflectMapDeserializer<'a> {
    registry: &'a TypeRegistry,
}

impl<'a, 'de> DeserializeSeed<'de> for ReflectMapDeserializer<'a> {
    type Value = Vec<Box<dyn Reflect>>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(ReflectMapVisitor {
            registry: self.registry,
        })
    }
}

struct ReflectMapVisitor<'a> {
    pub registry: &'a TypeRegistry,
}

impl<'a, 'de> Visitor<'de> for ReflectMapVisitor<'a> {
    type Value = Vec<Box<dyn Reflect>>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("map of reflect types")
    }

    fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut added = HashSet::new();
        let mut entries = Vec::new();
        while let Some(registration) =
            map.next_key_seed(TypeRegistrationDeserializer::new(self.registry))?
        {
            if !added.insert(registration.type_id()) {
                return Err(Error::custom(format_args!(
                    "duplicate reflect type: `{}`",
                    registration.type_info().type_path(),
                )));
            }

            entries.push(
                map.next_value_seed(TypedReflectDeserializer::new(registration, self.registry))?,
            );
        }

        Ok(entries)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut dynamic_properties = Vec::new();
        while let Some(entity) =
            seq.next_element_seed(UntypedReflectDeserializer::new(self.registry))?
        {
            dynamic_properties.push(entity);
        }

        Ok(dynamic_properties)
    }
}

// --------------------------------------------------------------------------------------------------------------------

impl<C: ExtractComponent, R: ExtractResource> Serialize for Snapshot2<C, R> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut ser = serializer.serialize_struct("Snapshot", 2)?;
        ser.serialize_field("entities", &self.entities)?;
        ser.serialize_field("resources", &self.resources)?;
        ser.end()
    }
}

impl<'de, C: ExtractComponent, R: ExtractResource> Deserialize<'de> for Snapshot2<C, R> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Fields {
            Entities,
            Resources,
        }

        struct SnapshotVisitor<C, R>(PhantomData<(C, R)>);

        impl<'de, C: ExtractComponent, R: ExtractResource> Visitor<'de> for SnapshotVisitor<C, R> {
            type Value = Snapshot2<C, R>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("snapshot struct")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let entities = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;

                let resources = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;

                Ok(Snapshot2 {
                    entities,
                    resources,
                })
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut entities = None;
                let mut resources = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Fields::Entities => {
                            if entities.is_some() {
                                return Err(serde::de::Error::duplicate_field("entities"));
                            }
                            entities = Some(map.next_value()?);
                        }
                        Fields::Resources => {
                            if resources.is_some() {
                                return Err(serde::de::Error::duplicate_field("resources"));
                            }
                            resources = Some(map.next_value()?);
                        }
                    }
                }

                let entities =
                    entities.ok_or_else(|| serde::de::Error::missing_field("entities"))?;
                let resources =
                    resources.ok_or_else(|| serde::de::Error::missing_field("resources"))?;

                Ok(Snapshot2 {
                    entities,
                    resources,
                })
            }
        }

        const FIELDS: &[&str] = &["entities", "resources"];
        deserializer.deserialize_struct("Snapshot", FIELDS, SnapshotVisitor(PhantomData))
    }
}

impl<C: ExtractComponent> Serialize for Entities<C> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;

        for (entity, components) in &self.0 {
            map.serialize_entry(entity, components)?;
        }

        map.end()
    }
}

impl<'de, C: ExtractComponent> Deserialize<'de> for Entities<C> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct EntitiesVisitor<C>(PhantomData<C>);

        impl<'de, C: ExtractComponent> Visitor<'de> for EntitiesVisitor<C> {
            type Value = Entities<C>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a sequence of extracted entities")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut entities = Vec::new();

                while let Some(entry) = map.next_entry()? {
                    entities.push(entry);
                }

                Ok(Entities(entities))
            }
        }

        deserializer.deserialize_map(EntitiesVisitor(PhantomData))
    }
}

impl<C: ExtractComponent> Serialize for Components<C> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(None)?;
        C::serialize(&self.0, &mut seq)?;
        seq.end()
    }
}

impl<'de, C: ExtractComponent> Deserialize<'de> for Components<C> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ComponentsVisitor<C>(PhantomData<C>);

        impl<'de, C: ExtractComponent> Visitor<'de> for ComponentsVisitor<C> {
            type Value = Components<C>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a sequence of extracted components")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                Ok(Components(C::deserialize(&mut seq)?))
            }
        }

        deserializer.deserialize_seq(ComponentsVisitor(PhantomData))
    }
}

impl<R: ExtractResource> Serialize for Resources<R> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(None)?;
        R::serialize(&self.0, &mut seq)?;
        seq.end()
    }
}

impl<'de, R: ExtractResource> Deserialize<'de> for Resources<R> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ResourcesVisitor<R>(PhantomData<R>);

        impl<'de, R: ExtractResource> Visitor<'de> for ResourcesVisitor<R> {
            type Value = Resources<R>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a sequence of extracted resources")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                Ok(Resources(R::deserialize(&mut seq)?))
            }
        }

        deserializer.deserialize_seq(ResourcesVisitor(PhantomData))
    }
}

// Unit types ---------------------------------------------------------------------------------------------------------

pub(crate) struct UnitSer<'a, T> {
    pub(crate) value: &'a T,
}

impl<'a, T> UnitSer<'a, T> {
    pub fn new(value: &'a T) -> Self {
        Self { value }
    }
}

impl<'a, T: Serialize> Serialize for UnitSer<'a, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if std::mem::size_of::<T>() == 0 {
            let seq = serializer.serialize_map(Some(0))?;
            seq.end()
        } else {
            self.value.serialize(serializer)
        }
    }
}

pub(crate) struct UnitDe<T> {
    pub(crate) value: T,
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for UnitDe<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if std::mem::size_of::<T>() == 0 {
            struct UnitDeVisitor<T>(PhantomData<T>);

            impl<'de, T: Deserialize<'de>> Visitor<'de> for UnitDeVisitor<T> {
                type Value = T;

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    formatter.write_str("an empty map")
                }

                fn visit_map<A>(self, _: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::MapAccess<'de>,
                {
                    // SAFETY: T is Unit value
                    #[allow(clippy::uninit_assumed_init)]
                    Ok(unsafe { std::mem::MaybeUninit::<T>::uninit().assume_init() })
                }
            }

            deserializer.deserialize_map(UnitDeVisitor(PhantomData))
        } else {
            T::deserialize(deserializer).map(|value| UnitDe { value })
        }
    }
}
