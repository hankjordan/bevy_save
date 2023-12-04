use serde::{
    de::DeserializeSeed,
    Serialize,
};

use crate::{
    prelude::*,
    Error,
};

/// Interface between the [`Format`] and the disk or other storage.
pub trait Backend<K> {
    /// Attempts to serialize a value with the given [`Format`].
    ///
    /// # Errors
    /// - [`Error::Saving`] if serialization of the type fails
    /// - [`Error::IO`] if there is an IO or filesystem failure
    /// - See [`Error`]
    fn save<F: Format, T: Serialize>(&self, key: K, value: &T) -> Result<(), Error>;

    /// Attempts to deserialize a value with the given [`Format`].
    ///
    /// # Errors
    /// - [`Error::Loading`] if deserialization of the type fails
    /// - [`Error::IO`] if there is an IO or filesystem failure
    /// - See [`Error`]
    fn load<F: Format, S: for<'de> DeserializeSeed<'de, Value = T>, T>(
        &self,
        key: K,
        seed: S,
    ) -> Result<T, Error>;
}

#[cfg(not(target_arch = "wasm32"))]
mod desktop {
    use std::{
        fs::File,
        io::{
            BufReader,
            BufWriter,
        },
    };

    use bevy::prelude::*;

    #[allow(clippy::wildcard_imports)]
    use super::*;
    use crate::dir::get_save_file;

    /// Simple filesystem backend.
    ///
    /// Each name corresponds to an individual file on the disk.
    ///
    /// Files are stored in [`SAVE_DIR`](crate::SAVE_DIR).
    #[derive(Default, Resource)]
    pub struct FileIO;

    impl<K: std::fmt::Display> Backend<K> for FileIO {
        fn save<F: Format, T: Serialize>(&self, key: K, value: &T) -> Result<(), Error> {
            let path = get_save_file(format!("{key}{}", F::extension()));
            let dir = path.parent().expect("Invalid save directory");

            std::fs::create_dir_all(dir)?;

            let file = File::create(path)?;
            let writer = BufWriter::new(file);

            F::serialize(writer, value)
        }

        fn load<F: Format, S: for<'de> DeserializeSeed<'de, Value = T>, T>(
            &self,
            key: K,
            seed: S,
        ) -> Result<T, Error> {
            let path = get_save_file(format!("{key}{}", F::extension()));
            let file = File::open(path)?;
            let reader = BufReader::new(file);

            F::deserialize(reader, seed)
        }
    }

    /// Debug filesystem backend.
    ///
    /// Each name corresponds to an individual file on the disk.
    ///
    /// Files are stored relative to the active path.
    #[derive(Default, Resource)]
    pub struct DebugFileIO;

    impl<K: std::fmt::Display> Backend<K> for DebugFileIO {
        fn save<F: Format, T: Serialize>(&self, key: K, value: &T) -> Result<(), Error> {
            let file = File::create(format!("{key}{}", F::extension()))?;
            let writer = BufWriter::new(file);

            F::serialize(writer, value)
        }

        fn load<F: Format, S: for<'de> DeserializeSeed<'de, Value = T>, T>(
            &self,
            key: K,
            seed: S,
        ) -> Result<T, Error> {
            let file = File::open(format!("{key}{}", F::extension()))?;
            let reader = BufReader::new(file);

            F::deserialize(reader, seed)
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use desktop::*;
#[cfg(not(target_arch = "wasm32"))]
/// The [`Backend`] the default [`DynamicPipeline`](crate::DynamicPipeline) will use.
pub type DefaultBackend = desktop::FileIO;
#[cfg(not(target_arch = "wasm32"))]
/// The [`Backend`] the default debug [`DynamicPipeline`](crate::DynamicPipeline) will use.
pub type DefaultDebugBackend = desktop::DebugFileIO;

#[cfg(target_arch = "wasm32")]
mod wasm {
    use bevy::prelude::*;
    use fragile::Fragile;
    use web_sys::Storage;

    #[allow(clippy::wildcard_imports)]
    use super::*;
    use crate::WORKSPACE;

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
        fn save<F: Format, T: Serialize>(&self, key: &str, value: &T) -> Result<(), Error> {
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

        fn load<F: Format, S: for<'de> DeserializeSeed<'de, Value = T>, T>(
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
pub use wasm::*;
#[cfg(target_arch = "wasm32")]
/// The [`Backend`] the default [`Pipeline`](crate::Pipeline) will use.
pub type DefaultBackend = wasm::WebStorage;
#[cfg(target_arch = "wasm32")]
/// The [`Backend`] the default debug [`Pipeline`](crate::Pipeline) will use.
pub type DefaultDebugBackend = wasm::WebStorage;
