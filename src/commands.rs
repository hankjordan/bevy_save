//! Bevy commands for deferring mutation.

use std::marker::PhantomData;

use bevy::prelude::*;

use crate::prelude::*;

/// Save using the [`Pipeline`].
pub struct SaveCommand<P>(pub P);

impl<P: Pipeline> Command for SaveCommand<P> {
    fn apply(self, world: &mut World) {
        if let Err(e) = world.save(self.0) {
            warn!("Failed to save world: {:?}", e);
        }
    }
}

/// Load using the [`Pipeline`].
pub struct LoadCommand<P>(pub P);

impl<P: Pipeline> Command for LoadCommand<P> {
    fn apply(self, world: &mut World) {
        if let Err(e) = world.load(self.0) {
            warn!("Failed to load world: {:?}", e);
        }
    }
}

/// Create a checkpoint using the [`Pipeline`].
pub struct CheckpointCommand<P> {
    _marker: PhantomData<P>,
}

impl<P> Default for CheckpointCommand<P> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<P> CheckpointCommand<P> {
    /// Create a [`CheckpointCommand`] from the [`Pipeline`].
    pub fn new() -> Self {
        Self::default()
    }
}

impl<P: Pipeline> Command for CheckpointCommand<P> {
    fn apply(self, world: &mut World) {
        world.checkpoint::<P>();
    }
}

/// Rollback the specified amount using the [`Pipeline`].
pub struct RollbackCommand<P> {
    checkpoints: isize,
    _marker: PhantomData<P>,
}

impl<P> RollbackCommand<P> {
    /// Create a [`RollbackCommand`] from the [`Pipeline`] and checkpoint count.
    pub fn new(checkpoints: isize) -> Self {
        Self {
            checkpoints,
            _marker: PhantomData,
        }
    }
}

impl<P: Pipeline> Command for RollbackCommand<P> {
    fn apply(self, world: &mut World) {
        if let Err(e) = world.rollback::<P>(self.checkpoints) {
            warn!("Failed to rollback world: {:?}", e);
        }
    }
}
