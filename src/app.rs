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
    /// Register a type as saveable - it will be included in World snapshots and affected by save/load.
    fn register_saveable<T: 'static + GetTypeRegistration>(&mut self) -> &mut Self;
}

impl AppSaveableExt for App {
    fn register_saveable<T: 'static + GetTypeRegistration>(&mut self) -> &mut Self {
        self ////
            .init_resource::<Rollbacks>()
            .init_resource::<SaveableRegistry>()
            .register_type::<T>()
            .add_startup_system(register::<T>)
    }
}

fn register<T: 'static + GetTypeRegistration>(mut registry: ResMut<SaveableRegistry>) {
    registry.register::<T>();
}
