use std::marker::PhantomData;

use bevy::{
    ecs::entity::MapEntities,
    prelude::*,
};
use serde::Deserialize;

use crate::{
    prelude::*,
    typed::extract::{
        Extract,
        ExtractDeserialize,
        ExtractMap,
        Extractable,
    },
};

/// Registry for types to extracted when creating a [`Snapshot`] or deserializing from a save
pub struct SaveRegistry<C, R> {
    _marker: PhantomData<(C, R)>,
}

impl Default for SaveRegistry<(), ()> {
    fn default() -> Self {
        Self::new()
    }
}

impl SaveRegistry<(), ()> {
    /// Creates a new, empty save registry.
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<C, R> SaveRegistry<C, R> {
    /// Register a component to be extracted when building or included when deserializing snapshots.
    ///
    /// You must extract entities from the world in order to actually get any output.
    pub fn register_component<T: Component + Clone>(self) -> SaveRegistry<(C, Extract<T>), R> {
        SaveRegistry {
            _marker: PhantomData,
        }
    }

    /// Register a component to be extracted when building or included when deserializing snapshots.
    ///
    /// You must extract entities from the world in order to actually get any output.
    ///
    /// Runs the [`MapEntities`] implementation when applied.
    pub fn register_component_map<T: Component + Clone + MapEntities>(
        self,
    ) -> SaveRegistry<(C, ExtractMap<T>), R> {
        SaveRegistry {
            _marker: PhantomData,
        }
    }

    /// Register a resource to be extracted when building or included when deserializing snapshots.
    ///
    /// The resource is automatically extracted when the builder is built into a [`Snapshot`].
    pub fn register_resource<T: Resource + Clone>(self) -> SaveRegistry<C, (R, Extract<T>)> {
        SaveRegistry {
            _marker: PhantomData,
        }
    }

    /// Register a resource to be extracted when building or included when deserializing snapshots.
    ///
    /// The resource is automatically extracted when the builder is built into a [`Snapshot`].
    ///
    /// Runs the [`MapEntities`] implementation when applied.
    pub fn register_resource_map<T: Resource + Clone + MapEntities>(
        self,
    ) -> SaveRegistry<C, (R, ExtractMap<T>)> {
        SaveRegistry {
            _marker: PhantomData,
        }
    }
}

#[allow(clippy::unused_self)]
impl<C, R> SaveRegistry<C, R> {
    /// Creates a [`SnapshotBuilder`] from the registry and the given [`World`].
    pub fn builder<'w>(
        &self,
        world: &'w World,
    ) -> SnapshotBuilder<'w, impl Iterator<Item = Entity> + 'w, C, R> {
        SnapshotBuilder {
            world,
            entities: [].into_iter(),
            _marker: PhantomData,
        }
    }
}

#[allow(clippy::unused_self)]
impl<C, R> SaveRegistry<C, R>
where
    C: Extractable + ExtractDeserialize,
    R: Extractable + ExtractDeserialize,
{
    /// Deserializes the according [`Snapshot`] from the given [`Deserializer`](serde::Deserializer).
    ///
    /// # Errors
    /// If deserialization fails, due to mis-matching registered types or other issue.
    pub fn deserialize<'de, D: serde::de::Deserializer<'de>>(
        &self,
        de: D,
    ) -> Result<Snapshot<C, R>, D::Error> {
        Snapshot::<C, R>::deserialize(de)
    }
}
