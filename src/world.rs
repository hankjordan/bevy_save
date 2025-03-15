use bevy::{
    prelude::*,
    reflect::TypeRegistry,
};

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
    fn snapshot<P: Pipeline>(&self) -> Snapshot {
        P::capture(Snapshot::builder(self))
    }

    fn save<P: Pipeline>(&self, pipeline: P) -> Result<(), Error> {
        let registry = self.resource::<AppTypeRegistry>().read();
        self.save_with(pipeline, &registry)
    }

    fn save_with<P: Pipeline>(&self, pipeline: P, registry: &TypeRegistry) -> Result<(), Error> {
        let backend = self.resource::<P::Backend>();
        let snapshot = pipeline.capture_seed(Snapshot::builder(self));
        let ser = SnapshotSerializer::new(&snapshot, registry);

        backend.save::<P::Format, _>(pipeline.key(), &ser)
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

    /// Rolls back / forward the [`World`] state using the given [`TypeRegistry`].
    ///
    /// # Errors
    /// - See [`Error`]
    fn rollback_with<P: Pipeline>(
        &mut self,
        checkpoints: isize,
        registry: &TypeRegistry,
    ) -> Result<(), Error>;
}

impl WorldRollbackExt for World {
    fn checkpoint<P: Pipeline>(&mut self) {
        let rollback = P::capture(SnapshotBuilder::rollback(self));
        self.resource_mut::<Rollbacks>().checkpoint(rollback);
    }

    fn rollback<P: Pipeline>(&mut self, checkpoints: isize) -> Result<(), Error> {
        let app_type_registry = self.resource::<AppTypeRegistry>().clone();
        let type_registry = app_type_registry.read();

        self.rollback_with::<P>(checkpoints, &type_registry)
    }

    fn rollback_with<P: Pipeline>(
        &mut self,
        checkpoints: isize,
        registry: &TypeRegistry,
    ) -> Result<(), Error> {
        if let Some(rollback) = self
            .resource_mut::<Rollbacks>()
            .rollback(checkpoints)
            .map(|r| r.clone_reflect(registry))
        {
            P::apply(self, &rollback)
        } else {
            Ok(())
        }
    }
}
