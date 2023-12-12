use bevy::prelude::*;

use crate::{
    dynamic::{
        CloneReflect,
        DynamicSnapshotDeserializer,
        DynamicSnapshotSerializer,
    },
    pipeline::Pipeline,
    prelude::*,
    typed::extract::{
        ExtractDeserialize,
        ExtractSerialize,
    },
    Error,
};

/// Extension trait that adds save-related methods to Bevy's [`World`].
pub trait WorldSaveableExt {
    /// Captures a [`DynamicSnapshot`] from the current [`World`] state.
    fn snapshot<P: DynamicPipeline>(&self) -> DynamicSnapshot;

    /// Saves the game state with the given [`DynamicPipeline`].
    ///
    /// # Errors
    /// - See [`Error`]
    fn save<P: DynamicPipeline>(&self, pipeline: P) -> Result<(), Error>;

    /// Saves the game state with the given [`Pipeline`].
    ///
    /// # Errors
    /// - See [`Error`]
    fn save_typed<P>(&self, pipeline: P) -> Result<(), Error>
    where
        P: Pipeline,
        P::Components: ExtractSerialize,
        P::Resources: ExtractSerialize;

    /// Loads the game state with the given [`DynamicPipeline`].
    ///
    /// # Errors
    /// - See [`Error`]
    fn load<P: DynamicPipeline>(&mut self, pipeline: P) -> Result<(), Error>;

    /// Loads the game state with the given [`Pipeline`].
    ///
    /// # Errors
    /// - See [`Error`]
    fn load_typed<P>(&mut self, pipeline: P) -> Result<(), Error>
    where
        P: Pipeline,
        P::Components: ExtractDeserialize,
        P::Resources: ExtractDeserialize;
}

impl WorldSaveableExt for World {
    fn snapshot<P: DynamicPipeline>(&self) -> DynamicSnapshot {
        P::capture(DynamicSnapshot::builder(self))
    }

    fn save<P: DynamicPipeline>(&self, pipeline: P) -> Result<(), Error> {
        let registry = self.resource::<AppTypeRegistry>();
        let backend = self.resource::<P::Backend>();

        let snapshot = pipeline.capture_seed(DynamicSnapshot::builder(self));

        let ser = DynamicSnapshotSerializer::new(&snapshot, registry);

        backend.save::<P::Format, _>(pipeline.key(), &ser)
    }

    fn save_typed<P>(&self, pipeline: P) -> Result<(), Error>
    where
        P: Pipeline,
        P::Components: ExtractSerialize,
        P::Resources: ExtractSerialize,
    {
        let backend = self.resource::<P::Backend>();

        let snapshot = pipeline.capture_seed(self);

        backend.save::<P::Format, _>(pipeline.key(), &snapshot)
    }

    fn load<P: DynamicPipeline>(&mut self, pipeline: P) -> Result<(), Error> {
        let registry = self.resource::<AppTypeRegistry>().clone();
        let reg = registry.read();
        let backend = self.resource::<P::Backend>();

        let de = DynamicSnapshotDeserializer { registry: &reg };

        let snapshot = backend.load::<P::Format, _, _>(pipeline.key(), de)?;

        pipeline.apply_seed(self, &snapshot)
    }

    fn load_typed<P>(&mut self, pipeline: P) -> Result<(), Error>
    where
        P: Pipeline,
        P::Components: ExtractDeserialize,
        P::Resources: ExtractDeserialize,
    {
        let backend = self.resource::<P::Backend>();

        let snapshot = backend.load::<P::Format, _, _>(pipeline.key(), std::marker::PhantomData)?;

        pipeline.apply_seed(self, &snapshot);

        Ok(())
    }
}

/// Extension trait that adds rollback-related methods to Bevy's [`World`].
pub trait WorldRollbackExt {
    /// Creates a checkpoint for rollback.
    fn checkpoint<P: DynamicPipeline>(&mut self);

    /// Rolls back / forward the [`World`] state.
    ///
    /// # Errors
    /// - See [`Error`]
    fn rollback<P: DynamicPipeline>(&mut self, checkpoints: isize) -> Result<(), Error>;
}

impl WorldRollbackExt for World {
    fn checkpoint<P: DynamicPipeline>(&mut self) {
        let rollback = P::capture(DynamicSnapshotBuilder::rollback(self));
        self.resource_mut::<Rollbacks>().checkpoint(rollback);
    }

    fn rollback<P: DynamicPipeline>(&mut self, checkpoints: isize) -> Result<(), Error> {
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
