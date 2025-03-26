use std::any::Any;

use bevy::prelude::*;

use crate::{
    checkpoint::CheckpointRegistry,
    prelude::*,
};

/// Extension trait that adds save-related methods to Bevy's [`App`].
pub trait AppSaveableExt {
    /// Initialize a [`Pipeline`], allowing it to be used with [`WorldSaveableExt`] methods.
    fn init_pipeline<P: Pipeline>(&mut self) -> &mut Self;
}

impl AppSaveableExt for App {
    fn init_pipeline<P: Pipeline>(&mut self) -> &mut Self {
        P::build(self);
        self
    }
}

/// Extension trait that adds rollback checkpoint-related methods to Bevy's [`App`].
pub trait AppCheckpointExt {
    /// Set a type to allow rollback - it will be included in rollback checkpoints and affected by save/load.
    fn allow_checkpoint<T: Any>(&mut self) -> &mut Self;

    /// Set a type to ignore rollback - it will be included in save/load but it won't change during rollback.
    fn deny_checkpoint<T: Any>(&mut self) -> &mut Self;
}

impl AppCheckpointExt for App {
    fn allow_checkpoint<T: Any>(&mut self) -> &mut Self {
        self.world_mut()
            .resource_mut::<CheckpointRegistry>()
            .allow::<T>();
        self
    }

    fn deny_checkpoint<T: Any>(&mut self) -> &mut Self {
        self.world_mut()
            .resource_mut::<CheckpointRegistry>()
            .deny::<T>();
        self
    }
}
