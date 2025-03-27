//! [`Pipeline`] connects all of the pieces together, defining how your application state is captured, applied, saved, and loaded.

use bevy::prelude::*;

use crate::{
    error::Error,
    prelude::*,
};

/// Trait that defines how exactly your app saves and loads.
pub trait Pipeline {
    /// The interface between the saver / loader and data storage.
    type Backend: for<'a> Backend<Self::Key<'a>> + Resource + Default;
    /// The format used for serializing and deserializing data.
    type Format: Format;

    /// Used to uniquely identify each saved [`Snapshot`].
    type Key<'a>;

    /// Called when the pipeline is initialized with [`App::init_pipeline`](`AppSaveableExt::init_pipeline`).
    fn build(app: &mut App) {
        app.world_mut().insert_resource(Self::Backend::default());
    }

    /// Retrieve the unique identifier for the [`Snapshot`] being processed by the [`Pipeline`].
    fn key(&self) -> Self::Key<'_>;

    /// Retrieve a [`Snapshot`] from the [`World`].
    ///
    /// This is where you would do any special filtering you might need.
    ///
    /// You must extract [`Checkpoints`](crate::checkpoint::Checkpoints) if you want this pipeline to handle checkpoints properly.
    fn capture(&self, builder: SnapshotBuilder) -> Snapshot;

    /// Apply a [`Snapshot`] to the [`World`].
    ///
    /// Entity mapping goes here, along with your spawn hook and any other transformations you might need to perform.
    ///
    /// # Errors
    /// If a type included in the [`Snapshot`] has not been registered with the type registry.
    fn apply(&self, world: &mut World, snapshot: &Snapshot) -> Result<(), Error>;
}
