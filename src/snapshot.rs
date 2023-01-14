use std::{
    borrow::Cow,
    collections::HashSet,
};

use bevy::{
    ecs::entity::EntityMap,
    prelude::*,
    reflect::{
        serde::{
            TypedReflectDeserializer,
            TypedReflectSerializer,
            UntypedReflectDeserializer,
        },
        TypeRegistryArc,
        TypeRegistryInternal,
    },
    scene::serde::SceneSerializer,
};
use serde::{
    de::{
        self,
        DeserializeSeed,
        MapAccess,
        SeqAccess,
        Visitor,
    },
    ser::{
        SerializeMap,
        SerializeStruct,
    },
    Deserialize,
    Serialize,
};

use crate::{
    reflect::CloneReflect,
    scene::SceneDeserializer,
};

/// A complete snapshot of the game state.
/// 
/// Can be serialized via [`SnapshotSerializer`] and deserialized via [`SnapshotDeserializer`].
pub struct Snapshot {
    pub(crate) resources: Vec<Box<dyn Reflect>>,
    pub(crate) scene: DynamicScene,
}

impl Clone for Snapshot {
    fn clone(&self) -> Self {
        Self {
            resources: self.resources.clone_value(),
            scene: self.scene.clone_value(),
        }
    }
}

impl Snapshot {
    /// Apply the `Snapshot` to the `World`, restoring it to the saved state.
    pub fn apply(&self, world: &mut World) {
        world.clear_entities();
        world.clear_trackers();

        let _s = self.scene.write_to_world(world, &mut EntityMap::default());

        let registry_arc = world.resource::<AppTypeRegistry>().clone();
        let registry = registry_arc.read();

        for reflect in &self.resources {
            if let Some(reg) = registry.get_with_name(reflect.type_name()) {
                if let Some(res) = reg.data::<ReflectResource>() {
                    res.apply(world, reflect.as_reflect());
                }
            }
        }
    }
}

struct ResourcesSerializer<'a> {
    pub resources: &'a [Box<dyn Reflect>],
    pub registry: &'a TypeRegistryArc,
}

impl<'a> ResourcesSerializer<'a> {
    pub fn new(resources: &'a [Box<dyn Reflect>], registry: &'a TypeRegistryArc) -> Self {
        Self {
            resources,
            registry,
        }
    }
}

impl<'a> Serialize for ResourcesSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_map(Some(self.resources.len()))?;

        for resource in self.resources {
            state.serialize_entry(
                resource.type_name(),
                &TypedReflectSerializer::new(&**resource, &self.registry.read()),
            )?;
        }

        state.end()
    }
}

struct ResourcesDeserializer<'a> {
    pub registry: &'a TypeRegistryInternal,
}

impl<'a> ResourcesDeserializer<'a> {
    pub fn new(registry: &'a TypeRegistryInternal) -> Self {
        Self { registry }
    }
}

impl<'a, 'de> DeserializeSeed<'de> for ResourcesDeserializer<'a> {
    type Value = Vec<Box<dyn Reflect>>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(ResourcesVisitor {
            registry: self.registry,
        })
    }
}

#[derive(Deserialize)]
#[serde(transparent)]
struct BorrowableCowStr<'a>(#[serde(borrow)] Cow<'a, str>);

struct ResourcesVisitor<'a> {
    registry: &'a TypeRegistryInternal,
}

impl<'a, 'de> Visitor<'de> for ResourcesVisitor<'a> {
    type Value = Vec<Box<dyn Reflect>>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("map of resources")
    }

    fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut added = HashSet::new();
        let mut resources = Vec::new();

        while let Some(BorrowableCowStr(key)) = map.next_key()? {
            if !added.insert(key.clone()) {
                return Err(de::Error::custom(format!("duplicate resource: `{key}`")));
            }

            let registration = self
                .registry
                .get_with_name(&key)
                .ok_or_else(|| de::Error::custom(format!("no registration found for `{key}`")))?;

            resources.push(
                map.next_value_seed(TypedReflectDeserializer::new(registration, self.registry))?,
            );
        }

        Ok(resources)
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

