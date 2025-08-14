use bevy::prelude::*;
use thiserror::Error;

/// An error that may occur when loading saves, snapshots, or checkpoints.
#[derive(Error, Debug)]
pub enum Error {
    /// Saving or serialization error.
    #[error("saving error: {0}")]
    Saving(Box<dyn std::error::Error + Send + Sync>),

    /// Loading or deserialization error.
    #[error("loading error: {0}")]
    Loading(Box<dyn std::error::Error + Send + Sync>),

    /// Flow error.
    #[error("flow error: {0}")]
    Flow(#[from] crate::flows::FlowError),

    #[cfg(feature = "reflect")]
    /// Scene spawning error.
    #[error("scene spawn error: {0}")]
    SceneSpawnError(#[from] bevy::scene::SceneSpawnError),

    /// IO / Filesystem error.
    #[error("io error: {0}")]
    IO(#[from] std::io::Error),

    /// Other error.
    #[error("other error: {0}")]
    Other(Box<dyn std::error::Error + Send + Sync>),

    /// Custom error.
    #[error("custom error: {0}")]
    Custom(String),
}

impl Error {
    /// Saving or serialization error.
    pub fn saving(err: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Saving(Box::new(err))
    }

    /// Loading or deserialization error.
    pub fn loading(err: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Loading(Box::new(err))
    }

    /// Other error.
    pub fn other(error: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Other(Box::new(error))
    }

    /// Custom error.
    pub fn custom(error: impl std::fmt::Display) -> Self {
        Self::Custom(format!("{error}"))
    }
}
