use std::collections::HashMap;

use bevy::{
    prelude::*,
    reflect::GetTypeRegistration,
};

/// The global registry of types that should be tracked by `bevy_save`.
/// 
/// Only types that are registered in here and [`AppTypeRegistry`] are included in save/load and rollback.
#[derive(Resource, Default)]
pub struct SaveableRegistry {
    types: HashMap<String, bool>,
}

impl SaveableRegistry {
    /// Register a type to be included in saves and rollback.
    pub fn register<T: GetTypeRegistration>(&mut self) {
        let type_reg = T::get_type_registration();
        self.types.insert(type_reg.type_name().into(), true);
    }

    /// Exclude a type from rollback.
    ///
    /// The type is still included in saves.
    ///
    /// # Panics
    /// - If called on a type that has not been registered
    pub fn ignore_rollback<T: GetTypeRegistration>(&mut self) {
        let type_reg = T::get_type_registration();
        *self.types.get_mut(type_reg.type_name()).unwrap() = false;
    }

    /// Include a type in rollbacks.
    ///
    /// # Panics
    /// - If called on a type that has not been registered
    pub fn allow_rollback<T: GetTypeRegistration>(&mut self) {
        let type_reg = T::get_type_registration();
        *self.types.get_mut(type_reg.type_name()).unwrap() = true;
    }

    /// Returns whether or not a type name is registered in the [`SaveableRegistry`].
    pub fn contains(&self, type_name: &str) -> bool {
        self.types.contains_key(type_name)
    }

    /// Returns whether or not a type name is included in rollback.
    ///
    /// # Panics
    /// - If called on a type that has not been registered
    pub fn can_rollback(&self, type_name: &str) -> bool {
        *self.types.get(type_name).unwrap()
    }

    /// Returns an iterator over registered type names.
    pub fn types(&self) -> impl Iterator<Item = &String> {
        self.types.keys()
    }
}
