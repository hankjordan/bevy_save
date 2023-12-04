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
