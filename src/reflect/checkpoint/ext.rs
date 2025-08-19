use bevy::prelude::*;

use crate::{
    prelude::*,
    reflect::checkpoint::Checkpoints,
};

/// Extension trait that adds rollback checkpoint-related methods to Bevy's
/// [`World`].
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
        if let Some(rollback) = self
            .resource_mut::<Checkpoints>()
            .rollback(checkpoints)
            .cloned()
        {
            self.apply(pathway, rollback.into()).map(|_| ())
        } else {
            Ok(())
        }
    }
}
