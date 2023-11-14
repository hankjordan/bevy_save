use bevy::prelude::*;

use crate::{
    Applier,
    CloneReflect,
    Error,
    Pipeline,
    Rollback,
    Rollbacks,
    Snapshot,
};

/// Extension trait that adds save-related methods to Bevy's [`World`].
pub trait WorldSaveableExt: Sized {
    /// Returns a [`Snapshot`] of the current [`World`] state.
    fn snapshot(&self) -> Snapshot;

    /// Creates a checkpoint for rollback.
    fn checkpoint(&mut self);

    /// Rolls back / forward the [`World`] state.
    ///
    /// # Errors
    /// - See [`SaveableError`]
    fn rollback(&mut self, checkpoints: isize) -> Result<(), Error>;

    /// Rolls back / forward the [`World`] state.
    ///
    /// The applier allows you to customize how the [`Rollback`] will be applied to the [`World`].
    ///
    /// # Errors
    /// - See [`SaveableError`]
    fn rollback_applier(&mut self, checkpoints: isize) -> Option<Applier<Rollback>>;

    /// Saves the game state with the given [`Pipeline`].
    ///
    /// # Errors
    /// - See [`Error`]
    fn save<P: Pipeline>(&self, pipeline: P) -> Result<(), Error>;

    /// Loads the game state with the given [`Pipeline`].
    ///
    /// # Errors
    /// - See [`Error`]
    fn load<P: Pipeline>(&mut self, pipeline: P) -> Result<(), Error>;
}

impl WorldSaveableExt for World {
    fn snapshot(&self) -> Snapshot {
        Snapshot::from_world(self)
    }

    fn checkpoint(&mut self) {
        let rollback = Rollback::from_world(self);
        let mut state = self.resource_mut::<Rollbacks>();
        state.checkpoint(rollback);
    }

    fn rollback(&mut self, checkpoints: isize) -> Result<(), Error> {
        self.rollback_applier(checkpoints)
            .map_or(Ok(()), |a| a.apply())
    }

    fn rollback_applier(&mut self, checkpoints: isize) -> Option<Applier<Rollback>> {
        let mut state = self.resource_mut::<Rollbacks>();

        state
            .rollback(checkpoints)
            .map(|r| r.clone_value())
            .map(|snap| snap.into_applier(self))
    }

    fn save<P: Pipeline>(&self, pipeline: P) -> Result<(), Error> {
        pipeline.save(self)
    }

    fn load<P: Pipeline>(&mut self, pipeline: P) -> Result<(), Error> {
        pipeline.load(self)
    }
}
