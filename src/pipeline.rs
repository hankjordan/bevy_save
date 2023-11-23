use bevy::prelude::*;

use crate::{
    prelude::*,
    Error,
};

/// Trait that defines how exactly your app saves and loads.
pub trait Pipeline: Sized {
    /// The interface between the saver / loader and data storage.
    type Backend: for<'a> Backend<Self::Key<'a>> + Resource + Default;
    /// The format used for serializing and deserializing data.
    type Format: Format;

    /// Used to uniquely identify each saved [`Snapshot`].
    type Key<'a>;

    /// Called when the pipeline is initialized with [`App::init_pipeline`](`AppSaveableExt::init_pipeline`).
    fn build(app: &mut App) {
        app.world.insert_resource(Self::Backend::default());
    }

    /// Retrieve the unique identifier for the [`Snapshot`] being processed by the [`Pipeline`].
    fn key(&self) -> Self::Key<'_>;

    /// Retrieve a [`Snapshot`] from the [`World`].
    /// 
    /// This is where you would do any special filtering you might need.
    /// 
    /// You must extract [`Rollbacks`] if you want this pipeline to handle rollbacks properly.
    fn capture(world: &World) -> Snapshot {
        Snapshot::from_world(world)
    }

    /// Retrieve a [`Snapshot`] from the [`World`], using the [`Pipeline`] as a seed.
    /// 
    /// This is where you would do any special filtering you might need.
    /// 
    /// You must extract [`Rollbacks`] if you want this pipeline to handle rollbacks properly.
    fn capture_seed(&self, world: &World) -> Snapshot {
        Self::capture(world)
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
    /// Entity mapping goes here, along with your spawn hook and any other transformations you might need to perform.
    /// 
    /// # Errors
    /// If a type included in the [`Snapshot`] has not been registered with the type registry.
    fn apply_seed(&self, world: &mut World, snapshot: &Snapshot) -> Result<(), Error> {
        Self::apply(world, snapshot)
    }
}

impl<'a> Pipeline for &'a str {
    type Backend = DefaultBackend;
    type Format = DefaultFormat;

    type Key<'k> = &'k str;

    fn key(&self) -> Self::Key<'_> {
        self
    }
}

/// Uses `JSON` and saves to the given local path.
pub struct DebugPipeline<'a>(pub &'a str);

impl<'a> Pipeline for DebugPipeline<'a> {
    type Backend = DefaultDebugBackend;
    type Format = DefaultDebugFormat;

    type Key<'k> = &'k str;

    fn key(&self) -> Self::Key<'_> {
        self.0
    }
}
