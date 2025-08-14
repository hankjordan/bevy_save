use std::any::{
    Any,
    TypeId,
};

use bevy::prelude::*;

/// The registry of types that should be included in
/// [`Checkpoints`](crate::reflect::checkpoint::Checkpoints).
///
/// Only types that are registered in here and [`AppTypeRegistry`] are included in checkpoints.
#[derive(Resource, Default)]
pub struct CheckpointRegistry {
    types: SceneFilter,
}

impl CheckpointRegistry {
    /// Allow all types to be included in checkpoints.
    pub fn allow_all(&mut self) {
        self.types = SceneFilter::allow_all();
    }

    /// Deny all types from being included in checkpoints.
    pub fn deny_all(&mut self) {
        self.types = SceneFilter::deny_all();
    }

    /// Include a type `T` in checkpoints.
    pub fn allow<T: Any>(&mut self) {
        self.types = std::mem::take(&mut self.types).allow::<T>();
    }

    /// Include a type in checkpoints.
    pub fn allow_id(&mut self, type_id: TypeId) {
        self.types = std::mem::take(&mut self.types).allow_by_id(type_id);
    }

    /// Exclude a type `T` from checkpoints.
    ///
    /// The type is still included in normal snapshots.
    pub fn deny<T: Any>(&mut self) {
        self.types = std::mem::take(&mut self.types).deny::<T>();
    }

    /// Exclude a type from checkpoints.
    ///
    /// The type is still included in normal snapshots.
    pub fn deny_id(&mut self, type_id: TypeId) {
        self.types = std::mem::take(&mut self.types).deny_by_id(type_id);
    }

    /// Check if a type is allowed to be included in checkpoints.
    pub fn is_allowed<T: Any>(&self) -> bool {
        self.types.is_allowed::<T>()
    }

    /// Check if a type is allowed to be included in checkpoints by id.
    pub fn is_allowed_by_id(&self, type_id: TypeId) -> bool {
        self.types.is_allowed_by_id(type_id)
    }

    /// Check if a type is denied from being included in checkpoints.
    pub fn is_denied<T: Any>(&self) -> bool {
        self.types.is_denied::<T>()
    }

    /// Check if a type is denied from being included in checkpoints by id.
    pub fn is_denied_by_id(&self, type_id: TypeId) -> bool {
        self.types.is_denied_by_id(type_id)
    }
}
