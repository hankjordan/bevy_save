use bevy::{
    ecs::entity::EntityMap,
    prelude::*,
    reflect::{
        serde::TypedReflectSerializer,
        TypeRegistryArc,
    },
    scene::serde::SceneSerializer,
};
use serde::{
    ser::{
        SerializeMap,
        SerializeStruct,
    },
    Serialize,
};

use crate::reflect::CloneReflect;

/// A complete snapshot of the game state.
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

#[allow(missing_docs)]
pub struct ResourcesSerializer<'a> {
    pub resources: &'a [Box<dyn Reflect>],
    pub registry: &'a TypeRegistryArc,
}

#[allow(missing_docs)]
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

#[allow(missing_docs)]
pub struct SnapshotSerializer<'a> {
    pub snapshot: &'a Snapshot,
    pub registry: &'a TypeRegistryArc,
}

#[allow(missing_docs)]
impl<'a> SnapshotSerializer<'a> {
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

        let mut state = serializer.serialize_struct("Snapshot", 2)?;

        state.serialize_field("resources", &resources)?;
        state.serialize_field("scene", &scene)?;

        state.end()
    }
}
