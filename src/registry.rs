use std::collections::HashMap;

use bevy::{
    prelude::*,
    reflect::{
        GetTypeRegistration,
        TypeRegistration,
    },
};

/// Contains the [`TypeRegistration`] and rollback eligibility for a saveable type.
pub struct SaveableRegistration {
    type_reg: TypeRegistration,
    rollback: bool,
}

impl SaveableRegistration {
    /// Returns a reference to the [`TypeRegistration`] of the saveable type.
    pub fn type_reg(&self) -> &TypeRegistration {
        &self.type_reg
    }

    /// Whether or not this saveable may participate in rollback.
    pub fn rollback(&self) -> bool {
        self.rollback
    }
}

/// The global registry of types that should be tracked by `bevy_save`.
/// Only types that are registered in here and [`AppTypeRegistry`] are included in save/load and rollback.
#[derive(Resource, Default)]
pub struct SaveableRegistry {
    types: HashMap<String, SaveableRegistration>,
}

impl SaveableRegistry {
    /// Register a type to be included in saves and rollback.
    pub fn register<T: GetTypeRegistration>(&mut self) {
        let type_reg = T::get_type_registration();

        self.types
            .insert(type_reg.type_name().into(), SaveableRegistration {
                type_reg,
                rollback: true,
            });
    }

    /// Exclude a type from rollback.
    ///
    /// The type is still included in saves.
    ///
    /// # Panics
    /// - If called on a type that has not been registered
    pub fn ignore_rollback<T: GetTypeRegistration>(&mut self) {
        let type_reg = T::get_type_registration();
        self.types.get_mut(type_reg.type_name()).unwrap().rollback = false;
    }

    /// Include a type in rollbacks.
    ///
    /// # Panics
    /// - If called on a type that has not been registered
    pub fn allow_rollback<T: GetTypeRegistration>(&mut self) {
        let type_reg = T::get_type_registration();
        self.types.get_mut(type_reg.type_name()).unwrap().rollback = true;
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
        self.types.get(type_name).unwrap().rollback()
    }

    /// Returns a reference to a [`SaveableRegistration`].
    pub fn get(&self, type_name: &str) -> Option<&SaveableRegistration> {
        self.types.get(type_name)
    }

    /// Returns an iterator of [`TypeRegistration`] references.
    pub fn types(&self) -> impl Iterator<Item = &TypeRegistration> {
        self.types.values().map(|reg| &reg.type_reg)
    }
}
