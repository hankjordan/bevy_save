//! [`Pipeline`] connects all of the pieces together, defining how your
//! application state is captured, applied, saved, and loaded.

use bevy::prelude::*;

use crate::{
    backend::AppBackend,
    error::Error,
    plugins::SaveReflectPlugin,
    prelude::*,
};

/// Trait that defines how exactly your app saves and loads.
pub trait Pipeline {
    /// The interface between the saver / loader and data storage.
    type Backend: for<'a> Backend<Self::Key<'a>> + FromWorld + Send + Sync + 'static;
    /// The format used for serializing and deserializing data.
    type Format: Format;

    /// Used to uniquely identify each saved [`Snapshot`].
    type Key<'a>;

    /// Called when the pipeline is initialized with
    /// [`App::init_pipeline`](`AppPipelineExt::init_pipeline`).
    fn build(app: &mut App) {
        let backend = Self::Backend::from_world(app.world_mut());
        app.world_mut().insert_resource(AppBackend(backend));
    }

    /// Retrieve the unique identifier for the [`Snapshot`] being processed by
    /// the [`Pipeline`].
    fn key(&self) -> Self::Key<'_>;

    /// Retrieve a [`Snapshot`] from the [`World`].
    ///
    /// This is where you would do any special filtering you might need.
    ///
    /// You must extract
    /// [`Checkpoints`](crate::reflect::checkpoint::Checkpoints) if you want
    /// this pipeline to handle checkpoints properly.
    fn capture(&self, builder: BuilderRef) -> Snapshot;

    /// Apply a [`Snapshot`] to the [`World`].
    ///
    /// Entity mapping goes here, along with your spawn hook and any other
    /// transformations you might need to perform.
    ///
    /// # Errors
    /// If a type included in the [`Snapshot`] has not been registered with the
    /// type registry.
    fn apply(&self, world: &mut World, snapshot: &Snapshot) -> Result<(), Error>;
}

/// Extension trait that adds pipeline-related methods to Bevy's [`App`].
pub trait AppPipelineExt {
    /// Initialize a [`Pipeline`], allowing it to be used with global save and
    /// load methods.
    fn init_pipeline<P: Pipeline>(&mut self) -> &mut Self;
}

impl AppPipelineExt for App {
    fn init_pipeline<P: Pipeline>(&mut self) -> &mut Self {
        // `Snapshot` must be registered to use the pipeline
        if !self.is_plugin_added::<SaveReflectPlugin>() {
            self.add_plugins(SaveReflectPlugin);
        }

        P::build(self);
        self
    }
}
