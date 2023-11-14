use thiserror::Error;

/// An error that may occur when loading saves or rollbacks.
#[derive(Error, Debug)]
pub enum Error {
    /// A Component was not registered in the AppTypeRegistry.
    #[error("scene contains the unregistered component `{type_path}`. you must add `#[reflect(Component)]` to your type")]
    UnregisteredComponent {
        /// The type name of the unregistered Component
        type_path: String,
    },

    /// A Resource was not registered in the AppTypeRegistry.
    #[error("scene contains the unregistered resource `{type_path}`. you must add `#[reflect(Resource)]` to your type")]
    UnregisteredResource {
        /// The type name of the unregistered Resource
        type_path: String,
    },

    /// A type was not registered in the AppTypeRegistry.
    #[error("scene contains the unregistered type `{type_path}`. you must register the type using `app.register_type::<T>()`")]
    UnregisteredType {
        /// The type name of the unregistered type
        type_path: String,
    },

    /// Saving or serialization error.
    #[error("error occurred while saving")]
    Saving,

    /// Loading or deserialization error.
    #[error("error occurred while loading")]
    Loading,

    /// IO / Filesystem error.
    #[error("io error: {0}")]
    IO(std::io::Error),

    /// Other error.
    #[error("other error: {0}")]
    Other(Box<dyn std::error::Error>),
}

impl Error {
    /// Saving or serialization error.
    pub fn saving(_: impl std::error::Error) -> Self {
        Self::Saving
    }

    /// Loading or deserialization error.
    pub fn loading(_: impl std::error::Error) -> Self {
        Self::Loading
    }

    /// Other error.
    pub fn other(error: impl std::error::Error + 'static) -> Self {
        Self::Other(Box::new(error))
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}
