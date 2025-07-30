use std::any::{
    Any,
    TypeId,
};

use bevy::prelude::*;

fn take<T, F>(mut_ref: &mut T, closure: F)
where
    F: FnOnce(T) -> T,
{
    use std::ptr;

    // SAFETY: We have an exclusive reference to the value
    unsafe {
        let old_t = ptr::read(mut_ref);
        ptr::write(mut_ref, closure(old_t));
    }
}

/// The registry of types that should be included in [`Checkpoints`](crate::reflect::checkpoint::Checkpoints).
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

    /// Include a type in checkpoints.
    pub fn allow<T: Any>(&mut self) {
        take(&mut self.types, |types| types.allow::<T>());
    }

    /// Exclude a type from checkpoints.
    ///
    /// The type is still included in normal snapshots.
    pub fn deny<T: Any>(&mut self) {
        take(&mut self.types, |types| types.deny::<T>());
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
