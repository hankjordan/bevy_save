use bevy::prelude::*;

use crate::{
    Capture,
    CloneReflect,
    Error,
    Pipeline,
    Rollbacks,
    Snapshot, SnapshotSerializer, Backend, SnapshotDeserializer,
};

/// Extension trait that adds save-related methods to Bevy's [`World`].
pub trait WorldSaveableExt: Sized {
    /// Captures a [`Snapshot`] from the current [`World`] state.
    fn snapshot<C: Capture>(&self) -> Snapshot;

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
    fn snapshot<C: Capture>(&self) -> Snapshot {
        C::capture(self)
    }

    fn save<P: Pipeline>(&self, pipeline: P) -> Result<(), Error> {
        let registry = self.resource::<AppTypeRegistry>();
        let backend = self.resource::<P::Backend>();

        let snapshot = pipeline.capture(self);

        let ser = SnapshotSerializer::new(&snapshot, registry);

        backend.save::<P::Format, _>(pipeline.key(), ser)
    }

    fn load<P: Pipeline>(&mut self, pipeline: P) -> Result<(), Error> {
        let registry = self.resource::<AppTypeRegistry>().clone();
        let reg = registry.read();
        let backend = self.resource::<P::Backend>();

        let de = SnapshotDeserializer { registry: &reg };

        let snapshot = backend.load::<P::Format, _>(pipeline.key(), de)?;

        pipeline.apply(self, &snapshot)
    }
}

/// Extension trait that adds rollback-related methods to Bevy's [`World`].
pub trait WorldRollbackExt {
    /// Creates a checkpoint for rollback.
    fn checkpoint<C: Capture>(&mut self);

    /// Rolls back / forward the [`World`] state.
    ///
    /// # Errors
    /// - See [`Error`]
    fn rollback<C: Capture>(&mut self, checkpoints: isize) -> Result<(), Error>;
}

impl WorldRollbackExt for World {
    fn checkpoint<C: Capture>(&mut self) {
        let rollback = C::capture(self);
        self.resource_mut::<Rollbacks>().checkpoint(rollback);
    }

    fn rollback<C: Capture>(&mut self, checkpoints: isize) -> Result<(), Error> {
        if let Some(rollback) = self
            .resource_mut::<Rollbacks>()
            .rollback(checkpoints)
            .map(|r| r.clone_value())
        {
            C::apply(self, &rollback)
        } else {
            Ok(())
        }
    }
}
