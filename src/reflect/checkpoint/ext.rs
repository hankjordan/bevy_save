use std::any::Any;

use bevy::{
    prelude::*,
    reflect::TypeRegistry,
};

use crate::{
    prelude::*,
    reflect::checkpoint::{
        CheckpointRegistry,
        Checkpoints,
    },
};

/// Extension trait that adds rollback checkpoint-related methods to Bevy's [`App`].
pub trait AppCheckpointExt {
    /// Set a type to allow rollback - it will be included in rollback checkpoints and affected by save/load.
    fn allow_checkpoint<T: Any>(&mut self) -> &mut Self;

    /// Set a type to ignore rollback - it will be included in save/load but it won't change during rollback.
    fn deny_checkpoint<T: Any>(&mut self) -> &mut Self;
}

impl AppCheckpointExt for App {
    fn allow_checkpoint<T: Any>(&mut self) -> &mut Self {
        self.world_mut()
            .resource_mut::<CheckpointRegistry>()
            .allow::<T>();
        self
    }

    fn deny_checkpoint<T: Any>(&mut self) -> &mut Self {
        self.world_mut()
            .resource_mut::<CheckpointRegistry>()
            .deny::<T>();
        self
    }
}

/// Extension trait that adds rollback checkpoint-related methods to Bevy's [`World`].
pub trait WorldCheckpointExt {
    /// Creates a checkpoint for rollback and stores it in [`Checkpoints`].
    fn checkpoint<P>(&mut self, pathway: &P)
    where
        P: Pathway<
            Capture: CaptureInput<P, Builder: Into<Builder> + From<Builder>> + Into<Snapshot>,
        >;

    /// Rolls back / forward the [`World`] state.
    ///
    /// # Errors
    /// - See [`Error`]
    fn rollback<P>(&mut self, pathway: &P, checkpoints: isize) -> Result<(), Error>
    where
        P: Pathway<Capture: CaptureOutput<P> + From<Snapshot>>;

    /// Rolls back / forward the [`World`] state using the given [`TypeRegistry`].
    ///
    /// # Errors
    /// - See [`Error`]
    fn rollback_with<P>(
        &mut self,
        pathway: &P,
        checkpoints: isize,
        registry: &TypeRegistry,
    ) -> Result<(), Error>
    where
        P: Pathway<Capture: CaptureOutput<P> + From<Snapshot>>;
}

impl WorldCheckpointExt for World {
    fn checkpoint<P>(&mut self, pathway: &P)
    where
        P: Pathway<
            Capture: CaptureInput<P, Builder: Into<Builder> + From<Builder>> + Into<Snapshot>,
        >,
    {
        let builder = P::Capture::builder(pathway, self).into().into_checkpoint();
        let rollback = self.capture_with(pathway, builder.into());
        self.resource_mut::<Checkpoints>()
            .checkpoint(rollback.into());
    }

    fn rollback<P>(&mut self, pathway: &P, checkpoints: isize) -> Result<(), Error>
    where
        P: Pathway<Capture: CaptureOutput<P> + From<Snapshot>>,
    {
        let registry_arc = self.resource::<AppTypeRegistry>().clone();
        let registry = registry_arc.read();

        self.rollback_with(pathway, checkpoints, &registry)
    }

    fn rollback_with<P>(
        &mut self,
        pathway: &P,
        checkpoints: isize,
        registry: &TypeRegistry,
    ) -> Result<(), Error>
    where
        P: Pathway<Capture: CaptureOutput<P> + From<Snapshot>>,
    {
        if let Some(rollback) = self
            .resource_mut::<Checkpoints>()
            .rollback(checkpoints)
            .map(|r| r.clone_reflect(registry))
        {
            self.apply(pathway, rollback.into()).map(|_| ())
        } else {
            Ok(())
        }
    }
}
