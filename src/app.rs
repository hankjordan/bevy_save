use bevy::{
    prelude::*,
    reflect::GetTypeRegistration,
};

use crate::prelude::*;

/// Extension trait that adds save-related methods to Bevy's [`App`].
pub trait AppSaveableExt {
    /// Initialize a [`Pipeline`], allowing it to be used with [`WorldSaveableExt`] methods.
    fn init_pipeline<P: Pipeline>(&mut self) -> &mut Self;

    /// Register a type as saveable - it will be included in rollback and affected by save/load.
    fn register_saveable<T: GetTypeRegistration>(&mut self) -> &mut Self;

    /// Set a type to ignore rollback - it will be included in save/load but it won't change during rollback.
    fn ignore_rollback<T: GetTypeRegistration>(&mut self) -> &mut Self;

    /// Set a type to allow rollback - it will be included in rollback and affected by save/load.
    fn allow_rollback<T: GetTypeRegistration>(&mut self) -> &mut Self;
}

impl AppSaveableExt for App {
    fn init_pipeline<P: Pipeline>(&mut self) -> &mut Self {
        P::build(self);
        self
    }

    fn register_saveable<T: GetTypeRegistration>(&mut self) -> &mut Self {
        self ////
            .init_resource::<SaveableRegistry>()
            .init_resource::<Rollbacks>()
            .register_type::<T>();

        let mut registry = self.world.resource_mut::<SaveableRegistry>();

        registry.register::<T>();

        self
    }

    fn ignore_rollback<T: GetTypeRegistration>(&mut self) -> &mut Self {
        let mut registry = self.world.resource_mut::<SaveableRegistry>();

        registry.ignore_rollback::<T>();

        self
    }

    fn allow_rollback<T: GetTypeRegistration>(&mut self) -> &mut Self {
        let mut registry = self.world.resource_mut::<SaveableRegistry>();

        registry.allow_rollback::<T>();

        self
    }
}
