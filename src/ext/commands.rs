use bevy::prelude::*;

use crate::{
    commands::{
        CheckpointCommand,
        LoadCommand,
        RollbackCommand,
        SaveCommand,
        SpawnPrefabCommand,
    },
    prelude::*,
};

/// Extension trait that adds save-related methods to Bevy's [`Commands`].
pub trait CommandsSaveableExt {
    /// Save using the [`Pipeline`].
    fn save<P: Pipeline + Send + 'static>(&mut self, pipeline: P);

    /// Load using the [`Pipeline`].
    fn load<P: Pipeline + Send + 'static>(&mut self, pipeline: P);
}

impl CommandsSaveableExt for Commands<'_, '_> {
    fn save<P: Pipeline + Send + 'static>(&mut self, pipeline: P) {
        self.queue(SaveCommand(pipeline));
    }

    fn load<P: Pipeline + Send + 'static>(&mut self, pipeline: P) {
        self.queue(LoadCommand(pipeline));
    }
}

/// Extension trait that adds rollback checkpoint-related methods to Bevy's [`Commands`].
pub trait CommandsCheckpointExt {
    /// Create a checkpoint using the [`Pipeline`].
    fn checkpoint<P: Pipeline + Send + 'static>(&mut self, pipeline: P);

    /// Rollback the specified amount using the [`Pipeline`].
    fn rollback<P: Pipeline + Send + 'static>(&mut self, pipeline: P, checkpoints: isize);
}

impl CommandsCheckpointExt for Commands<'_, '_> {
    fn checkpoint<P: Pipeline + Send + 'static>(&mut self, pipeline: P) {
        self.queue(CheckpointCommand::new(pipeline));
    }

    fn rollback<P: Pipeline + Send + 'static>(&mut self, pipeline: P, checkpoints: isize) {
        self.queue(RollbackCommand::new(pipeline, checkpoints));
    }
}

/// Extension trait that adds prefab-related methods to Bevy's [`Commands`].
pub trait CommandsPrefabExt {
    /// Spawn a [`Prefab`] entity.
    fn spawn_prefab<P: Prefab + Send + 'static>(&mut self, prefab: P) -> EntityCommands;
}

impl CommandsPrefabExt for Commands<'_, '_> {
    fn spawn_prefab<P: Prefab + Send + 'static>(&mut self, prefab: P) -> EntityCommands {
        let target = self.spawn(P::Marker::default()).id();
        self.queue(SpawnPrefabCommand::new(target, prefab));
        self.entity(target)
    }
}
