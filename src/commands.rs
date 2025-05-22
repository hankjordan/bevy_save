//! Bevy commands for deferring mutation.

use bevy::{log::prelude::*, prelude::*};

use crate::prelude::*;

/// Save using the [`Pipeline`].
pub struct SaveCommand<P>(pub P);

impl<P: Pipeline + Send + 'static> Command for SaveCommand<P> {
    fn apply(self, world: &mut World) {
        if let Err(e) = world.save(self.0) {
            warn!("Failed to save world: {:?}", e);
        }
    }
}

/// Load using the [`Pipeline`].
pub struct LoadCommand<P>(pub P);

impl<P: Pipeline + Send + 'static> Command for LoadCommand<P> {
    fn apply(self, world: &mut World) {
        if let Err(e) = world.load(self.0) {
            warn!("Failed to load world: {:?}", e);
        }
    }
}

/// Create a checkpoint using the [`Pipeline`].
pub struct CheckpointCommand<P> {
    pipeline: P,
}

impl<P> CheckpointCommand<P> {
    /// Create a [`CheckpointCommand`] from the [`Pipeline`].
    pub fn new(pipeline: P) -> Self {
        Self { pipeline }
    }
}

impl<P: Pipeline + Send + 'static> Command for CheckpointCommand<P> {
    fn apply(self, world: &mut World) {
        world.checkpoint(self.pipeline);
    }
}

/// Rollback the specified amount using the [`Pipeline`].
pub struct RollbackCommand<P> {
    pipeline: P,
    checkpoints: isize,
}

impl<P> RollbackCommand<P> {
    /// Create a [`RollbackCommand`] from the [`Pipeline`] and checkpoint count.
    pub fn new(pipeline: P, checkpoints: isize) -> Self {
        Self {
            pipeline,
            checkpoints,
        }
    }
}

impl<P: Pipeline + Send + 'static> Command for RollbackCommand<P> {
    fn apply(self, world: &mut World) {
        if let Err(e) = world.rollback(self.pipeline, self.checkpoints) {
            warn!("Failed to rollback world: {:?}", e);
        }
    }
}

/// Spawn an instance of the [`Prefab`].
pub struct SpawnPrefabCommand<P> {
    target: Entity,
    prefab: P,
    target_original: Option<Entity>,
}

impl<P> SpawnPrefabCommand<P> {
    /// Create a [`SpawnPrefabCommand`] from the target entity and [`Prefab`].
    pub fn new(target: Entity, prefab: P, target_original: Option<Entity>) -> Self {
        Self {
            target,
            prefab,
            target_original,
        }
    }
}

impl<P: Prefab + Send + 'static> Command for SpawnPrefabCommand<P> {
    fn apply(self, world: &mut World) {
        self.prefab.spawn(self.target, world, self.target_original);
    }
}
