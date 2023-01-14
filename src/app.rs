use bevy::{
    prelude::*,
    reflect::GetTypeRegistration,
};

use crate::{
    Rollbacks,
    SaveableRegistry,
};

/// Extension trait that adds save-related methods to Bevy's `App`.
pub trait AppSaveableExt {
    /// Register a type as saveable - it will be included in rollback and affected by save/load.
    fn register_saveable<T: 'static + GetTypeRegistration>(&mut self) -> &mut Self;

    /// Set a type to ignore rollback - if registered it will be included in save/load but it won't change during rollback.
    fn ignore_rollback<T: 'static + GetTypeRegistration>(&mut self) -> &mut Self;
}

impl AppSaveableExt for App {
    fn register_saveable<T: 'static + GetTypeRegistration>(&mut self) -> &mut Self {
        self ////
            .init_resource::<Rollbacks>()
            .init_resource::<SaveableRegistry>()
            ////
            .register_type::<T>()
            ////
            .add_startup_system(register_saveable::<T>)
    }

    fn ignore_rollback<T: 'static + GetTypeRegistration>(&mut self) -> &mut Self {
        self.add_startup_system(ignore_rollback::<T>)
    }
}

fn register_saveable<T: 'static + GetTypeRegistration>(mut registry: ResMut<SaveableRegistry>) {
    registry.register::<T>();
}

fn ignore_rollback<T: 'static + GetTypeRegistration>(mut registry: ResMut<SaveableRegistry>) {
    registry.ignore_rollback::<T>();
}
