use bevy::prelude::*;

use crate::{
    Backend,
    CloneReflect,
    Error,
    Pipeline,
    Rollbacks,
    Snapshot,
    SnapshotBuilder,
    SnapshotDeserializer,
    SnapshotSerializer,
};

/// Extension trait that adds save-related methods to Bevy's [`World`].
pub trait WorldSaveableExt: Sized {
    /// Captures a [`Snapshot`] from the current [`World`] state.
    fn snapshot<P: Pipeline>(&self) -> Snapshot;

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
    fn snapshot<P: Pipeline>(&self) -> Snapshot {
        P::capture(Snapshot::builder(self))
    }

    fn save<P: Pipeline>(&self, pipeline: P) -> Result<(), Error> {
        let registry = self.resource::<AppTypeRegistry>();
        let backend = self.resource::<P::Backend>();

        let snapshot = pipeline.capture_seed(Snapshot::builder(self));

        let ser = SnapshotSerializer::new(&snapshot, registry);

        backend.save::<P::Format, _>(pipeline.key(), &ser)
    }

    fn load<P: Pipeline>(&mut self, pipeline: P) -> Result<(), Error> {
        let registry = self.resource::<AppTypeRegistry>().clone();
        let reg = registry.read();
        let backend = self.resource::<P::Backend>();

        let de = SnapshotDeserializer { registry: &reg };

        let snapshot = backend.load::<P::Format, _, _>(pipeline.key(), de)?;

        pipeline.apply_seed(self, &snapshot)
    }
}

/// Extension trait that adds rollback-related methods to Bevy's [`World`].
pub trait WorldRollbackExt {
    /// Creates a checkpoint for rollback.
    fn checkpoint<P: Pipeline>(&mut self);

    /// Rolls back / forward the [`World`] state.
    ///
    /// # Errors
    /// - See [`Error`]
    fn rollback<P: Pipeline>(&mut self, checkpoints: isize) -> Result<(), Error>;
}

impl WorldRollbackExt for World {
    fn checkpoint<P: Pipeline>(&mut self) {
        let rollback = P::capture(SnapshotBuilder::rollback(self));
        self.resource_mut::<Rollbacks>().checkpoint(rollback);
    }

    fn rollback<P: Pipeline>(&mut self, checkpoints: isize) -> Result<(), Error> {
        if let Some(rollback) = self
            .resource_mut::<Rollbacks>()
            .rollback(checkpoints)
            .map(|r| r.clone_value())
        {
            P::apply(self, &rollback)
        } else {
            Ok(())
        }
    }
}
