use std::{
    any::{
        Any,
        TypeId,
    },
    marker::PhantomData,
};

use bevy::{
    ecs::entity::MapEntities,
    prelude::*,
};
use serde::Deserialize;

use crate::{
    extract::{
        Extract,
        ExtractComponent,
        ExtractDeserialize,
        ExtractMap,
        ExtractResource,
    },
    Snapshot,
    SnapshotBuilder,
};

fn take<T, F>(mut_ref: &mut T, closure: F)
where
    F: FnOnce(T) -> T,
{
    use std::ptr;

    unsafe {
        let old_t = ptr::read(mut_ref);
        ptr::write(mut_ref, closure(old_t));
    }
}

/// The global registry of types that should be included in [`Rollbacks`](crate::Rollbacks).
///
/// Only types that are registered in here and [`AppTypeRegistry`] are included in rollbacks.
#[derive(Resource, Default)]
pub struct RollbackRegistry {
    types: SceneFilter,
}

impl RollbackRegistry {
    /// Allow all types to roll back.
    pub fn allow_all(&mut self) {
        self.types = SceneFilter::allow_all();
    }

    /// Deny all types from rolling back.
    pub fn deny_all(&mut self) {
        self.types = SceneFilter::deny_all();
    }

    /// Include a type in rollbacks.
    pub fn allow<T: Any>(&mut self) {
        take(&mut self.types, |types| types.allow::<T>());
    }

    /// Exclude a type from rollback.
    ///
    /// The type is still included in normal snapshots.
    pub fn deny<T: Any>(&mut self) {
        take(&mut self.types, |types| types.deny::<T>());
    }

    /// Check if a type is allowed to roll back.
    pub fn is_allowed<T: Any>(&self) -> bool {
        self.types.is_allowed::<T>()
    }

    /// Check if a type is allowed to roll back by id.
    pub fn is_allowed_by_id(&self, type_id: TypeId) -> bool {
        self.types.is_allowed_by_id(type_id)
    }

    /// Check if a type is denied from rolling back.
    pub fn is_denied<T: Any>(&self) -> bool {
        self.types.is_denied::<T>()
    }

    /// Check if a type is denied from rolling back by id.
    pub fn is_denied_by_id(&self, type_id: TypeId) -> bool {
        self.types.is_denied_by_id(type_id)
    }
}

// --------------------------------------------------------------------------------------------------------------------

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
impl<C, R> SaveRegistry<C, R>
where
    C: ExtractComponent + ExtractDeserialize,
    R: ExtractResource + ExtractDeserialize,
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

#[allow(clippy::unused_self)]
impl<C, R> SaveRegistry<C, R>
where
    C: ExtractComponent,
    R: ExtractResource,
{
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
