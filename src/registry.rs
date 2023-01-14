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
    types: Vec<TypeRegistration>,
}

impl SaveableRegistry {
    /// Register a type to be included in saves and rollback.
    pub fn register<T: GetTypeRegistration>(&mut self) {
        self.types.push(T::get_type_registration());
    }

    /// Returns an Iterator of registered saveable types.
    pub fn types(&self) -> impl Iterator<Item = &TypeRegistration> {
        self.types.iter()
    }
}
