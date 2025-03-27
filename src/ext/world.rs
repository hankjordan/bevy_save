use bevy::{
    prelude::*,
    reflect::TypeRegistry,
    tasks::block_on,
};

use crate::{
    checkpoint::Checkpoints,
    error::Error,
    prelude::*,
    serde::{
        SnapshotDeserializer,
        SnapshotSerializer,
    },
};

/// Extension trait that adds save-related methods to Bevy's [`World`].
pub trait WorldSaveableExt: Sized {
    /// Captures a [`Snapshot`] from the current [`World`] state.
    fn snapshot<P: Pipeline>(&self, pipeline: P) -> Snapshot;

    /// Saves the application state with the given [`Pipeline`].
    ///
    /// # Errors
    /// - See [`Error`]
    fn save<P: Pipeline>(&self, pipeline: P) -> Result<(), Error>;

    /// Saves the application state with the given [`Pipeline`] and [`TypeRegistry`].
    ///
    /// # Errors
    /// - See [`Error`]
    fn save_with<P: Pipeline>(&self, pipeline: P, registry: &TypeRegistry) -> Result<(), Error>;

    /// Loads the application state with the given [`Pipeline`].
    ///
    /// # Errors
    /// - See [`Error`]
    fn load<P: Pipeline>(&mut self, pipeline: P) -> Result<(), Error>;

    /// Loads the application state with the given [`Pipeline`] and [`TypeRegistry`].
    ///
    /// # Errors
    /// - See [`Error`]
    fn load_with<P: Pipeline>(&mut self, pipeline: P, registry: &TypeRegistry)
        -> Result<(), Error>;
}

impl WorldSaveableExt for World {
    fn snapshot<P: Pipeline>(&self, pipeline: P) -> Snapshot {
        pipeline.capture(Snapshot::builder(self))
    }

    fn save<P: Pipeline>(&self, pipeline: P) -> Result<(), Error> {
        let registry = self.resource::<AppTypeRegistry>().read();
        self.save_with(pipeline, &registry)
    }

    fn save_with<P: Pipeline>(&self, pipeline: P, registry: &TypeRegistry) -> Result<(), Error> {
        let backend = self.resource::<P::Backend>();
        let snapshot = pipeline.capture(Snapshot::builder(self));
        let ser = SnapshotSerializer::new(&snapshot, registry);

        block_on(backend.save::<P::Format, _>(pipeline.key(), &ser))
    }

    fn load<P: Pipeline>(&mut self, pipeline: P) -> Result<(), Error> {
        let app_type_registry = self.resource::<AppTypeRegistry>().clone();
        let type_registry = app_type_registry.read();
        self.load_with(pipeline, &type_registry)
    }

    fn load_with<P: Pipeline>(
        &mut self,
        pipeline: P,
        registry: &TypeRegistry,
    ) -> Result<(), Error> {
        let backend = self.resource::<P::Backend>();
        let de = SnapshotDeserializer { registry };
        let snapshot = block_on(backend.load::<P::Format, _, _>(pipeline.key(), de))?;

        pipeline.apply(self, &snapshot)
    }
}

/// Extension trait that adds rollback checkpoint-related methods to Bevy's [`World`].
pub trait WorldCheckpointExt {
    /// Creates a checkpoint for rollback and stores it in [`Checkpoints`].
    fn checkpoint<P: Pipeline>(&mut self, pipeline: P);

    /// Rolls back / forward the [`World`] state.
    ///
    /// # Errors
    /// - See [`Error`]
    fn rollback<P: Pipeline>(&mut self, pipeline: P, checkpoints: isize) -> Result<(), Error>;

    /// Rolls back / forward the [`World`] state using the given [`TypeRegistry`].
    ///
    /// # Errors
    /// - See [`Error`]
    fn rollback_with<P: Pipeline>(
        &mut self,
        pipeline: P,
        checkpoints: isize,
        registry: &TypeRegistry,
    ) -> Result<(), Error>;
}

impl WorldCheckpointExt for World {
    fn checkpoint<P: Pipeline>(&mut self, pipeline: P) {
        let rollback = pipeline.capture(SnapshotBuilder::checkpoint(self));
        self.resource_mut::<Checkpoints>().checkpoint(rollback);
    }

    fn rollback<P: Pipeline>(&mut self, pipeline: P, checkpoints: isize) -> Result<(), Error> {
        let app_type_registry = self.resource::<AppTypeRegistry>().clone();
        let type_registry = app_type_registry.read();

        self.rollback_with(pipeline, checkpoints, &type_registry)
    }

    fn rollback_with<P: Pipeline>(
        &mut self,
        pipeline: P,
        checkpoints: isize,
        registry: &TypeRegistry,
    ) -> Result<(), Error> {
        if let Some(rollback) = self
            .resource_mut::<Checkpoints>()
            .rollback(checkpoints)
            .map(|r| r.clone_reflect(registry))
        {
            pipeline.apply(self, &rollback)
        } else {
            Ok(())
        }
    }
}
