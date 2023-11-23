use serde::{
    de::DeserializeSeed,
    Serialize,
};

use crate::{
    Error,
    Format,
};

/// Interface between the [`Format`] and the disk or other storage.
pub trait Backend<K> {
    /// Attempts to serialize a value with the given [`Format`].
    ///
    /// # Errors
    /// - [`Error::Saving`] if serialization of the type fails
    /// - [`Error::IO`] if there is an IO or filesystem failure
    /// - See [`Error`]
    fn save<F: Format, T: Serialize>(&self, key: K, value: T) -> Result<(), Error>;

    /// Attempts to deserialize a value with the given [`Format`].
    ///
    /// # Errors
    /// - [`Error::Loading`] if deserialization of the type fails
    /// - [`Error::IO`] if there is an IO or filesystem failure
    /// - See [`Error`]
    fn load<'de, F: Format, T: DeserializeSeed<'de>>(
        &self,
        key: K,
        seed: T,
    ) -> Result<T::Value, Error>;
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
    use crate::{
        get_save_file,
        AsDeserializer,
        AsSerializer,
    };

    /// Simple filesystem backend.
    ///
    /// Each name corresponds to an individual file on the disk.
    ///
    /// Files are stored in `SAVE_DIR`.
    #[derive(Default, Resource)]
    pub struct FileIO;

    impl<K: std::fmt::Display> Backend<K> for FileIO {
        fn save<F: Format, T: Serialize>(&self, key: K, value: T) -> Result<(), Error> {
            let path = get_save_file(format!("{key}{}", F::extension()));
            let dir = path.parent().expect("Invalid save directory");

            std::fs::create_dir_all(dir)?;

            let file = File::create(path)?;
            let writer = BufWriter::new(file);

            let mut ser = F::serializer(writer);
            let ser = ser.as_serializer();

            value.serialize(ser).map_err(Error::saving)?;

            Ok(())
        }

        fn load<'de, F: Format, T: DeserializeSeed<'de>>(
            &self,
            key: K,
            seed: T,
        ) -> Result<T::Value, Error> {
            let path = get_save_file(format!("{key}{}", F::extension()));
            let file = File::open(path)?;
            let reader = BufReader::new(file);

            let mut de = F::deserializer(reader);
            let de = de.as_deserializer();

            seed.deserialize(de).map_err(Error::loading)
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
        fn save<F: Format, T: Serialize>(&self, key: K, value: T) -> Result<(), Error> {
            let file = File::create(format!("{key}{}", F::extension()))?;
            let writer = BufWriter::new(file);

            let mut ser = F::serializer(writer);
            let ser = ser.as_serializer();

            value.serialize(ser).map_err(Error::saving)?;

            Ok(())
        }

        fn load<'de, F: Format, T: DeserializeSeed<'de>>(
            &self,
            key: K,
            seed: T,
        ) -> Result<T::Value, Error> {
            let file = File::open(format!("{key}{}", F::extension()))?;
            let reader = BufReader::new(file);

            let mut de = F::deserializer(reader);
            let de = de.as_deserializer();

            seed.deserialize(de).map_err(Error::loading)
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use desktop::FileIO;
#[cfg(not(target_arch = "wasm32"))]
/// The [`Backend`] the default [`Pipeline`](crate::Pipeline) will use.
pub type DefaultBackend = desktop::FileIO;
#[cfg(not(target_arch = "wasm32"))]
/// The [`Backend`] the default debug [`Pipeline`](crate::Pipeline) will use.
pub type DefaultDebugBackend = desktop::DebugFileIO;

#[cfg(target_arch = "wasm32")]
mod wasm {
    use bevy::prelude::*;

    #[allow(clippy::wildcard_imports)]
    use super::*;
    use crate::WORKSPACE;

    /// Simple `WebStorage` backend.
    #[derive(Default, Resource)]
    pub struct WebStorage;

    impl<'a> Backend<&'a str> for WebStorage {
        fn save<F: Format, T: Serialize>(&self, key: &str, value: T) -> Result<(), Error> {
            let storage = web_sys::window()
                .expect("No window")
                .local_storage()
                .expect("Failed to get local storage")
                .expect("No local storage");

            storage
                .set_item(
                    &format!("{WORKSPACE}.{key}"),
                    &serde_json::to_string(&value).map_err(Error::saving)?,
                )
                .expect("Failed to save");

            Ok(())
        }

        fn load<'de, F: Format, T: DeserializeSeed<'de>>(
            &self,
            key: &str,
            seed: T,
        ) -> Result<T::Value, Error> {
            let storage = web_sys::window()
                .expect("No window")
                .local_storage()
                .expect("Failed to get local storage")
                .expect("No local storage");

            let value = storage
                .get_item(&format!("{WORKSPACE}.{key}"))
                .expect("Failed to load")
                .ok_or(Error::custom("Invalid key"))?;

            let bytes = value.into_bytes();

            let mut de = serde_json::Deserializer::from_reader(bytes.as_slice());

            seed.deserialize(&mut de).map_err(Error::loading)
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm::WebStorage;
#[cfg(target_arch = "wasm32")]
/// The [`Backend`] the default [`Pipeline`](crate::Pipeline) will use.
pub type DefaultBackend = wasm::WebStorage;
#[cfg(target_arch = "wasm32")]
/// The [`Backend`] the default debug [`Pipeline`](crate::Pipeline) will use.
pub type DefaultDebugBackend = wasm::WebStorage;
