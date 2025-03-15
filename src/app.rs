use std::any::Any;

use bevy::prelude::*;

use crate::prelude::*;

/// Extension trait that adds save-related methods to Bevy's [`App`].
pub trait AppSaveableExt {
    /// Initialize a [`Pipeline`], allowing it to be used with [`WorldSaveableExt`] methods.
    fn init_pipeline<P: Pipeline>(&mut self) -> &mut Self;

    /// Set a type to allow rollback - it will be included in rollback and affected by save/load.
    fn allow_rollback<T: Any>(&mut self) -> &mut Self;

    /// Set a type to ignore rollback - it will be included in save/load but it won't change during rollback.
    fn deny_rollback<T: Any>(&mut self) -> &mut Self;
}

impl AppSaveableExt for App {
    fn init_pipeline<P: Pipeline>(&mut self) -> &mut Self {
        P::build(self);
        self
    }

    fn allow_rollback<T: Any>(&mut self) -> &mut Self {
        let mut registry = self.world_mut().resource_mut::<RollbackRegistry>();
        registry.allow::<T>();
        self
    }

    fn deny_rollback<T: Any>(&mut self) -> &mut Self {
        let mut registry = self.world_mut().resource_mut::<RollbackRegistry>();
        registry.deny::<T>();
        self
    }
}
