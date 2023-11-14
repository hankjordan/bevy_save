use std::{
    fs::File,
    io::{
        BufReader,
        BufWriter,
    },
};

use serde::{
    de::DeserializeSeed,
    Serialize,
};

use crate::{
    get_save_file,
    Error,
    Loader,
    Saver,
};

/// Interface between the [`Saver`] / [`Loader`] and the disk or other storage.
pub trait Backend<K> {
    /// Attempts to serialize a value with the given [`Saver`].
    ///
    /// # Errors
    /// - [`Error::Saving`] if serialization of the type fails
    /// - [`Error::IO`] if there is an IO or filesystem failure
    /// - See [`Error`]
    fn save<S: Saver, T: Serialize>(&self, key: K, value: T) -> Result<(), Error>;

    /// Attempts to deserialize a value with the given [`Loader`].
    ///
    /// # Errors
    /// - [`Error::Loading`] if deserialization of the type fails
    /// - [`Error::IO`] if there is an IO or filesystem failure
    /// - See [`Error`]
    fn load<'de, L: Loader, T: DeserializeSeed<'de>>(
        &self,
        key: K,
        seed: T,
    ) -> Result<T::Value, Error>;
}

#[cfg(not(target_arch = "wasm32"))]
mod desktop {
    use std::ffi::OsStr;

    use bevy::prelude::*;

    #[allow(clippy::wildcard_imports)]
    use super::*;
    use crate::{
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
        fn save<S: Saver, T: Serialize>(&self, key: K, value: T) -> Result<(), Error> {
            let path = get_save_file(key);
            let dir = path.parent().expect("Invalid save directory");

            std::fs::create_dir_all(dir)?;

            let file = File::create(path)?;
            let writer = BufWriter::new(file);

            let mut ser = S::build(writer);
            let ser = ser.as_serializer();

            // TODO
            value.serialize(ser).expect("Failed to save");

            Ok(())
        }

        fn load<'de, L: Loader, T: DeserializeSeed<'de>>(
            &self,
            key: K,
            seed: T,
        ) -> Result<T::Value, Error> {
            let path = get_save_file(key);
            let file = File::open(path)?;
            let reader = BufReader::new(file);

            let mut de = L::build(reader);
            let de = de.as_deserializer();

            // TODO
            Ok(seed.deserialize(de).expect("Failed to load"))
        }
    }

    /// Debug filesystem backend.
    ///
    /// Each name corresponds to an individual file on the disk.
    ///
    /// Files are stored relative to the active path.
    #[derive(Default, Resource)]
    pub struct DebugFileIO;

    impl<K: AsRef<OsStr>> Backend<K> for DebugFileIO {
        fn save<S: Saver, T: Serialize>(&self, key: K, value: T) -> Result<(), Error> {
            let path = std::path::Path::new(&key);
            let file = File::create(path)?;
            let writer = BufWriter::new(file);

            let mut ser = S::build(writer);
            let ser = ser.as_serializer();

            value.serialize(ser).map_err(Error::saving)?;

            Ok(())
        }

        fn load<'de, L: Loader, T: DeserializeSeed<'de>>(
            &self,
            key: K,
            seed: T,
        ) -> Result<T::Value, Error> {
            let path = std::path::Path::new(&key);
            let file = File::open(path)?;
            let reader = BufReader::new(file);

            let mut de = L::build(reader);
            let de = de.as_deserializer();

            seed.deserialize(de).map_err(Error::loading)
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

// TODO
#[cfg(target_arch = "wasm32")]
mod wasm {
    use bevy::prelude::*;
    use web_sys::Storage;

    #[allow(clippy::wildcard_imports)]
    use super::*;

    /// Simple `WebStorage` backend.
    #[derive(Default, Resource)]
    pub struct WebStorage;

    pub struct WebReader {
        value: String,
    }

    impl Read for WebReader {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            todo!()
        }
    }

    pub struct WebWriter {
        storage: Storage,
        key: String,
        value: String,
    }

    impl Write for WebWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            todo!()
        }

        fn flush(&mut self) -> std::io::Result<()> {
            todo!()
        }
    }

    impl<K> Backend2<K> for WebStorage {
        type Reader = WebReader;
        type Writer = WebWriter;

        fn reader(&mut self, key: K) -> Result<Self::Reader, Error> {
            todo!()
        }

        fn writer(&mut self, key: K) -> Result<Self::Writer, Error> {
            todo!()
        }
    }

    impl<'a> Backend<&'a str> for WebStorage {
        fn save<S: Saver, T: Serialize>(&self, key: K, value: T) -> Result<(), Error> {
            todo!()
        }

        fn load_seed<'de, L: Loader, T: DeserializeSeed<'de>>(
            &self,
            key: K,
            seed: T,
        ) -> Result<T::Value, Error> {
            todo!()
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm::WebStorage;
#[cfg(target_arch = "wasm32")]
pub type DefaultBackend = wasm::WebStorage;
#[cfg(target_arch = "wasm32")]
pub type DefaultDebugBackend = wasm::WebStorage;
