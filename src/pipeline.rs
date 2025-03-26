//! [`Pipeline`] connects all of the pieces together, defining how your application state is captured, applied, saved, and loaded.

use bevy::prelude::*;

use crate::{
    backend::{
        DefaultBackend,
        DefaultDebugBackend,
    },
    error::Error,
    format::{
        DefaultDebugFormat,
        DefaultFormat,
    },
    prelude::*,
};

/// Trait that defines how exactly your app saves and loads.
pub trait Pipeline: Send + Sized + 'static {
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
    fn capture(builder: SnapshotBuilder) -> Snapshot {
        builder.build()
    }

    /// Retrieve a [`Snapshot`] from the [`World`], using the [`Pipeline`] as a seed.
    ///
    /// This is usually used for partial snapshots.
    ///
    /// This is where you would do any special filtering you might need.
    ///
    /// You must extract [`Checkpoints`](crate::checkpoint::Checkpoints) if you want this pipeline to handle checkpoints properly.
    fn capture_seed(&self, builder: SnapshotBuilder) -> Snapshot {
        Self::capture(builder)
    }

    /// Apply a [`Snapshot`] to the [`World`].
    ///
    /// Entity mapping goes here, along with your spawn hook and any other transformations you might need to perform.
    ///
    /// # Errors
    /// If a type included in the [`Snapshot`] has not been registered with the type registry.
    fn apply(world: &mut World, snapshot: &Snapshot) -> Result<(), Error> {
        snapshot.apply(world)
    }

    /// Apply a [`Snapshot`] to the [`World`], using the [`Pipeline`] as a seed.
    ///
    /// This is usually used for partial snapshots.
    ///
    /// Entity mapping goes here, along with your spawn hook and any other transformations you might need to perform.
    ///
    /// # Errors
    /// If a type included in the [`Snapshot`] has not been registered with the type registry.
    fn apply_seed(&self, world: &mut World, snapshot: &Snapshot) -> Result<(), Error> {
        Self::apply(world, snapshot)
    }
}

/// Uses [`DefaultFormat`] and saves with [`DefaultBackend`].
pub struct DefaultPipeline(pub String);

impl Pipeline for DefaultPipeline {
    type Backend = DefaultBackend;
    type Format = DefaultFormat;

    type Key<'k> = &'k str;

    fn key(&self) -> Self::Key<'_> {
        &self.0
    }
}

/// Uses [`DefaultDebugFormat`] and saves with [`DefaultDebugBackend`].
pub struct DefaultDebugPipeline(pub String);

impl Pipeline for DefaultDebugPipeline {
    type Backend = DefaultDebugBackend;
    type Format = DefaultDebugFormat;

    type Key<'k> = &'k str;

    fn key(&self) -> Self::Key<'_> {
        &self.0
    }
}
