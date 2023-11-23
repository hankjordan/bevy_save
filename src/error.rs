use bevy::prelude::*;
use thiserror::Error;

/// An error that may occur when loading saves or rollbacks.
#[derive(Error, Debug)]
pub enum Error {
    /// Saving or serialization error.
    #[error("error occurred while saving")]
    Saving,

    /// Loading or deserialization error.
    #[error("error occurred while loading")]
    Loading,

    /// Scene spawning error.
    #[error("scene spawn error: {0}")]
    SceneSpawnError(bevy::scene::SceneSpawnError),

    /// IO / Filesystem error.
    #[error("io error: {0}")]
    IO(std::io::Error),

    /// Other error.
    #[error("other error: {0}")]
    Other(Box<dyn std::error::Error>),

    /// Custom error.
    #[error("custom error: {0}")]
    Custom(String),
}

impl Error {
    /// Saving or serialization error.
    pub fn saving(err: impl std::error::Error) -> Self {
        // TODO
        error!("Saving error: {err:?}");
        Self::Saving
    }

    /// Loading or deserialization error.
    pub fn loading(err: impl std::error::Error) -> Self {
        // TODO
        error!("Loading error: {err:?}");
        Self::Loading
    }

    /// Other error.
    pub fn other(error: impl std::error::Error + 'static) -> Self {
        Self::Other(Box::new(error))
    }

    /// Custom error.
    pub fn custom(error: impl std::fmt::Display) -> Self {
        Self::Custom(format!("{error}"))
    }
}

impl From<bevy::scene::SceneSpawnError> for Error {
    fn from(value: bevy::scene::SceneSpawnError) -> Self {
        Self::SceneSpawnError(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}
