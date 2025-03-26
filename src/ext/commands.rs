use bevy::prelude::*;

use crate::{
    commands::{
        CheckpointCommand,
        LoadCommand,
        RollbackCommand,
        SaveCommand,
    },
    prelude::*,
};

/// Extension trait that adds save-related methods to Bevy's [`Commands`].
pub trait CommandsSaveableExt {
    /// Save using the [`Pipeline`].
    fn save<P: Pipeline>(&mut self, pipeline: P);

    /// Load using the [`Pipeline`].
    fn load<P: Pipeline>(&mut self, pipeline: P);
}

impl CommandsSaveableExt for Commands<'_, '_> {
    fn save<P: Pipeline>(&mut self, pipeline: P) {
        self.queue(SaveCommand(pipeline));
    }

    fn load<P: Pipeline>(&mut self, pipeline: P) {
        self.queue(LoadCommand(pipeline));
    }
}

/// Extension trait that adds rollback checkpoint-related methods to Bevy's [`Commands`].
pub trait CommandsCheckpointExt {
    /// Create a checkpoint using the [`Pipeline`].
    fn checkpoint<P: Pipeline>(&mut self);

    /// Rollback the specified amount using the [`Pipeline`].
    fn rollback<P: Pipeline>(&mut self, checkpoints: isize);
}

impl CommandsCheckpointExt for Commands<'_, '_> {
    fn checkpoint<P: Pipeline>(&mut self) {
        self.queue(CheckpointCommand::<P>::new());
    }

    fn rollback<P: Pipeline>(&mut self, checkpoints: isize) {
        self.queue(RollbackCommand::<P>::new(checkpoints));
    }
}
