use std::collections::{HashMap, HashSet};

use bevy::{
    prelude::*,
    reflect::{
        GetTypeRegistration,
        TypeRegistration,
    },
};

/// The global registry of types that should be tracked by `bevy_save`.
#[derive(Resource, Default)]
pub struct SaveableRegistry {
    types: HashMap<String, TypeRegistration>,

    ignore_rollback: HashSet<String>,
}

impl SaveableRegistry {
    /// Register a type to be included in saves and rollback.
    pub fn register<T: GetTypeRegistration>(&mut self) {
        let reg = T::get_type_registration();

        self.types.insert(reg.type_name().to_owned(), reg);
    }

    /// Register a type to be excluded from rollback.
    /// 
    /// The type is still included in saves.
    pub fn ignore_rollback<T: GetTypeRegistration>(&mut self) {
        let reg = T::get_type_registration();

        self.ignore_rollback.insert(reg.type_name().to_owned());
    }

    /// Returns whether or not a type name is included in rollback.
    pub fn can_rollback(&self, name: &str) -> bool {
        !self.ignore_rollback.contains(name)
    }

    /// Returns an [`Iterator`] of registered saveable types.
    pub fn types(&self) -> impl Iterator<Item = &TypeRegistration> {
        self.types.values()
    }
}
