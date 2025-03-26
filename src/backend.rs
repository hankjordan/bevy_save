//! [`Backend`] acts as an interface between [`Format`] and storage for persisting values.

use bevy::utils::{
    ConditionalSend,
    ConditionalSendFuture,
};
use serde::{
    de::DeserializeSeed,
    Serialize,
};

use crate::{
    error::Error,
    prelude::*,
};

/// Interface between the [`Format`] and the disk or other storage.
///
/// # Implementation
/// The preferred style for implementing this method is an `async fn` returning a result.
/// ```
/// # use bevy::utils::ConditionalSend;
/// # use serde::{de::DeserializeSeed, Serialize};
/// # use bevy_save::prelude::*;
/// #
/// # pub struct ExampleBackend;
/// #
/// impl<K: Send> Backend<K> for ExampleBackend {
///     async fn save<F: Format, T: Serialize + ConditionalSend + Sync>(
///         &self,
///         key: K,
///         value: &T
///     ) -> Result<(), Error> {
///         // ...
///         # Err(Error::custom("Unimplemented"))
///     }
///
///     async fn load<F: Format, S: for<'de> DeserializeSeed<'de, Value = T> + ConditionalSend, T>(
///         &self,
///         key: K,
///         seed: S,
///     ) -> Result<T, Error> {
///         // ...
///         # Err(Error::custom("Unimplemented"))
///     }
/// }
/// ```
pub trait Backend<K> {
    /// Attempts to serialize a value with the given [`Format`].
    ///
    /// # Errors
    /// - [`Error::Saving`] if serialization of the type fails
    /// - [`Error::IO`] if there is an IO or filesystem failure
    /// - See [`Error`]
    fn save<F: Format, T: Serialize + ConditionalSend + Sync>(
        &self,
        key: K,
        value: &T,
    ) -> impl ConditionalSendFuture<Output = Result<(), Error>>;

    /// Attempts to deserialize a value with the given [`Format`].
    ///
    /// # Errors
    /// - [`Error::Loading`] if deserialization of the type fails
    /// - [`Error::IO`] if there is an IO or filesystem failure
    /// - See [`Error`]
    fn load<F: Format, S: for<'de> DeserializeSeed<'de, Value = T> + ConditionalSend, T>(
        &self,
        key: K,
        seed: S,
    ) -> impl ConditionalSendFuture<Output = Result<T, Error>>;
}

#[cfg(not(target_arch = "wasm32"))]
mod desktop {
    use async_std::{
        fs::{
            create_dir_all,
            File,
        },
        io::{
            ReadExt,
            WriteExt,
        },
    };
    use bevy::prelude::*;

    #[allow(clippy::wildcard_imports)]
    use super::*;

    /// Simple filesystem backend.
    ///
    /// Each name corresponds to an individual file on the disk.
    ///
    /// Files are stored in `SAVE_DIR`.
    #[derive(Default, Resource)]
    pub struct FileIO;

    impl<K: std::fmt::Display + Send> Backend<K> for FileIO {
        async fn save<F: Format, T: Serialize>(&self, key: K, value: &T) -> Result<(), Error> {
            let path = get_save_file(format!("{key}{}", F::extension()));
            let dir = path.parent().expect("Invalid save directory");

            create_dir_all(dir).await?;

            let mut buf = Vec::new();

            F::serialize(&mut buf, value)?;

            let mut file = File::create(path).await?;

            Ok(file.write_all(&buf).await?)
        }

        async fn load<F: Format, S: for<'de> DeserializeSeed<'de, Value = T>, T>(
            &self,
            key: K,
            seed: S,
        ) -> Result<T, Error> {
            let path = get_save_file(format!("{key}{}", F::extension()));

            let mut file = File::open(path).await?;
            let mut buf = Vec::new();

            file.read_to_end(&mut buf).await?;

            F::deserialize(&*buf, seed)
        }
    }

    /// Debug filesystem backend.
    ///
    /// Each name corresponds to an individual file on the disk.
    ///
    /// Files are stored relative to the active path.
    #[derive(Default, Resource)]
    pub struct DebugFileIO;

    impl<K: std::fmt::Display + Send> Backend<K> for DebugFileIO {
        async fn save<F: Format, T: Serialize>(&self, key: K, value: &T) -> Result<(), Error> {
            let mut buf = Vec::new();

            F::serialize(&mut buf, value)?;

            let mut file = File::create(format!("{key}{}", F::extension())).await?;

            Ok(file.write_all(&buf).await?)
        }

        async fn load<F: Format, S: for<'de> DeserializeSeed<'de, Value = T>, T>(
            &self,
            key: K,
            seed: S,
        ) -> Result<T, Error> {
            let mut file = File::open(format!("{key}{}", F::extension())).await?;
            let mut buf = Vec::new();

            file.read_to_end(&mut buf).await?;

            F::deserialize(&*buf, seed)
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use desktop::FileIO;
#[cfg(not(target_arch = "wasm32"))]
/// The [`Backend`] the default [`Pipeline`] will use.
pub type DefaultBackend = desktop::FileIO;
#[cfg(not(target_arch = "wasm32"))]
/// The [`Backend`] the default debug [`Pipeline`] will use.
pub type DefaultDebugBackend = desktop::DebugFileIO;

#[cfg(target_arch = "wasm32")]
mod wasm {
    use bevy::prelude::*;
    use fragile::Fragile;
    use serde::{
        de::DeserializeSeed,
        Serialize,
    };
    use web_sys::Storage;

    use crate::prelude::*;

    /// Simple `WebStorage` backend.
    #[derive(Resource)]
    pub struct WebStorage {
        storage: Fragile<Storage>,
    }

    impl Default for WebStorage {
        fn default() -> Self {
            Self {
                storage: Fragile::new(
                    web_sys::window()
                        .expect("No window")
                        .local_storage()
                        .expect("Failed to get local storage")
                        .expect("No local storage"),
                ),
            }
        }
    }

    impl<'a> Backend<&'a str> for WebStorage {
        async fn save<F: Format, T: Serialize>(&self, key: &str, value: &T) -> Result<(), Error> {
            let mut buf: Vec<u8> = Vec::new();

            F::serialize(&mut buf, value)?;

            self.storage
                .get()
                .set_item(
                    &format!("{WORKSPACE}.{key}"),
                    &serde_json::to_string(&buf).map_err(Error::saving)?,
                )
                .expect("Failed to save");

            Ok(())
        }

        async fn load<F: Format, S: for<'de> DeserializeSeed<'de, Value = T>, T>(
            &self,
            key: &str,
            seed: S,
        ) -> Result<T, Error> {
            let value = self
                .storage
                .get()
                .get_item(&format!("{WORKSPACE}.{key}"))
                .expect("Failed to load")
                .ok_or(Error::custom("Invalid key"))?;

            let buf: Vec<u8> = serde_json::from_str(&value).map_err(Error::loading)?;

            F::deserialize(&*buf, seed)
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm::WebStorage;
#[cfg(target_arch = "wasm32")]
/// The [`Backend`] the default [`Pipeline`] will use.
pub type DefaultBackend = wasm::WebStorage;
#[cfg(target_arch = "wasm32")]
/// The [`Backend`] the default debug [`Pipeline`] will use.
pub type DefaultDebugBackend = wasm::WebStorage;