const SNAPSHOT_STRUCT: &str = "Snapshot";
const SNAPSHOT_FIELDS: &[&str] = &["resources", "scene"];

/// A serializer for Snapshot that uses reflection.
pub struct SnapshotSerializer<'a> {
    /// The Snapshot to be serialized.
    pub snapshot: &'a Snapshot,

    /// The TypeRegistry to use for reflection.
    pub registry: &'a TypeRegistryArc,
}

impl<'a> SnapshotSerializer<'a> {
    /// Returns a new instance of `SnapshotSerializer`.
    pub fn new(snapshot: &'a Snapshot, registry: &'a TypeRegistryArc) -> Self {
        Self { snapshot, registry }
    }
}

impl<'a> Serialize for SnapshotSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let resources = ResourcesSerializer::new(&self.snapshot.resources, self.registry);
        let scene = SceneSerializer::new(&self.snapshot.scene, self.registry);

        let mut state = serializer.serialize_struct(SNAPSHOT_STRUCT, SNAPSHOT_FIELDS.len())?;

        state.serialize_field(SNAPSHOT_FIELDS[0], &resources)?;
        state.serialize_field(SNAPSHOT_FIELDS[1], &scene)?;

        state.end()
    }
}

/// A deserializer for Snapshot that uses reflection.
pub struct SnapshotDeserializer<'a> {
    /// The TypeRegistry to use for reflection.
    pub registry: &'a TypeRegistryInternal,
}

impl<'a> SnapshotDeserializer<'a> {
    /// Returns a new instance of `SnapshotDeserializer`.
    pub fn new(registry: &'a TypeRegistryInternal) -> Self {
        Self { registry }
    }
}

impl<'a, 'de> DeserializeSeed<'de> for SnapshotDeserializer<'a> {
    type Value = Snapshot;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_struct(SNAPSHOT_STRUCT, SNAPSHOT_FIELDS, SnapshotVisitor {
            registry: self.registry,
        })
    }
}

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "lowercase")]
enum SnapshotFields {
    Resources,
    Scene,
}

struct SnapshotVisitor<'a> {
    registry: &'a TypeRegistryInternal,
}

impl<'a, 'de> Visitor<'de> for SnapshotVisitor<'a> {
    type Value = Snapshot;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("struct Snapshot")
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let resources = seq
            .next_element_seed(ResourcesDeserializer::new(self.registry))?
            .ok_or_else(|| de::Error::missing_field(SNAPSHOT_FIELDS[0]))?;

        let scene = seq
            .next_element_seed(SceneDeserializer {
                type_registry: self.registry,
            })?
            .ok_or_else(|| de::Error::missing_field(SNAPSHOT_FIELDS[1]))?;

        Ok(Self::Value { resources, scene })
    }

    fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
    where
        V: MapAccess<'de>,
    {
        let mut resources = None;
        let mut scene = None;

        while let Some(key) = map.next_key()? {
            match key {
                SnapshotFields::Resources => {
                    if resources.is_some() {
                        return Err(de::Error::duplicate_field(SNAPSHOT_FIELDS[0]));
                    }
                    resources =
                        Some(map.next_value_seed(ResourcesDeserializer::new(self.registry))?);
                }

                SnapshotFields::Scene => {
                    if scene.is_some() {
                        return Err(de::Error::duplicate_field(SNAPSHOT_FIELDS[1]));
                    }

                    scene = Some(map.next_value_seed(SceneDeserializer {
                        type_registry: self.registry,
                    })?);
                }
            }
        }

        let resources = resources.ok_or_else(|| de::Error::missing_field(SNAPSHOT_FIELDS[0]))?;
        let scene = scene.ok_or_else(|| de::Error::missing_field(SNAPSHOT_FIELDS[1]))?;

        Ok(Self::Value { resources, scene })
    }
}
