use thiserror::Error;

/// An error that may occur when loading saves or rollbacks.
#[derive(Error, Debug)]
pub enum SaveableError {
    /// A Component was not registered in the AppTypeRegistry.
    #[error("scene contains the unregistered component `{type_name}`. you must add `#[reflect(Component)]` to your type")]
    UnregisteredComponent {
        /// The type name of the unregistered Component
        type_name: String,
    },

    /// A Resource was not registered in the AppTypeRegistry.
    #[error("scene contains the unregistered resource `{type_name}`. you must add `#[reflect(Resource)]` to your type")]
    UnregisteredResource {
        /// The type name of the unregistered Resource
        type_name: String,
    },

    /// A type was not registered in the AppTypeRegistry.
    #[error("scene contains the unregistered type `{type_name}`. you must register the type using `app.register_type::<T>()`")]
    UnregisteredType {
        /// The type name of the unregistered type
        type_name: String,
    },

    /// Deserialization failure or other error.
    #[error("other error: {0}")]
    Other(Box<dyn std::error::Error>),
}

impl SaveableError {
    /// Deserialization failure or other error.
    pub fn other(error: impl std::error::Error + 'static) -> Self {
        Self::Other(Box::new(error))
    }
}
